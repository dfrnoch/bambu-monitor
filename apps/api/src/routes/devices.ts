import { Elysia, t } from "elysia";
import { cameraWebRtcResponse } from "../devices/camera";
import { deviceManager } from "../devices/manager";
import { probeTcp } from "../devices/probe";
import { parsePort, sanitizeHost } from "../devices/store";
import type { DeviceCommand, DeviceEvent } from "../devices/types";

const deviceBody = t.Object({
	name: t.String({ minLength: 1 }),
	host: t.String({ minLength: 1 }),
	serial: t.String({ minLength: 1 }),
	accessCode: t.String({ minLength: 1 }),
	mqttPort: t.Optional(t.Number({ minimum: 1, maximum: 65_535 })),
	mqttUseTls: t.Optional(t.Boolean()),
	model: t.Optional(t.String()),
});

const probeBody = t.Object({
	host: t.String({ minLength: 1 }),
	mqttPort: t.Optional(t.Number({ minimum: 1, maximum: 65_535 })),
});

const commandBody = t.Object({
	command: t.Union([
		t.Literal("pause"),
		t.Literal("resume"),
		t.Literal("stop"),
		t.Literal("refresh"),
		t.Literal("connect"),
		t.Literal("disconnect"),
	]),
});

function sseMessage(event: DeviceEvent) {
	return `data: ${JSON.stringify(event)}\n\n`;
}

export const deviceRoute = new Elysia({ name: "devices", prefix: "/devices" })
	.get("/", () => deviceManager.list())
	.post("/", ({ body }) => deviceManager.create(body), { body: deviceBody })
	.post(
		"/probe",
		({ body }) => probeTcp(sanitizeHost(body.host), parsePort(body.mqttPort)),
		{ body: probeBody },
	)
	.put("/:id", ({ params, body }) => deviceManager.update(params.id, body), {
		body: t.Partial(deviceBody),
	})
	.delete("/:id", ({ params }) => deviceManager.delete(params.id))
	.post(
		"/:id/command",
		({ params, body }) =>
			deviceManager.command(params.id, body.command as DeviceCommand),
		{ body: commandBody },
	)
	.get("/:id/camera", async ({ params }) => {
		const config = await deviceManager.config(params.id);

		if (!config) {
			return new Response("Printer not found.", { status: 404 });
		}

		return cameraWebRtcResponse(config);
	})
	.get("/events", async () => {
		await deviceManager.ready();
		let heartbeat: Timer | null = null;
		let write: ((event: DeviceEvent) => void) | null = null;

		const stream = new ReadableStream({
			async start(controller) {
				const encoder = new TextEncoder();
				write = (event: DeviceEvent) =>
					controller.enqueue(encoder.encode(sseMessage(event)));
				heartbeat = setInterval(() => {
					controller.enqueue(encoder.encode(": keepalive\n\n"));
				}, 25_000);
				const devices = await deviceManager.list();

				write({ type: "devices", devices });
				deviceManager.on("event", write);
			},
			cancel() {
				if (heartbeat) {
					clearInterval(heartbeat);
				}

				if (write) {
					deviceManager.off("event", write);
				}
			},
		});

		return new Response(stream, {
			headers: {
				"Cache-Control": "no-cache",
				Connection: "keep-alive",
				"Content-Type": "text/event-stream",
			},
		});
	});
