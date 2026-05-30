import type { DeviceTelemetry } from "./types";

type BambuReport = {
	print?: Record<string, unknown>;
};

function numberOrNull(value: unknown): number | null {
	if (typeof value === "number" && Number.isFinite(value)) {
		return value;
	}

	if (typeof value === "string" && value.trim() !== "") {
		const parsed = Number(value);
		return Number.isFinite(parsed) ? parsed : null;
	}

	return null;
}

function stringOrNull(value: unknown): string | null {
	return typeof value === "string" && value.trim() !== "" ? value : null;
}

function recordOrNull(value: unknown): Record<string, unknown> | null {
	return value && typeof value === "object" && !Array.isArray(value)
		? (value as Record<string, unknown>)
		: null;
}

function arrayOrEmpty(value: unknown): unknown[] {
	return Array.isArray(value) ? value : [];
}

function parseAms(print: Record<string, unknown>): DeviceTelemetry["ams"] {
	const ams = recordOrNull(print.ams);
	const units = arrayOrEmpty(ams?.ams);
	const trays = units.flatMap((unit, unitIndex) => {
		const unitRecord = recordOrNull(unit);
		const unitId = stringOrNull(unitRecord?.id) ?? String(unitIndex);

		return arrayOrEmpty(unitRecord?.tray).map((tray, trayIndex) => {
			const trayRecord = recordOrNull(tray);
			const trayId =
				stringOrNull(trayRecord?.id) ??
				stringOrNull(trayRecord?.tray_id) ??
				String(trayIndex);

			return {
				id: `${unitId}-${trayId}`,
				material:
					stringOrNull(trayRecord?.tray_type) ??
					stringOrNull(trayRecord?.tray_sub_brands),
				color: stringOrNull(trayRecord?.tray_color),
				remaining:
					numberOrNull(trayRecord?.remain) ??
					numberOrNull(trayRecord?.tray_weight),
			};
		});
	});

	if (!ams && trays.length === 0) {
		return null;
	}

	return {
		activeTray: stringOrNull(print.tray_now),
		targetTray: stringOrNull(print.tray_tar),
		humidity: numberOrNull(ams?.humidity) ?? numberOrNull(print.ams_humidity),
		trayCount: trays.length,
		trays,
	};
}

export function parseTelemetry(payload: unknown): DeviceTelemetry | null {
	const report = payload as BambuReport;
	const print = report.print;

	if (!print) {
		return null;
	}

	return {
		state: stringOrNull(print.gcode_state) ?? "unknown",
		progress: numberOrNull(print.mc_percent),
		taskName: stringOrNull(print.subtask_name),
		layerCurrent: numberOrNull(print.layer_num),
		layerTotal: numberOrNull(print.total_layer_num),
		nozzleTemp: numberOrNull(print.nozzle_temper),
		nozzleTarget: numberOrNull(print.nozzle_target_temper),
		bedTemp: numberOrNull(print.bed_temper),
		bedTarget: numberOrNull(print.bed_target_temper),
		chamberTemp: numberOrNull(print.chamber_temper),
		amsHumidity: numberOrNull(print.ams_humidity),
		ams: parseAms(print),
		speedLevel: numberOrNull(print.spd_lvl),
		fanSpeed: numberOrNull(print.cooling_fan_speed),
		auxiliaryFanSpeed: numberOrNull(print.big_fan1_speed),
		chamberFanSpeed: numberOrNull(print.big_fan2_speed),
		heatbreakFanSpeed: numberOrNull(print.heatbreak_fan_speed),
		wifiSignal: stringOrNull(print.wifi_signal),
		remainingMinutes: numberOrNull(print.mc_remaining_time),
		rawUpdatedAt: new Date().toISOString(),
	};
}
