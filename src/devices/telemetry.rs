use super::*;

pub(super) fn parse_telemetry(payload: &Value) -> Option<DeviceTelemetry> {
    let print = payload.get("print")?.as_object()?;

    Some(DeviceTelemetry {
        state: string_or_none(print.get("gcode_state")).unwrap_or_else(|| "unknown".to_string()),
        progress: number_or_none(print.get("mc_percent")),
        task_name: string_or_none(print.get("subtask_name")),
        layer_current: number_or_none(print.get("layer_num")),
        layer_total: number_or_none(print.get("total_layer_num")),
        nozzle_temp: number_or_none(print.get("nozzle_temper")),
        nozzle_target: number_or_none(print.get("nozzle_target_temper")),
        bed_temp: number_or_none(print.get("bed_temper")),
        bed_target: number_or_none(print.get("bed_target_temper")),
        chamber_temp: number_or_none(print.get("chamber_temper")),
        ams_humidity: number_or_none(print.get("ams_humidity")),
        ams: parse_ams(print),
        speed_level: number_or_none(print.get("spd_lvl")),
        fan_speed: number_or_none(print.get("cooling_fan_speed")),
        auxiliary_fan_speed: number_or_none(print.get("big_fan1_speed")),
        chamber_fan_speed: number_or_none(print.get("big_fan2_speed")),
        heatbreak_fan_speed: number_or_none(print.get("heatbreak_fan_speed")),
        wifi_signal: string_or_none(print.get("wifi_signal")),
        remaining_minutes: number_or_none(print.get("mc_remaining_time")),
        raw_updated_at: Some(now_iso()),
    })
}

fn parse_ams(print: &serde_json::Map<String, Value>) -> Option<AmsTelemetry> {
    let ams = print.get("ams").and_then(Value::as_object);
    let units = ams
        .and_then(|ams| ams.get("ams"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut trays = Vec::new();

    for (unit_index, unit) in units.iter().enumerate() {
        let Some(unit) = unit.as_object() else {
            continue;
        };
        let unit_id = string_or_none(unit.get("id")).unwrap_or_else(|| unit_index.to_string());
        let unit_trays = unit
            .get("tray")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        for (tray_index, tray) in unit_trays.iter().enumerate() {
            let Some(tray) = tray.as_object() else {
                continue;
            };
            let tray_id = string_or_none(tray.get("id"))
                .or_else(|| string_or_none(tray.get("tray_id")))
                .unwrap_or_else(|| tray_index.to_string());

            trays.push(AmsTray {
                id: format!("{unit_id}-{tray_id}"),
                material: string_or_none(tray.get("tray_type"))
                    .or_else(|| string_or_none(tray.get("tray_sub_brands"))),
                color: string_or_none(tray.get("tray_color")),
                remaining: number_or_none(tray.get("remain"))
                    .or_else(|| number_or_none(tray.get("tray_weight"))),
            });
        }
    }

    if ams.is_none() && trays.is_empty() {
        return None;
    }

    Some(AmsTelemetry {
        active_tray: string_or_none(print.get("tray_now")),
        target_tray: string_or_none(print.get("tray_tar")),
        humidity: ams
            .and_then(|ams| number_or_none(ams.get("humidity")))
            .or_else(|| number_or_none(print.get("ams_humidity"))),
        tray_count: trays.len(),
        trays,
    })
}

fn number_or_none(value: Option<&Value>) -> Option<f64> {
    match value? {
        Value::Number(value) => value.as_f64(),
        Value::String(value) if !value.trim().is_empty() => value.parse::<f64>().ok(),
        _ => None,
    }
}

fn string_or_none(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(value) if !value.trim().is_empty() => Some(value.clone()),
        _ => None,
    }
}
