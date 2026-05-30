import type {
  DeviceCommand,
  DeviceCreateInput,
  DeviceProbeResult,
  DeviceSnapshot,
} from "@bambu-monitor/contracts";

const API_URL = import.meta.env.VITE_API_URL ?? "http://localhost:3000";

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${API_URL}${path}`, {
    ...init,
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      ...init?.headers,
    },
  });

  if (!response.ok) {
    throw new Error(await response.text());
  }

  return response.json() as Promise<T>;
}

export function eventsUrl() {
  return `${API_URL}/devices/events`;
}

export function cameraFeedUrl(id: string) {
  return `${API_URL}/devices/${id}/camera`;
}

export function listDevices() {
  return request<DeviceSnapshot[]>("/devices/");
}

export function createDevice(input: DeviceCreateInput) {
  return request<DeviceSnapshot>("/devices/", {
    method: "POST",
    body: JSON.stringify(input),
  });
}

export function probeDeviceConnection(input: {
  host: string;
  mqttPort?: number;
}) {
  return request<DeviceProbeResult>("/devices/probe", {
    method: "POST",
    body: JSON.stringify(input),
  });
}

export function deleteDevice(id: string) {
  return request<boolean>(`/devices/${id}`, { method: "DELETE" });
}

export function sendDeviceCommand(id: string, command: DeviceCommand) {
  return request<DeviceSnapshot | null>(`/devices/${id}/command`, {
    method: "POST",
    body: JSON.stringify({ command }),
  });
}
