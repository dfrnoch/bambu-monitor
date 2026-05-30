import { EventEmitter } from "node:events";
import mqtt, { type MqttClient } from "mqtt";
import { inferCamera } from "./camera";
import { parseTelemetry } from "./telemetry";
import type {
  DeviceCommand,
  DeviceConfig,
  DeviceConnection,
  DeviceSnapshot,
  DeviceTelemetry,
} from "./types";

type ClientEvents = {
  snapshot: [DeviceSnapshot];
};

export class BambuDeviceClient extends EventEmitter<ClientEvents> {
  #client: MqttClient | null = null;
  #connection: DeviceConnection = "offline";
  #error: string | null = null;
  #lastSeenAt: string | null = null;
  #telemetry: DeviceTelemetry | null = null;

  constructor(private config: DeviceConfig) {
    super();
  }

  get id() {
    return this.config.id;
  }

  updateConfig(config: DeviceConfig) {
    this.config = config;
    this.disconnect();
    this.connect();
  }

  snapshot(): DeviceSnapshot {
    const { accessCode: _accessCode, ...safeConfig } = this.config;

    return {
      config: safeConfig,
      connection: this.#connection,
      camera: inferCamera(safeConfig),
      lastSeenAt: this.#lastSeenAt,
      error: this.#error,
      telemetry: this.#telemetry,
    };
  }

  connect() {
    if (this.#client) {
      return;
    }

    this.#connection = "connecting";
    this.#error = null;
    this.#emitSnapshot();

    const protocol = this.config.mqttUseTls ? "mqtts" : "mqtt";

    this.#client = mqtt.connect(
      `${protocol}://${this.config.host}:${this.config.mqttPort}`,
      {
        username: "bblp",
        password: this.config.accessCode,
        rejectUnauthorized: false,
        reconnectPeriod: 5000,
        connectTimeout: 8000,
        clean: true,
        clientId: `bambu-monitor-${this.config.id}`,
      },
    );

    this.#client.on("connect", () => {
      this.#connection = "online";
      this.#error = null;
      this.#lastSeenAt = new Date().toISOString();
      this.#client?.subscribe(`device/${this.config.serial}/report`);
      this.requestPushAll();
      this.#emitSnapshot();
    });

    this.#client.on("message", (_topic, message) => {
      try {
        const payload = JSON.parse(message.toString());
        const telemetry = parseTelemetry(payload);

        if (telemetry) {
          this.#telemetry = telemetry;
          this.#lastSeenAt = new Date().toISOString();
          this.#connection = "online";
          this.#error = null;
          this.#emitSnapshot();
        }
      } catch (error) {
        this.#error = error instanceof Error ? error.message : String(error);
        this.#emitSnapshot();
      }
    });

    this.#client.on("error", (error) => {
      this.#connection = "error";
      this.#error = error.message;
      this.#emitSnapshot();
    });

    this.#client.on("close", () => {
      if (this.#client) {
        this.#connection = "connecting";
        this.#emitSnapshot();
      }
    });
  }

  disconnect() {
    const client = this.#client;
    this.#client = null;
    client?.end(true);
    this.#connection = "offline";
    this.#emitSnapshot();
  }

  requestPushAll() {
    this.#publish({
      pushing: {
        command: "pushall",
        sequence_id: Date.now().toString(),
        version: 1,
        push_target: 1,
      },
    });
  }

  send(command: DeviceCommand) {
    if (command === "connect") {
      this.connect();
      return;
    }

    if (command === "disconnect") {
      this.disconnect();
      return;
    }

    if (command === "refresh") {
      this.requestPushAll();
      return;
    }

    this.#publish({
      print: {
        command,
        sequence_id: Date.now().toString(),
      },
    });
  }

  #publish(payload: unknown) {
    this.#client?.publish(
      `device/${this.config.serial}/request`,
      JSON.stringify(payload),
    );
  }

  #emitSnapshot() {
    this.emit("snapshot", this.snapshot());
  }
}
