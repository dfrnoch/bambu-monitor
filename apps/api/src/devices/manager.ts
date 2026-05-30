import { EventEmitter } from "node:events";
import { BambuDeviceClient } from "./mqtt-client";
import {
	loadDevices,
	normalizeDevice,
	parsePort,
	sanitizeHost,
	saveDevices,
} from "./store";
import type {
	DeviceCommand,
	DeviceConfig,
	DeviceCreateInput,
	DeviceEvent,
	DeviceSnapshot,
} from "./types";

type ManagerEvents = {
	event: [DeviceEvent];
};

class DeviceManager extends EventEmitter<ManagerEvents> {
	#configs = new Map<string, DeviceConfig>();
	#clients = new Map<string, BambuDeviceClient>();
	#ready: Promise<void>;

	constructor() {
		super();
		this.#ready = this.#load();
	}

	async ready() {
		await this.#ready;
	}

	async list(): Promise<DeviceSnapshot[]> {
		await this.ready();
		return [...this.#clients.values()].map((client) => client.snapshot());
	}

	async create(input: DeviceCreateInput): Promise<DeviceSnapshot> {
		await this.ready();
		const config = normalizeDevice(input);
		this.#configs.set(config.id, config);
		await this.#persist();
		const client = this.#createClient(config);
		client.connect();
		this.#emitDevices();
		return client.snapshot();
	}

	async update(
		id: string,
		input: Partial<DeviceCreateInput>,
	): Promise<DeviceSnapshot | null> {
		await this.ready();
		const current = this.#configs.get(id);

		if (!current) {
			return null;
		}

		const next: DeviceConfig = {
			...current,
			...input,
			name: input.name?.trim() ?? current.name,
			host: input.host ? sanitizeHost(input.host) : current.host,
			serial: input.serial?.trim() ?? current.serial,
			accessCode: input.accessCode?.trim() ?? current.accessCode,
			mqttPort:
				input.mqttPort === undefined
					? current.mqttPort
					: parsePort(input.mqttPort),
			mqttUseTls: input.mqttUseTls ?? current.mqttUseTls,
		};
		const model = input.model?.trim();

		if (model) {
			next.model = model;
		}

		this.#configs.set(id, next);
		await this.#persist();
		const client = this.#clients.get(id);
		client?.updateConfig(next);
		this.#emitDevices();
		return client?.snapshot() ?? null;
	}

	async delete(id: string): Promise<boolean> {
		await this.ready();
		const client = this.#clients.get(id);
		client?.disconnect();
		this.#clients.delete(id);
		const deleted = this.#configs.delete(id);
		await this.#persist();
		this.#emitDevices();
		return deleted;
	}

	async command(
		id: string,
		command: DeviceCommand,
	): Promise<DeviceSnapshot | null> {
		await this.ready();
		const client = this.#clients.get(id);

		if (!client) {
			return null;
		}

		client.send(command);
		return client.snapshot();
	}

	async config(id: string): Promise<DeviceConfig | null> {
		await this.ready();

		return this.#configs.get(id) ?? null;
	}

	async #load() {
		const devices = await loadDevices();

		for (const config of devices) {
			this.#configs.set(config.id, config);
			const client = this.#createClient(config);

			if (Bun.env.AUTO_CONNECT !== "false") {
				client.connect();
			}
		}
	}

	#createClient(config: DeviceConfig) {
		const client = new BambuDeviceClient(config);
		client.on("snapshot", (device) =>
			this.emit("event", { type: "snapshot", device }),
		);
		this.#clients.set(config.id, client);
		return client;
	}

	async #persist() {
		await saveDevices([...this.#configs.values()]);
	}

	async #emitDevices() {
		this.emit("event", { type: "devices", devices: await this.list() });
	}
}

export const deviceManager = new DeviceManager();
