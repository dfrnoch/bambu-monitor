import { createConnection } from "node:net";
import type { DeviceProbeResult } from "./types";

export function probeTcp(
	host: string,
	port: number,
): Promise<DeviceProbeResult> {
	const startedAt = performance.now();

	return new Promise((resolve) => {
		const socket = createConnection({ host, port, timeout: 4000 });

		socket.once("connect", () => {
			const latencyMs = Math.round(performance.now() - startedAt);
			socket.end();
			resolve({ ok: true, host, port, error: null, latencyMs });
		});

		socket.once("timeout", () => {
			socket.destroy();
			resolve({
				ok: false,
				host,
				port,
				error: `Timed out connecting to ${host}:${port}`,
				latencyMs: null,
			});
		});

		socket.once("error", (error) => {
			resolve({
				ok: false,
				host,
				port,
				error: error.message,
				latencyMs: null,
			});
		});
	});
}
