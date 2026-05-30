import { spawn } from "node:child_process";
import type { DeviceCamera, DeviceConfig } from "./types";

const cameraPath = "/streaming/live/1";
const defaultGo2rtcUrl = "http://127.0.0.1:1984";
let go2rtcStarted = false;

export function inferCamera(
	config: Omit<DeviceConfig, "accessCode">,
): DeviceCamera {
	const camera = cameraDetails(config);

	return {
		supported: true,
		relayUrl: `/devices/${config.id}/camera`,
		port: camera.port,
		protocol: "rtsps",
		path: cameraPath,
		mode: "webrtc",
		streamName: streamName(config.id),
		message: "Camera is detected automatically and streamed through WebRTC.",
	};
}

export function cameraStreamUrl(config: DeviceConfig) {
	const camera = cameraDetails(config);

	return `rtsps://bblp:${encodeURIComponent(config.accessCode)}@${config.host}:${camera.port}${cameraPath}`;
}

export async function cameraWebRtcResponse(config: DeviceConfig) {
	const status = await ensureGo2rtcStream(config);

	if (!status.ok) {
		return htmlResponse(cameraErrorHtml(status.message), status.status);
	}

	const src = encodeURIComponent(streamName(config.id));
	const playerUrl = `${go2rtcPublicUrl()}/webrtc.html?src=${src}&media=video`;

	return htmlResponse(`
		<!doctype html>
		<html>
			<head>
				<meta charset="utf-8" />
				<meta name="viewport" content="width=device-width, initial-scale=1" />
				<style>
					html, body, iframe {
						background: #18181b;
						border: 0;
						height: 100%;
						margin: 0;
						overflow: hidden;
						width: 100%;
					}
				</style>
			</head>
			<body>
				<iframe
					allow="autoplay; fullscreen; microphone; camera"
					src="${escapeHtml(playerUrl)}"
					title="Bambu camera WebRTC player"
				></iframe>
			</body>
		</html>
	`);
}

async function ensureGo2rtcStream(config: DeviceConfig) {
	const available = await ensureGo2rtcAvailable();

	if (!available) {
		return {
			ok: false,
			status: 503,
			message:
				"WebRTC camera needs go2rtc running on the API host. Install go2rtc or set GO2RTC_URL, then reload the camera.",
		};
	}

	const baseUrl = go2rtcUrl();
	const source = cameraStreamUrl(config);
	const url = new URL("/api/streams", baseUrl);

	url.searchParams.set("name", streamName(config.id));
	url.searchParams.set("src", source);

	try {
		const response = await fetch(url, { method: "PUT" });

		if (response.ok) {
			return { ok: true, status: 200, message: "ok" };
		}

		return {
			ok: false,
			status: 502,
			message: `go2rtc rejected the printer stream (${response.status}).`,
		};
	} catch {
		return {
			ok: false,
			status: 503,
			message:
				"WebRTC camera needs go2rtc running on the API host. Start go2rtc on port 1984 and reload the camera.",
		};
	}
}

async function ensureGo2rtcAvailable() {
	if (await canReachGo2rtc()) {
		return true;
	}

	if (!go2rtcStarted) {
		go2rtcStarted = true;
		const process = spawn(Bun.env.GO2RTC_BIN ?? "go2rtc", [], {
			detached: true,
			stdio: "ignore",
		});

		process.once("error", () => {
			go2rtcStarted = false;
		});
		process.unref();
	}

	for (let attempt = 0; attempt < 10; attempt += 1) {
		await Bun.sleep(250);

		if (await canReachGo2rtc()) {
			return true;
		}
	}

	return false;
}

async function canReachGo2rtc() {
	try {
		const response = await fetch(new URL("/api/streams", go2rtcUrl()), {
			signal: AbortSignal.timeout(1000),
		});

		return response.ok;
	} catch {
		return false;
	}
}

function cameraDetails(config: Pick<DeviceConfig, "model">) {
	const model = config.model?.toLowerCase() ?? "";
	const isXOrH =
		model.includes("x1") || model.includes("x2") || model.includes("h2");
	const isPOrA =
		model.includes("p1") ||
		model.includes("p2") ||
		model.includes("a1") ||
		model.includes("a1 mini");

	return { port: isXOrH ? 322 : isPOrA ? 6000 : 322 };
}

function streamName(id: string) {
	return `bambu_${id.replaceAll("-", "_")}`;
}

function go2rtcUrl() {
	return (Bun.env.GO2RTC_URL ?? defaultGo2rtcUrl).replace(/\/$/, "");
}

function go2rtcPublicUrl() {
	return (Bun.env.GO2RTC_PUBLIC_URL ?? go2rtcUrl()).replace(/\/$/, "");
}

function htmlResponse(html: string, status = 200) {
	return new Response(html, {
		headers: {
			"Cache-Control": "no-store",
			"Content-Type": "text/html; charset=utf-8",
		},
		status,
	});
}

function cameraErrorHtml(message: string) {
	return `
		<!doctype html>
		<html>
			<head>
				<meta charset="utf-8" />
				<meta name="viewport" content="width=device-width, initial-scale=1" />
				<style>
					body {
						align-items: center;
						background: #18181b;
						color: #d4d4d8;
						display: flex;
						font: 14px system-ui, sans-serif;
						height: 100vh;
						justify-content: center;
						margin: 0;
						padding: 24px;
						text-align: center;
					}
				</style>
			</head>
			<body>${escapeHtml(message)}</body>
		</html>
	`;
}

function escapeHtml(value: string) {
	return value
		.replaceAll("&", "&amp;")
		.replaceAll("<", "&lt;")
		.replaceAll(">", "&gt;")
		.replaceAll('"', "&quot;");
}
