export type DeviceConnection = "offline" | "connecting" | "online" | "error";

export type DeviceConfig = {
	id: string;
	name: string;
	host: string;
	serial: string;
	accessCode: string;
	mqttPort: number;
	mqttUseTls: boolean;
	model?: string;
};

export type DeviceCamera = {
	supported: boolean;
	relayUrl: string | null;
	port: number | null;
	protocol: "rtsps" | null;
	path: string | null;
	mode: "webrtc";
	streamName: string | null;
	message: string;
};

export type DeviceCreateInput = Omit<
	DeviceConfig,
	"id" | "mqttPort" | "mqttUseTls"
> & {
	mqttPort?: number;
	mqttUseTls?: boolean;
};

export type DeviceProbeResult = {
	ok: boolean;
	host: string;
	port: number;
	error: string | null;
	latencyMs: number | null;
};

export type DeviceTelemetry = {
	state: string;
	progress: number | null;
	taskName: string | null;
	layerCurrent: number | null;
	layerTotal: number | null;
	nozzleTemp: number | null;
	nozzleTarget: number | null;
	bedTemp: number | null;
	bedTarget: number | null;
	chamberTemp: number | null;
	amsHumidity: number | null;
	ams: {
		activeTray: string | null;
		targetTray: string | null;
		humidity: number | null;
		trayCount: number;
		trays: Array<{
			id: string;
			material: string | null;
			color: string | null;
			remaining: number | null;
		}>;
	} | null;
	speedLevel: number | null;
	fanSpeed: number | null;
	auxiliaryFanSpeed: number | null;
	chamberFanSpeed: number | null;
	heatbreakFanSpeed: number | null;
	wifiSignal: string | null;
	remainingMinutes: number | null;
	rawUpdatedAt: string | null;
};

export type DeviceSnapshot = {
	config: Omit<DeviceConfig, "accessCode">;
	connection: DeviceConnection;
	camera: DeviceCamera;
	lastSeenAt: string | null;
	error: string | null;
	telemetry: DeviceTelemetry | null;
};

export type DeviceCommand =
	| "pause"
	| "resume"
	| "stop"
	| "refresh"
	| "connect"
	| "disconnect";

export type DeviceEvent =
	| {
			type: "snapshot";
			device: DeviceSnapshot;
	  }
	| {
			type: "devices";
			devices: DeviceSnapshot[];
	  };
