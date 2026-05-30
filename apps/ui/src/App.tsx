import type { DeviceCommand, DeviceSnapshot } from "@bambu-monitor/contracts";
import {
  Activity,
  Box,
  Camera,
  ChevronDown,
  Clock3,
  CloudOff,
  Fan,
  Gauge,
  Layers3,
  Pause,
  Play,
  Plus,
  Power,
  RefreshCw,
  Square,
  Thermometer,
  Trash2,
  X,
} from "lucide-react";
import type { FormEvent, ReactNode } from "react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Toaster, toast } from "sonner";
import {
  cameraFeedUrl,
  createDevice,
  deleteDevice,
  eventsUrl,
  listDevices,
  probeDeviceConnection,
  sendDeviceCommand,
} from "@/lib/queries";

const printerTypes = [
  "A1 mini",
  "A1",
  "P1P",
  "P1S",
  "X1 Carbon",
  "X1E",
  "H2D",
  "Other Bambu printer",
];

const commandLabels: Record<DeviceCommand, string> = {
  pause: "Pause",
  resume: "Resume",
  stop: "Stop",
  refresh: "Refresh",
  connect: "Connect",
  disconnect: "Disconnect",
};

function formatTemperature(
  value: number | null | undefined,
  target?: number | null,
) {
  if (value == null) {
    return "--";
  }

  return target == null ? `${value}C` : `${value}/${target}C`;
}

function formatPercent(value: number | null | undefined) {
  return value == null ? "--" : `${value}%`;
}

function formatRemaining(minutes: number | null | undefined) {
  if (minutes == null) {
    return "--";
  }

  const hours = Math.floor(minutes / 60);
  const rest = minutes % 60;

  return hours > 0 ? `${hours}h ${rest}m` : `${rest}m`;
}

function formatTime(value: string | null) {
  return value ? new Date(value).toLocaleTimeString() : "never";
}

function cssColor(value: string | null) {
  if (!value) {
    return "#e5e7eb";
  }

  const hex = value.replace("#", "").slice(0, 6);

  return /^[0-9a-f]{6}$/i.test(hex) ? `#${hex}` : "#e5e7eb";
}

function connectionLabel(connection: DeviceSnapshot["connection"]) {
  if (connection === "online") {
    return "Online";
  }

  if (connection === "connecting") {
    return "Connecting";
  }

  if (connection === "error") {
    return "Attention";
  }

  return "Offline";
}

function connectionClass(connection: DeviceSnapshot["connection"]) {
  if (connection === "online") {
    return "bg-emerald-500";
  }

  if (connection === "connecting") {
    return "bg-amber-400";
  }

  if (connection === "error") {
    return "bg-red-500";
  }

  return "bg-zinc-400";
}

function MainPrinterView({
  device,
  onCommand,
  onDelete,
}: {
  device: DeviceSnapshot;
  onCommand: (id: string, command: DeviceCommand) => void;
  onDelete: (id: string) => void;
}) {
  const telemetry = device.telemetry;
  const progress = Math.max(0, Math.min(100, telemetry?.progress ?? 0));
  const layer =
    telemetry?.layerCurrent && telemetry.layerTotal
      ? `${telemetry.layerCurrent}/${telemetry.layerTotal}`
      : "--";
  const isOnline = device.connection === "online";

  return (
    <section className="flex flex-col gap-3 pb-28">
      <section className="overflow-hidden rounded-[2rem] bg-zinc-950 text-white shadow-sm">
        <div className="flex items-center justify-between gap-3 px-4 pb-3 pt-4">
          <div className="min-w-0">
            <div className="flex items-center gap-2 text-xs font-semibold uppercase tracking-wide text-zinc-400">
              <span>{device.config.model ?? "Bambu printer"}</span>
              <span className="size-1 rounded-full bg-zinc-700" />
              <span>
                {telemetry?.state ?? connectionLabel(device.connection)}
              </span>
            </div>
            <h1 className="truncate text-2xl font-semibold">
              {device.config.name}
            </h1>
          </div>
          <StatusPill connection={device.connection} />
        </div>

        <CameraFeed device={device} />

        <div className="grid gap-3 p-4">
          <div className="flex items-start justify-between gap-4">
            <div className="min-w-0">
              <p className="text-xs font-semibold uppercase tracking-wide text-emerald-300">
                Current print
              </p>
              <p className="mt-1 line-clamp-2 text-xl font-semibold leading-tight">
                {telemetry?.taskName ?? "No active print"}
              </p>
            </div>
            <div className="text-right">
              <p className="text-4xl font-semibold tabular-nums">{progress}%</p>
              <p className="text-xs font-medium text-zinc-400">complete</p>
            </div>
          </div>

          <div>
            <div className="h-3 rounded-full bg-white/10">
              <div
                className="h-full rounded-full bg-emerald-400 transition-[width]"
                style={{ width: `${progress}%` }}
              />
            </div>
            <div className="mt-3 grid grid-cols-2 gap-2">
              <HeroStat icon={<Layers3 />} label="Layer" value={layer} />
              <HeroStat
                icon={<Clock3 />}
                label="Remaining"
                value={formatRemaining(telemetry?.remainingMinutes)}
              />
            </div>
          </div>
        </div>
      </section>

      {device.error ? (
        <div className="rounded-3xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
          {device.error}
        </div>
      ) : null}

      <div className="grid grid-cols-2 gap-3">
        <TemperaturePanel device={device} />
        <SpeedPanel device={device} />
      </div>

      <AmsPanel device={device} />
      <FanPanel device={device} />

      <section className="rounded-[1.75rem] bg-white p-4 shadow-sm ring-1 ring-zinc-200">
        <div className="flex items-center justify-between gap-3">
          <div className="min-w-0">
            <p className="text-sm font-semibold text-zinc-950">Connection</p>
            <p className="truncate text-xs text-zinc-500">
              {device.config.host} · last update {formatTime(device.lastSeenAt)}
            </p>
          </div>
          <div className="flex gap-2">
            <button
              className="grid size-10 place-items-center rounded-full bg-zinc-100 text-zinc-700"
              type="button"
              aria-label={
                device.connection === "offline"
                  ? "Connect printer"
                  : "Disconnect printer"
              }
              onClick={() =>
                onCommand(
                  device.config.id,
                  device.connection === "offline" ? "connect" : "disconnect",
                )
              }
            >
              {device.connection === "offline" ? (
                <Power className="size-5" />
              ) : (
                <CloudOff className="size-5" />
              )}
            </button>
            <button
              className="grid size-10 place-items-center rounded-full bg-red-50 text-red-600"
              type="button"
              aria-label={`Delete ${device.config.name}`}
              onClick={() => onDelete(device.config.id)}
            >
              <Trash2 className="size-5" />
            </button>
          </div>
        </div>
      </section>

      <ControlBar device={device} isOnline={isOnline} onCommand={onCommand} />
    </section>
  );
}

function StatusPill({
  connection,
}: {
  connection: DeviceSnapshot["connection"];
}) {
  return (
    <div className="flex shrink-0 items-center gap-2 rounded-full bg-white/10 px-3 py-1.5 text-xs font-semibold">
      <span className={`size-2 rounded-full ${connectionClass(connection)}`} />
      {connectionLabel(connection)}
    </div>
  );
}

function CameraFeed({ device }: { device: DeviceSnapshot }) {
  const camera = device.camera;

  return (
    <div className="relative mx-3 aspect-[4/3] overflow-hidden rounded-[1.5rem] bg-zinc-900 ring-1 ring-white/10">
      {camera.supported && camera.relayUrl ? (
        <iframe
          allow="autoplay; fullscreen"
          alt={`${device.config.name} live camera`}
          className="size-full border-0"
          src={cameraFeedUrl(device.config.id)}
          title={`${device.config.name} live camera`}
        />
      ) : (
        <div className="grid size-full place-items-center px-8 text-center">
          <div className="flex flex-col items-center gap-3 text-zinc-400">
            <Camera className="size-9" />
            <div>
              <p className="text-sm font-semibold text-zinc-300">
                Camera auto-detected
              </p>
              <p className="mt-1 text-xs leading-5">{camera.message}</p>
            </div>
          </div>
        </div>
      )}
      <div className="absolute left-3 top-3 rounded-full bg-black/55 px-3 py-1 text-xs font-semibold text-white backdrop-blur">
        {camera.mode.toUpperCase()} · {camera.protocol ?? "rtsps"}:
        {camera.port ?? "--"}
      </div>
    </div>
  );
}

function HeroStat({
  icon,
  label,
  value,
}: {
  icon: ReactNode;
  label: string;
  value: string;
}) {
  return (
    <div className="rounded-2xl bg-white/10 px-3 py-2">
      <div className="mb-1 flex items-center gap-1.5 text-xs font-medium text-zinc-400">
        <span className="[&_svg]:size-3.5">{icon}</span>
        {label}
      </div>
      <p className="text-lg font-semibold tabular-nums">{value}</p>
    </div>
  );
}

function TemperaturePanel({ device }: { device: DeviceSnapshot }) {
  const telemetry = device.telemetry;

  return (
    <Panel title="Temperatures" icon={<Thermometer />}>
      <div className="grid gap-3">
        <ValueRow
          label="Nozzle"
          value={formatTemperature(
            telemetry?.nozzleTemp,
            telemetry?.nozzleTarget,
          )}
        />
        <ValueRow
          label="Bed"
          value={formatTemperature(telemetry?.bedTemp, telemetry?.bedTarget)}
        />
        <ValueRow
          label="Chamber"
          value={formatTemperature(telemetry?.chamberTemp)}
        />
      </div>
    </Panel>
  );
}

function SpeedPanel({ device }: { device: DeviceSnapshot }) {
  const speed = device.telemetry?.speedLevel;
  const label =
    speed === 1
      ? "Silent"
      : speed === 2
        ? "Standard"
        : speed === 3
          ? "Sport"
          : speed === 4
            ? "Ludicrous"
            : "--";

  return (
    <Panel title="Motion" icon={<Gauge />}>
      <div className="grid gap-3">
        <ValueRow label="Mode" value={label} />
        <ValueRow
          label="Speed level"
          value={speed == null ? "--" : `${speed}`}
        />
        <ValueRow label="State" value={device.telemetry?.state ?? "--"} />
      </div>
    </Panel>
  );
}

function AmsPanel({ device }: { device: DeviceSnapshot }) {
  const ams = device.telemetry?.ams;
  const activeTray = ams?.activeTray ?? "--";
  const trays = ams?.trays ?? [];

  return (
    <Panel title="AMS & material" icon={<Box />}>
      <div className="grid grid-cols-3 gap-2">
        <CompactValue label="Active" value={activeTray} />
        <CompactValue label="Target" value={ams?.targetTray ?? "--"} />
        <CompactValue
          label="Humidity"
          value={ams?.humidity == null ? "--" : `${ams.humidity}`}
        />
      </div>
      <div className="mt-3 grid gap-2">
        {trays.length > 0 ? (
          trays.slice(0, 8).map((tray) => (
            <div
              className="flex items-center justify-between rounded-2xl bg-zinc-50 px-3 py-2"
              key={tray.id}
            >
              <div className="flex min-w-0 items-center gap-2">
                <span
                  className="size-4 rounded-full ring-1 ring-zinc-300"
                  style={{ backgroundColor: cssColor(tray.color) }}
                />
                <span className="truncate text-sm font-medium text-zinc-800">
                  {tray.id}
                </span>
              </div>
              <span className="truncate text-sm text-zinc-500">
                {tray.material ?? "Material"}
              </span>
            </div>
          ))
        ) : (
          <p className="rounded-2xl bg-zinc-50 px-3 py-3 text-sm text-zinc-500">
            No AMS tray data reported yet.
          </p>
        )}
      </div>
    </Panel>
  );
}

function FanPanel({ device }: { device: DeviceSnapshot }) {
  const telemetry = device.telemetry;

  return (
    <Panel title="Cooling" icon={<Fan />}>
      <div className="grid grid-cols-2 gap-2">
        <CompactValue label="Part" value={formatPercent(telemetry?.fanSpeed)} />
        <CompactValue
          label="Aux"
          value={formatPercent(telemetry?.auxiliaryFanSpeed)}
        />
        <CompactValue
          label="Chamber"
          value={formatPercent(telemetry?.chamberFanSpeed)}
        />
        <CompactValue
          label="Hotend"
          value={formatPercent(telemetry?.heatbreakFanSpeed)}
        />
      </div>
    </Panel>
  );
}

function Panel({
  children,
  icon,
  title,
}: {
  children: ReactNode;
  icon: ReactNode;
  title: string;
}) {
  return (
    <section className="rounded-[1.75rem] bg-white p-4 shadow-sm ring-1 ring-zinc-200">
      <div className="mb-3 flex items-center gap-2">
        <div className="grid size-8 place-items-center rounded-full bg-zinc-100 text-zinc-600 [&_svg]:size-4">
          {icon}
        </div>
        <h2 className="text-sm font-semibold text-zinc-950">{title}</h2>
      </div>
      {children}
    </section>
  );
}

function ValueRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-baseline justify-between gap-3">
      <span className="text-sm text-zinc-500">{label}</span>
      <span className="truncate text-right text-base font-semibold tabular-nums text-zinc-950">
        {value}
      </span>
    </div>
  );
}

function CompactValue({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-2xl bg-zinc-50 px-3 py-2">
      <p className="text-xs font-medium text-zinc-500">{label}</p>
      <p className="mt-1 truncate text-sm font-semibold tabular-nums text-zinc-950">
        {value}
      </p>
    </div>
  );
}

function ControlBar({
  device,
  isOnline,
  onCommand,
}: {
  device: DeviceSnapshot;
  isOnline: boolean;
  onCommand: (id: string, command: DeviceCommand) => void;
}) {
  return (
    <nav className="fixed inset-x-0 bottom-0 z-10 bg-zinc-100/90 px-3 pb-3 pt-2 backdrop-blur">
      <div className="mx-auto grid max-w-3xl grid-cols-4 gap-2 rounded-[1.75rem] bg-white p-2 shadow-lg ring-1 ring-zinc-200">
        <CommandButton
          command="refresh"
          icon={<RefreshCw />}
          onCommand={onCommand}
          device={device}
        />
        <CommandButton
          command={isOnline ? "pause" : "connect"}
          icon={isOnline ? <Pause /> : <Power />}
          onCommand={onCommand}
          device={device}
        />
        <CommandButton
          command="resume"
          icon={<Play />}
          onCommand={onCommand}
          device={device}
        />
        <CommandButton
          command="stop"
          icon={<Square />}
          onCommand={onCommand}
          device={device}
          danger
        />
      </div>
    </nav>
  );
}

function CommandButton({
  command,
  danger = false,
  device,
  icon,
  onCommand,
}: {
  command: DeviceCommand;
  danger?: boolean;
  device: DeviceSnapshot;
  icon: ReactNode;
  onCommand: (id: string, command: DeviceCommand) => void;
}) {
  return (
    <button
      className={`flex min-h-16 flex-col items-center justify-center gap-1 rounded-3xl text-xs font-semibold ${
        danger
          ? "bg-red-50 text-red-600"
          : "bg-zinc-50 text-zinc-900 active:bg-zinc-100"
      }`}
      type="button"
      onClick={() => onCommand(device.config.id, command)}
    >
      <span className="text-current [&_svg]:size-5">{icon}</span>
      <span className="max-w-full truncate px-1">{commandLabels[command]}</span>
    </button>
  );
}

function AddPrinterSheet({
  onClose,
  onCreated,
  open,
}: {
  onClose: () => void;
  onCreated: () => void;
  open: boolean;
}) {
  const [saving, setSaving] = useState(false);
  const [probing, setProbing] = useState(false);
  const [probeMessage, setProbeMessage] = useState<string | null>(null);
  const formRef = useRef<HTMLFormElement>(null);

  function getFormValues(formElement: HTMLFormElement) {
    const form = new FormData(formElement);

    return {
      name: String(form.get("name") ?? ""),
      host: String(form.get("host") ?? ""),
      serial: String(form.get("serial") ?? ""),
      accessCode: String(form.get("accessCode") ?? ""),
      model: String(form.get("model") ?? ""),
      mqttPort: 8883,
      mqttUseTls: true,
    };
  }

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSaving(true);
    const formElement = event.currentTarget;

    try {
      await createDevice(getFormValues(formElement));
      formElement.reset();
      setProbeMessage(null);
      onClose();
      onCreated();
      toast.success("Printer added");
    } catch (error) {
      toast.error(
        error instanceof Error ? error.message : "Could not add printer",
      );
    } finally {
      setSaving(false);
    }
  }

  async function probe() {
    const formElement = formRef.current;

    if (!formElement) {
      return;
    }

    setProbing(true);
    setProbeMessage(null);

    try {
      const values = getFormValues(formElement);
      const result = await probeDeviceConnection({
        host: values.host,
        mqttPort: 8883,
      });

      setProbeMessage(
        result.ok
          ? `Printer reachable in ${result.latencyMs} ms`
          : (result.error ?? "Connection test failed"),
      );
    } catch (error) {
      setProbeMessage(error instanceof Error ? error.message : "Probe failed");
    } finally {
      setProbing(false);
    }
  }

  if (!open) {
    return null;
  }

  return (
    <div className="fixed inset-0 z-20 flex items-end bg-zinc-950/40 p-2 backdrop-blur-sm sm:items-center sm:justify-center">
      <div className="max-h-[94dvh] w-full overflow-auto rounded-[2rem] bg-zinc-50 p-4 shadow-2xl sm:max-w-lg">
        <div className="mb-4 flex items-center justify-between">
          <div>
            <h2 className="text-xl font-semibold text-zinc-950">Add printer</h2>
            <p className="text-sm text-zinc-500">
              Camera and LAN settings are detected automatically.
            </p>
          </div>
          <button
            aria-label="Close add printer"
            className="grid size-10 place-items-center rounded-full bg-white text-zinc-700 shadow-sm ring-1 ring-zinc-200"
            type="button"
            onClick={onClose}
          >
            <X className="size-5" />
          </button>
        </div>

        <form className="grid gap-3" onSubmit={submit} ref={formRef}>
          <Field name="name" label="Name" placeholder="Living room printer" />
          <SelectField
            name="model"
            label="Printer type"
            options={printerTypes}
          />
          <Field name="host" label="Printer IP" placeholder="192.168.10.13" />
          <Field name="serial" label="Serial number" placeholder="01P00A..." />
          <Field
            name="accessCode"
            label="Access code"
            placeholder="LAN access code"
            type="password"
          />

          {probeMessage ? (
            <p className="rounded-2xl bg-zinc-900 px-4 py-3 text-sm text-white">
              {probeMessage}
            </p>
          ) : null}

          <div className="grid grid-cols-2 gap-2 pt-1">
            <button
              className="h-12 rounded-2xl bg-white text-sm font-semibold text-zinc-900 shadow-sm ring-1 ring-zinc-200 disabled:opacity-60"
              disabled={probing}
              type="button"
              onClick={probe}
            >
              {probing ? "Testing..." : "Test"}
            </button>
            <button
              className="h-12 rounded-2xl bg-zinc-950 text-sm font-semibold text-white shadow-sm disabled:opacity-60"
              disabled={saving}
              type="submit"
            >
              {saving ? "Adding..." : "Add printer"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

function Field({
  label,
  name,
  placeholder,
  required = true,
  type = "text",
}: {
  label: string;
  name: string;
  placeholder: string;
  required?: boolean;
  type?: string;
}) {
  return (
    <label className="grid gap-1.5 text-sm font-medium text-zinc-700">
      {label}
      <input
        className="h-12 rounded-2xl border-0 bg-white px-4 text-base text-zinc-950 shadow-sm ring-1 ring-zinc-200 outline-none transition focus:ring-2 focus:ring-emerald-500"
        name={name}
        placeholder={placeholder}
        required={required}
        type={type}
      />
    </label>
  );
}

function SelectField({
  label,
  name,
  options,
}: {
  label: string;
  name: string;
  options: string[];
}) {
  return (
    <label className="grid gap-1.5 text-sm font-medium text-zinc-700">
      {label}
      <div className="relative">
        <select
          className="h-12 w-full appearance-none rounded-2xl border-0 bg-white px-4 pr-10 text-base text-zinc-950 shadow-sm ring-1 ring-zinc-200 outline-none transition focus:ring-2 focus:ring-emerald-500"
          defaultValue={options[0]}
          name={name}
          required
        >
          {options.map((option) => (
            <option key={option} value={option}>
              {option}
            </option>
          ))}
        </select>
        <ChevronDown className="pointer-events-none absolute right-4 top-1/2 size-4 -translate-y-1/2 text-zinc-500" />
      </div>
    </label>
  );
}

function EmptyState({ onAdd }: { onAdd: () => void }) {
  return (
    <div className="grid min-h-[60dvh] place-items-center rounded-[2rem] bg-white p-8 text-center shadow-sm ring-1 ring-zinc-200">
      <div>
        <div className="mx-auto mb-4 grid size-16 place-items-center rounded-full bg-emerald-50 text-emerald-600">
          <Plus className="size-7" />
        </div>
        <h1 className="text-2xl font-semibold text-zinc-950">
          Add your printer
        </h1>
        <p className="mt-2 text-sm text-zinc-500">
          Connect a Bambu printer on your LAN to see print progress, camera,
          AMS, temperatures, and controls.
        </p>
        <button
          className="mt-6 h-12 rounded-2xl bg-zinc-950 px-6 text-sm font-semibold text-white"
          type="button"
          onClick={onAdd}
        >
          Add printer
        </button>
      </div>
    </div>
  );
}

export function App() {
  const [devices, setDevices] = useState<DeviceSnapshot[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [adding, setAdding] = useState(false);

  const selectedDevice = useMemo(
    () =>
      devices.find((device) => device.config.id === selectedId) ?? devices[0],
    [devices, selectedId],
  );

  const refresh = useCallback(async () => {
    const nextDevices = await listDevices();
    setDevices(nextDevices);
    setLoading(false);
  }, []);

  useEffect(() => {
    if (!selectedDevice && devices.length > 0) {
      setSelectedId(devices[0]?.config.id ?? null);
    }
  }, [devices, selectedDevice]);

  useEffect(() => {
    refresh().catch((error) => {
      setLoading(false);
      toast.error(error instanceof Error ? error.message : "API unavailable");
    });

    const events = new EventSource(eventsUrl(), { withCredentials: true });

    events.onmessage = (message) => {
      const event = JSON.parse(message.data) as
        | { type: "devices"; devices: DeviceSnapshot[] }
        | { type: "snapshot"; device: DeviceSnapshot };

      if (event.type === "devices") {
        setDevices(event.devices);
        return;
      }

      setDevices((current) =>
        current.map((device) =>
          device.config.id === event.device.config.id ? event.device : device,
        ),
      );
    };

    events.onerror = () => {
      toast.error("Live updates disconnected");
      events.close();
    };

    return () => events.close();
  }, [refresh]);

  async function onCommand(id: string, command: DeviceCommand) {
    try {
      await sendDeviceCommand(id, command);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Command failed");
    }
  }

  async function onDelete(id: string) {
    try {
      await deleteDevice(id);
      setDevices((current) =>
        current.filter((device) => device.config.id !== id),
      );
      if (selectedId === id) {
        setSelectedId(null);
      }
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Delete failed");
    }
  }

  return (
    <main className="min-h-dvh bg-zinc-100 text-zinc-950">
      <div className="mx-auto flex w-full max-w-3xl flex-col gap-3 px-3 pb-8 pt-3 sm:px-5">
        <header className="sticky top-0 z-10 -mx-3 bg-zinc-100/90 px-3 pb-2 pt-3 backdrop-blur sm:-mx-5 sm:px-5">
          <div className="flex items-center gap-2">
            <div className="min-w-0 flex-1">
              <label className="sr-only" htmlFor="printer-switcher">
                Selected printer
              </label>
              <div className="relative">
                <select
                  className="h-12 w-full appearance-none rounded-2xl border-0 bg-white px-4 pr-10 text-base font-semibold text-zinc-950 shadow-sm ring-1 ring-zinc-200 outline-none"
                  disabled={devices.length === 0}
                  id="printer-switcher"
                  value={selectedDevice?.config.id ?? ""}
                  onChange={(event) => setSelectedId(event.target.value)}
                >
                  {devices.length === 0 ? (
                    <option value="">No printers</option>
                  ) : null}
                  {devices.map((device) => (
                    <option key={device.config.id} value={device.config.id}>
                      {device.config.name}
                    </option>
                  ))}
                </select>
                <ChevronDown className="pointer-events-none absolute right-4 top-1/2 size-4 -translate-y-1/2 text-zinc-500" />
              </div>
            </div>
            <button
              className="grid size-12 place-items-center rounded-2xl bg-white text-zinc-700 shadow-sm ring-1 ring-zinc-200"
              type="button"
              aria-label="Refresh printers"
              onClick={() => refresh()}
            >
              <Activity className="size-5" />
            </button>
            <button
              className="grid size-12 place-items-center rounded-2xl bg-zinc-950 text-white shadow-sm"
              type="button"
              aria-label="Add printer"
              onClick={() => setAdding(true)}
            >
              <Plus className="size-5" />
            </button>
          </div>
        </header>

        {selectedDevice ? (
          <MainPrinterView
            device={selectedDevice}
            onCommand={onCommand}
            onDelete={onDelete}
          />
        ) : null}

        {!loading && devices.length === 0 ? (
          <EmptyState onAdd={() => setAdding(true)} />
        ) : null}
      </div>

      <AddPrinterSheet
        open={adding}
        onClose={() => setAdding(false)}
        onCreated={refresh}
      />
      <Toaster position="top-center" richColors />
    </main>
  );
}

export default App;
