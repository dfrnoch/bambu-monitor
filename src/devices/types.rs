use super::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceConfig {
    pub id: String,
    pub name: String,
    pub host: String,
    pub serial: String,
    pub access_code: String,
    pub mqtt_port: u16,
    pub mqtt_use_tls: bool,
    pub model: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SafeDeviceConfig {
    pub id: String,
    pub name: String,
    pub host: String,
    pub serial: String,
    pub mqtt_port: u16,
    pub mqtt_use_tls: bool,
    pub model: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCamera {
    pub supported: bool,
    pub relay_url: Option<String>,
    pub port: Option<u16>,
    pub protocol: Option<String>,
    pub path: Option<String>,
    pub mode: String,
    pub stream_name: Option<String>,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCreateInput {
    pub name: String,
    pub host: String,
    pub serial: String,
    pub access_code: String,
    pub mqtt_port: Option<u16>,
    pub mqtt_use_tls: Option<bool>,
    pub model: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceUpdateInput {
    pub name: Option<String>,
    pub host: Option<String>,
    pub serial: Option<String>,
    pub access_code: Option<String>,
    pub mqtt_port: Option<u16>,
    pub mqtt_use_tls: Option<bool>,
    pub model: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSnapshot {
    pub config: SafeDeviceConfig,
    pub connection: DeviceConnection,
    pub camera: DeviceCamera,
    pub last_seen_at: Option<String>,
    pub error: Option<String>,
    pub telemetry: Option<DeviceTelemetry>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceConnection {
    Offline,
    Connecting,
    Online,
    Error,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceTelemetry {
    pub state: String,
    pub progress: Option<f64>,
    pub task_name: Option<String>,
    pub layer_current: Option<f64>,
    pub layer_total: Option<f64>,
    pub nozzle_temp: Option<f64>,
    pub nozzle_target: Option<f64>,
    pub bed_temp: Option<f64>,
    pub bed_target: Option<f64>,
    pub chamber_temp: Option<f64>,
    pub ams_humidity: Option<f64>,
    pub ams: Option<AmsTelemetry>,
    pub speed_level: Option<f64>,
    pub fan_speed: Option<f64>,
    pub auxiliary_fan_speed: Option<f64>,
    pub chamber_fan_speed: Option<f64>,
    pub heatbreak_fan_speed: Option<f64>,
    pub wifi_signal: Option<String>,
    pub remaining_minutes: Option<f64>,
    pub raw_updated_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AmsTelemetry {
    pub active_tray: Option<String>,
    pub target_tray: Option<String>,
    pub humidity: Option<f64>,
    pub tray_count: usize,
    pub trays: Vec<AmsTray>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AmsTray {
    pub id: String,
    pub material: Option<String>,
    pub color: Option<String>,
    pub remaining: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum DeviceEvent {
    Snapshot { device: Box<DeviceSnapshot> },
    Devices { devices: Vec<DeviceSnapshot> },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCommandInput {
    pub command: DeviceCommand,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceCommand {
    Pause,
    Resume,
    Stop,
    Refresh,
    Connect,
    Disconnect,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProbeInput {
    pub host: String,
    #[serde(rename = "mqttPort")]
    pub mqtt_port: Option<u16>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeResult {
    pub ok: bool,
    pub host: String,
    pub port: u16,
    pub error: Option<String>,
    pub latency_ms: Option<u128>,
}
