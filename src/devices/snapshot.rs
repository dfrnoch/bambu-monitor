use super::*;

pub(super) fn normalize_new_device(input: DeviceCreateInput) -> DeviceConfig {
    DeviceConfig {
        id: Uuid::new_v4().to_string(),
        name: input.name.trim().to_string(),
        host: sanitize_host(&input.host),
        serial: input.serial.trim().to_string(),
        access_code: input.access_code.trim().to_string(),
        mqtt_port: input.mqtt_port.map(parse_port).unwrap_or(8883),
        mqtt_use_tls: input.mqtt_use_tls.unwrap_or(true),
        model: input.model.and_then(|value| non_empty(Some(value))),
    }
}

pub(super) fn normalize_stored_device(input: DeviceConfig) -> DeviceConfig {
    DeviceConfig {
        host: sanitize_host(&input.host),
        mqtt_port: parse_port(input.mqtt_port),
        mqtt_use_tls: input.mqtt_use_tls,
        ..input
    }
}

pub(super) fn snapshot_from_state(config: &DeviceConfig, state: ClientState) -> DeviceSnapshot {
    DeviceSnapshot {
        config: SafeDeviceConfig {
            id: config.id.clone(),
            name: config.name.clone(),
            host: config.host.clone(),
            serial: config.serial.clone(),
            mqtt_port: config.mqtt_port,
            mqtt_use_tls: config.mqtt_use_tls,
            model: config.model.clone(),
        },
        connection: state.connection,
        camera: infer_camera(config),
        last_seen_at: state.last_seen_at,
        error: state.error,
        telemetry: state.telemetry,
    }
}
