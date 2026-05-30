import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import type { DeviceConfig, DeviceCreateInput } from "./types";

const dataDir = Bun.env.DATA_DIR ?? join(process.cwd(), "data");
const devicesPath = join(dataDir, "devices.json");

async function persist(devices: DeviceConfig[]) {
	await mkdir(dirname(devicesPath), { recursive: true });
	await writeFile(devicesPath, JSON.stringify(devices, null, 2));
}

export async function loadDevices(): Promise<DeviceConfig[]> {
	try {
		const text = await readFile(devicesPath, "utf8");
		const parsed = JSON.parse(text);
		return Array.isArray(parsed)
			? parsed.map((device) => normalizeStoredDevice(device))
			: [];
	} catch (error) {
		if ((error as NodeJS.ErrnoException).code === "ENOENT") {
			await persist([]);
			return [];
		}

		throw error;
	}
}

export async function saveDevices(devices: DeviceConfig[]) {
	await persist(devices);
}

export function parsePort(value: unknown) {
	const port = typeof value === "number" ? value : Number(value);

	if (Number.isInteger(port) && port > 0 && port <= 65_535) {
		return port;
	}

	return 8883;
}

export function sanitizeHost(value: string) {
	const trimmed = value.trim();

	if (!trimmed.includes("://")) {
		return trimmed.split(":")[0] ?? trimmed;
	}

	try {
		return new URL(trimmed).hostname;
	} catch {
		return trimmed;
	}
}

function normalizeStoredDevice(input: DeviceConfig): DeviceConfig {
	return {
		...input,
		host: sanitizeHost(input.host),
		mqttPort: parsePort(input.mqttPort),
		mqttUseTls: input.mqttUseTls ?? true,
	};
}

export function normalizeDevice(input: DeviceCreateInput): DeviceConfig {
	const device: DeviceConfig = {
		id: crypto.randomUUID(),
		name: input.name.trim(),
		host: sanitizeHost(input.host),
		serial: input.serial.trim(),
		accessCode: input.accessCode.trim(),
		mqttPort: parsePort(input.mqttPort),
		mqttUseTls: input.mqttUseTls ?? true,
	};

	const model = input.model?.trim();

	if (model) {
		device.model = model;
	}

	return device;
}
