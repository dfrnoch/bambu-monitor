use axum::{
    Json, Router,
    extract::Path,
    http::{HeaderMap, StatusCode, header},
    response::{
        Html, IntoResponse,
        sse::{Event as SseEvent, KeepAlive, Sse},
    },
    routing::{get, post, put},
};
use chrono::SecondsFormat;
use leptos::prelude::LeptosOptions;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS, Transport};
use rustls::{
    ClientConfig, DigitallySignedStruct, Error as TlsError, SignatureScheme,
    client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    pki_types::{CertificateDer, ServerName, UnixTime},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    convert::Infallible,
    env,
    path::PathBuf,
    process::{Command, Stdio},
    sync::{
        Arc, LazyLock,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{RwLock, broadcast, mpsc},
    task::JoinHandle,
    time::{Instant, timeout},
};
use tokio_stream::{StreamExt, wrappers::BroadcastStream};
use uuid::Uuid;

static DEVICE_MANAGER: LazyLock<Arc<DeviceManager>> =
    LazyLock::new(|| Arc::new(DeviceManager::new()));

const MQTT_MAX_PACKET_SIZE_BYTES: usize = 1024 * 1024;
const CAMERA_PATH: &str = "/streaming/live/1";
const DEFAULT_GO2RTC_URL: &str = "http://127.0.0.1:1984";
static GO2RTC_STARTED: AtomicBool = AtomicBool::new(false);

pub fn router() -> Router<LeptosOptions> {
    Router::new()
        .route("/devices", get(list_devices).post(create_device))
        .route("/devices/", get(list_devices).post(create_device))
        .route("/devices/events", get(events))
        .route("/devices/probe", post(probe_device))
        .route("/devices/{id}", put(update_device).delete(delete_device))
        .route("/devices/{id}/camera", get(camera_device))
        .route("/devices/{id}/command", post(command_device))
}

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

pub struct DeviceManager {
    devices: RwLock<HashMap<String, DeviceConfig>>,
    clients: RwLock<HashMap<String, Arc<BambuDeviceClient>>>,
    loaded: AtomicBool,
    events: broadcast::Sender<DeviceEvent>,
    devices_path: PathBuf,
}

impl DeviceManager {
    pub fn new() -> Self {
        let (events, _) = broadcast::channel(128);
        let data_dir = env::var("DATA_DIR").unwrap_or_else(|_| "data".to_string());

        Self {
            devices: RwLock::new(HashMap::new()),
            clients: RwLock::new(HashMap::new()),
            loaded: AtomicBool::new(false),
            events,
            devices_path: PathBuf::from(data_dir).join("devices.json"),
        }
    }

    async fn ensure_loaded(&self) -> Result<(), AppError> {
        if self.loaded.load(Ordering::SeqCst) {
            return Ok(());
        }

        let text = match tokio::fs::read_to_string(&self.devices_path).await {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                self.persist(&[]).await?;
                "[]".to_string()
            }
            Err(error) => return Err(error.into()),
        };

        let devices: Vec<DeviceConfig> = serde_json::from_str(&text)?;
        let auto_connect =
            env::var("AUTO_CONNECT").unwrap_or_else(|_| "true".to_string()) != "false";

        {
            let mut device_guard = self.devices.write().await;
            let mut client_guard = self.clients.write().await;

            for config in devices.into_iter().map(normalize_stored_device) {
                let client = self.create_client(config.clone());
                if auto_connect {
                    client.connect().await;
                }
                client_guard.insert(config.id.clone(), client);
                device_guard.insert(config.id.clone(), config);
            }
        }

        self.loaded.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn list(&self) -> Result<Vec<DeviceSnapshot>, AppError> {
        self.ensure_loaded().await?;
        Ok(self.client_snapshots().await)
    }

    async fn create(&self, input: DeviceCreateInput) -> Result<DeviceSnapshot, AppError> {
        self.ensure_loaded().await?;
        let config = normalize_new_device(input);
        let client = self.create_client(config.clone());

        {
            let mut device_guard = self.devices.write().await;
            device_guard.insert(config.id.clone(), config.clone());
            self.persist_locked(&device_guard).await?;
        }

        self.clients
            .write()
            .await
            .insert(config.id.clone(), client.clone());
        client.connect().await;
        self.emit_devices().await;
        Ok(client.snapshot().await)
    }

    async fn update(
        &self,
        id: &str,
        input: DeviceUpdateInput,
    ) -> Result<Option<DeviceSnapshot>, AppError> {
        self.ensure_loaded().await?;
        let config = {
            let mut guard = self.devices.write().await;
            let Some(current) = guard.get_mut(id) else {
                return Ok(None);
            };

            if let Some(name) = non_empty(input.name) {
                current.name = name;
            }
            if let Some(host) = non_empty(input.host) {
                current.host = sanitize_host(&host);
            }
            if let Some(serial) = non_empty(input.serial) {
                current.serial = serial;
            }
            if let Some(access_code) = non_empty(input.access_code) {
                current.access_code = access_code;
            }
            if let Some(port) = input.mqtt_port {
                current.mqtt_port = parse_port(port);
            }
            if let Some(use_tls) = input.mqtt_use_tls {
                current.mqtt_use_tls = use_tls;
            }
            if input.model.is_some() {
                current.model = input.model.and_then(|value| non_empty(Some(value)));
            }

            let config = current.clone();
            self.persist_locked(&guard).await?;
            config
        };

        let client = self.create_client(config.clone());
        if let Some(old) = self
            .clients
            .write()
            .await
            .insert(id.to_string(), client.clone())
        {
            old.disconnect().await;
        }
        client.connect().await;
        self.emit_devices().await;
        Ok(Some(client.snapshot().await))
    }

    async fn delete(&self, id: &str) -> Result<bool, AppError> {
        self.ensure_loaded().await?;
        if let Some(client) = self.clients.write().await.remove(id) {
            client.disconnect().await;
        }

        let deleted = {
            let mut guard = self.devices.write().await;
            let deleted = guard.remove(id).is_some();
            self.persist_locked(&guard).await?;
            deleted
        };

        self.emit_devices().await;
        Ok(deleted)
    }

    async fn command(
        &self,
        id: &str,
        command: DeviceCommand,
    ) -> Result<Option<DeviceSnapshot>, AppError> {
        self.ensure_loaded().await?;
        let Some(client) = self.clients.read().await.get(id).cloned() else {
            return Ok(None);
        };

        client.send(command).await;
        Ok(Some(client.snapshot().await))
    }

    async fn config(&self, id: &str) -> Result<Option<DeviceConfig>, AppError> {
        self.ensure_loaded().await?;
        Ok(self.devices.read().await.get(id).cloned())
    }

    fn subscribe(&self) -> broadcast::Receiver<DeviceEvent> {
        self.events.subscribe()
    }

    fn create_client(&self, config: DeviceConfig) -> Arc<BambuDeviceClient> {
        let events = self.events.clone();
        Arc::new(BambuDeviceClient::new(config, events))
    }

    async fn client_snapshots(&self) -> Vec<DeviceSnapshot> {
        let clients = self
            .clients
            .read()
            .await
            .values()
            .cloned()
            .collect::<Vec<_>>();
        let mut snapshots = Vec::with_capacity(clients.len());

        for client in clients {
            snapshots.push(client.snapshot().await);
        }

        snapshots
    }

    async fn emit_devices(&self) {
        let devices = self.client_snapshots().await;
        let _ = self.events.send(DeviceEvent::Devices { devices });
    }

    async fn persist_locked(
        &self,
        devices: &HashMap<String, DeviceConfig>,
    ) -> Result<(), AppError> {
        let devices = devices.values().cloned().collect::<Vec<_>>();
        self.persist(&devices).await
    }

    async fn persist(&self, devices: &[DeviceConfig]) -> Result<(), AppError> {
        if let Some(parent) = self.devices_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let text = serde_json::to_string_pretty(devices)?;
        tokio::fs::write(&self.devices_path, text).await?;
        Ok(())
    }
}

struct BambuDeviceClient {
    config: DeviceConfig,
    state: Arc<RwLock<ClientState>>,
    events: broadcast::Sender<DeviceEvent>,
    worker: RwLock<Option<ClientWorker>>,
}

struct ClientWorker {
    command_tx: mpsc::Sender<ClientCommand>,
    task: JoinHandle<()>,
}

#[derive(Clone, Debug)]
struct ClientState {
    connection: DeviceConnection,
    error: Option<String>,
    last_seen_at: Option<String>,
    telemetry: Option<DeviceTelemetry>,
}

enum ClientCommand {
    Publish(Value),
    Stop,
}

#[derive(Debug)]
struct NoCertificateVerification;

impl ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, TlsError> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, TlsError> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, TlsError> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::ED25519,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
        ]
    }
}

fn insecure_tls_config() -> ClientConfig {
    ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(NoCertificateVerification))
        .with_no_client_auth()
}

impl BambuDeviceClient {
    fn new(config: DeviceConfig, events: broadcast::Sender<DeviceEvent>) -> Self {
        let state = Arc::new(RwLock::new(ClientState {
            connection: DeviceConnection::Offline,
            error: None,
            last_seen_at: None,
            telemetry: None,
        }));

        Self {
            config,
            state,
            events,
            worker: RwLock::new(None),
        }
    }

    async fn snapshot(&self) -> DeviceSnapshot {
        let state = self.state.read().await.clone();
        snapshot_from_state(&self.config, state)
    }

    async fn connect(&self) {
        let mut worker = self.worker.write().await;

        if let Some(worker) = worker.as_ref() {
            let _ = worker
                .command_tx
                .send(ClientCommand::Publish(push_all_payload()))
                .await;
            return;
        }

        let (command_tx, command_rx) = mpsc::channel(32);
        let task_state = self.state.clone();
        let task_config = self.config.clone();
        let task_events = self.events.clone();
        let task = tokio::spawn(async move {
            mqtt_task(task_config, task_state, command_rx, task_events).await;
        });
        *worker = Some(ClientWorker { command_tx, task });
    }

    async fn disconnect(&self) {
        if let Some(worker) = self.worker.write().await.take() {
            let _ = worker.command_tx.send(ClientCommand::Stop).await;
            worker.task.abort();
        }

        {
            let mut state = self.state.write().await;
            state.connection = DeviceConnection::Offline;
            emit_snapshot(&self.config, &state, &self.events);
        }
    }

    async fn send(&self, command: DeviceCommand) {
        match command {
            DeviceCommand::Connect | DeviceCommand::Refresh => self.connect().await,
            DeviceCommand::Disconnect => self.disconnect().await,
            DeviceCommand::Pause => self.publish_print_command("pause").await,
            DeviceCommand::Resume => self.publish_print_command("resume").await,
            DeviceCommand::Stop => self.publish_print_command("stop").await,
        }
    }

    async fn publish_print_command(&self, command: &str) {
        let payload = json!({
            "print": {
                "command": command,
                "sequence_id": now_millis().to_string(),
            }
        });
        let worker = self.worker.read().await;

        if let Some(worker) = worker.as_ref() {
            let _ = worker
                .command_tx
                .send(ClientCommand::Publish(payload))
                .await;
        }
    }
}

async fn mqtt_task(
    config: DeviceConfig,
    state: Arc<RwLock<ClientState>>,
    mut commands: mpsc::Receiver<ClientCommand>,
    events: broadcast::Sender<DeviceEvent>,
) {
    set_state(
        &config,
        &state,
        &events,
        DeviceConnection::Connecting,
        None,
        None,
    )
    .await;

    let mut options = MqttOptions::new(
        format!("bambu-monitor-{}", config.id),
        config.host.clone(),
        config.mqtt_port,
    );
    options.set_credentials("bblp", config.access_code.clone());
    options.set_keep_alive(Duration::from_secs(30));
    options.set_clean_session(true);
    options.set_max_packet_size(MQTT_MAX_PACKET_SIZE_BYTES, MQTT_MAX_PACKET_SIZE_BYTES);
    if config.mqtt_use_tls {
        options.set_transport(Transport::tls_with_config(insecure_tls_config().into()));
    }

    let (client, mut event_loop) = AsyncClient::new(options, 16);
    let request_topic = format!("device/{}/request", config.serial);
    let report_topic = format!("device/{}/report", config.serial);
    let mut connected = false;
    let mut pending = vec![push_all_payload()];

    loop {
        tokio::select! {
            command = commands.recv() => {
                match command {
                    Some(ClientCommand::Publish(payload)) => {
                        if connected {
                            publish_json(&client, &request_topic, payload).await;
                        } else {
                            pending.push(payload);
                        }
                    }
                    Some(ClientCommand::Stop) | None => {
                        let _ = client.disconnect().await;
                        set_state(&config, &state, &events, DeviceConnection::Offline, None, None).await;
                        break;
                    }
                }
            }
            event = event_loop.poll() => {
                match event {
                    Ok(Event::Incoming(Packet::ConnAck(_))) => {
                        connected = true;
                        let _ = client.subscribe(report_topic.clone(), QoS::AtMostOnce).await;
                        set_state(&config, &state, &events, DeviceConnection::Online, None, Some(now_iso())).await;

                        for payload in pending.drain(..) {
                            publish_json(&client, &request_topic, payload).await;
                        }
                    }
                    Ok(Event::Incoming(Packet::Publish(message))) => {
                        match serde_json::from_slice::<Value>(&message.payload) {
                            Ok(payload) => {
                                if let Some(telemetry) = parse_telemetry(&payload) {
                                    let mut guard = state.write().await;
                                    guard.connection = DeviceConnection::Online;
                                    guard.error = None;
                                    guard.last_seen_at = Some(now_iso());
                                    guard.telemetry = Some(telemetry);
                                    emit_snapshot(&config, &guard, &events);
                                }
                            }
                            Err(error) => {
                                set_state(&config, &state, &events, DeviceConnection::Error, Some(error.to_string()), None).await;
                            }
                        }
                    }
                    Ok(_) => {}
                    Err(error) => {
                        connected = false;
                        set_state(&config, &state, &events, DeviceConnection::Error, Some(connection_error_message(&config.host, config.mqtt_port, &error.to_string())), None).await;
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        set_state(&config, &state, &events, DeviceConnection::Connecting, None, None).await;
                    }
                }
            }
        }
    }
}

async fn publish_json(client: &AsyncClient, topic: &str, payload: Value) {
    if let Ok(bytes) = serde_json::to_vec(&payload) {
        let _ = client.publish(topic, QoS::AtMostOnce, false, bytes).await;
    }
}

async fn set_state(
    config: &DeviceConfig,
    state: &RwLock<ClientState>,
    events: &broadcast::Sender<DeviceEvent>,
    connection: DeviceConnection,
    error: Option<String>,
    last_seen_at: Option<String>,
) {
    let mut guard = state.write().await;
    guard.connection = connection;
    guard.error = error;
    if last_seen_at.is_some() {
        guard.last_seen_at = last_seen_at;
    }
    emit_snapshot(config, &guard, events);
}

fn emit_snapshot(
    config: &DeviceConfig,
    state: &ClientState,
    events: &broadcast::Sender<DeviceEvent>,
) {
    let _ = events.send(DeviceEvent::Snapshot {
        device: Box::new(snapshot_from_state(config, state.clone())),
    });
}

fn push_all_payload() -> Value {
    json!({
        "pushing": {
            "command": "pushall",
            "sequence_id": now_millis().to_string(),
            "version": 1,
            "push_target": 1,
        }
    })
}

async fn list_devices() -> Result<Json<Vec<DeviceSnapshot>>, AppError> {
    Ok(Json(DEVICE_MANAGER.list().await?))
}

async fn create_device(
    Json(input): Json<DeviceCreateInput>,
) -> Result<Json<DeviceSnapshot>, AppError> {
    Ok(Json(DEVICE_MANAGER.create(input).await?))
}

async fn update_device(
    Path(id): Path<String>,
    Json(input): Json<DeviceUpdateInput>,
) -> Result<ResponseJsonOrStatus<DeviceSnapshot>, AppError> {
    match DEVICE_MANAGER.update(&id, input).await? {
        Some(device) => Ok(ResponseJsonOrStatus::Json(device)),
        None => Ok(ResponseJsonOrStatus::Status(StatusCode::NOT_FOUND)),
    }
}

async fn delete_device(Path(id): Path<String>) -> Result<StatusCode, AppError> {
    if DEVICE_MANAGER.delete(&id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Ok(StatusCode::NOT_FOUND)
    }
}

async fn command_device(
    Path(id): Path<String>,
    Json(input): Json<DeviceCommandInput>,
) -> Result<ResponseJsonOrStatus<DeviceSnapshot>, AppError> {
    match DEVICE_MANAGER.command(&id, input.command).await? {
        Some(device) => Ok(ResponseJsonOrStatus::Json(device)),
        None => Ok(ResponseJsonOrStatus::Status(StatusCode::NOT_FOUND)),
    }
}

async fn probe_device(Json(input): Json<ProbeInput>) -> Json<ProbeResult> {
    let host = sanitize_host(&input.host);
    let port = input.mqtt_port.map(parse_port).unwrap_or(8883);
    let started_at = Instant::now();

    match timeout(
        Duration::from_secs(4),
        TcpStream::connect((host.as_str(), port)),
    )
    .await
    {
        Ok(Ok(stream)) => {
            drop(stream);
            Json(ProbeResult {
                ok: true,
                host,
                port,
                error: None,
                latency_ms: Some(started_at.elapsed().as_millis()),
            })
        }
        Ok(Err(error)) => {
            let error = connection_error_message(&host, port, &error.to_string());
            Json(ProbeResult {
                ok: false,
                host,
                port,
                error: Some(error),
                latency_ms: None,
            })
        }
        Err(_) => Json(ProbeResult {
            ok: false,
            host: host.clone(),
            port,
            error: Some(format!("Timed out connecting to {host}:{port}")),
            latency_ms: None,
        }),
    }
}

async fn camera_device(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let Some(config) = DEVICE_MANAGER.config(&id).await? else {
        return Ok((StatusCode::NOT_FOUND, "Printer not found.").into_response());
    };

    if let Some(error) = ensure_go2rtc_stream(&config).await {
        return Ok(html_response(
            camera_error_html(&error.message),
            error.status,
        ));
    }

    let go2rtc_url = go2rtc_public_url(&headers);
    let stream_name = percent_encode(&stream_name(&config.id));
    let player_url = format!("{go2rtc_url}/webrtc.html?src={stream_name}&media=video");
    let html = format!(
        r#"<!doctype html>
<html>
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<style>html,body,iframe{{background:#18181b;border:0;height:100%;margin:0;overflow:hidden;width:100%;}}</style>
</head>
<body><iframe allow="autoplay; fullscreen; microphone; camera" src="{}" title="Bambu camera WebRTC player"></iframe></body>
</html>"#,
        escape_html(&player_url)
    );

    Ok(html_response(html, StatusCode::OK))
}

struct CameraRouteError {
    status: StatusCode,
    message: String,
}

async fn ensure_go2rtc_stream(config: &DeviceConfig) -> Option<CameraRouteError> {
    if !ensure_go2rtc_available().await {
        return Some(CameraRouteError {
            status: StatusCode::SERVICE_UNAVAILABLE,
            message: "WebRTC camera needs go2rtc running on the API host. Install go2rtc or set GO2RTC_URL, then reload the camera.".to_string(),
        });
    }

    let name = stream_name(&config.id);
    let mut rejected_status = None;

    for port in camera_port_candidates(config) {
        let path = format!(
            "/api/streams?name={}&src={}",
            percent_encode(&name),
            percent_encode(&camera_stream_url(config, port))
        );

        match go2rtc_request("PUT", &path, Duration::from_secs(2)).await {
            Ok(response) if response.status.is_success() => return None,
            Ok(response) => rejected_status = Some(response.status),
            Err(_) => {
                return Some(CameraRouteError {
                    status: StatusCode::SERVICE_UNAVAILABLE,
                    message: "WebRTC camera needs go2rtc running on the API host. Start go2rtc on port 1984 and reload the camera.".to_string(),
                });
            }
        }
    }

    Some(CameraRouteError {
        status: StatusCode::BAD_GATEWAY,
        message: rejected_status
            .map(|status| format!("go2rtc rejected the printer stream ({}).", status.as_u16()))
            .unwrap_or_else(|| {
                "go2rtc could not register the camera stream. Check that LAN liveview is enabled and that the access code is correct.".to_string()
            }),
    })
}

async fn ensure_go2rtc_available() -> bool {
    if can_reach_go2rtc().await {
        return true;
    }

    if !GO2RTC_STARTED.swap(true, Ordering::SeqCst) {
        let binary = env::var("GO2RTC_BIN").unwrap_or_else(|_| "go2rtc".to_string());
        if Command::new(binary)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .is_err()
        {
            GO2RTC_STARTED.store(false, Ordering::SeqCst);
        }
    }

    for _ in 0..10 {
        tokio::time::sleep(Duration::from_millis(250)).await;
        if can_reach_go2rtc().await {
            return true;
        }
    }

    false
}

async fn can_reach_go2rtc() -> bool {
    matches!(
        go2rtc_request("GET", "/api/streams", Duration::from_secs(2)).await,
        Ok(response) if response.status.is_success()
    )
}

struct Go2rtcResponse {
    status: StatusCode,
}

async fn go2rtc_request(
    method: &str,
    path: &str,
    request_timeout: Duration,
) -> Result<Go2rtcResponse, ()> {
    let endpoint = parse_go2rtc_url(&go2rtc_url())?;
    let mut stream = timeout(
        request_timeout,
        TcpStream::connect((endpoint.host.as_str(), endpoint.port)),
    )
    .await
    .map_err(|_| ())?
    .map_err(|_| ())?;

    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nContent-Length: 0\r\n\r\n",
        endpoint.host_header
    );
    timeout(request_timeout, stream.write_all(request.as_bytes()))
        .await
        .map_err(|_| ())?
        .map_err(|_| ())?;

    let mut response = Vec::new();
    timeout(request_timeout, stream.read_to_end(&mut response))
        .await
        .map_err(|_| ())?
        .map_err(|_| ())?;
    let header_end = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|index| index + 4)
        .unwrap_or(response.len());
    let head = std::str::from_utf8(&response[..header_end]).map_err(|_| ())?;
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .and_then(|code| StatusCode::from_u16(code).ok())
        .ok_or(())?;

    Ok(Go2rtcResponse { status })
}

struct Go2rtcEndpoint {
    host: String,
    host_header: String,
    port: u16,
}

fn parse_go2rtc_url(url: &str) -> Result<Go2rtcEndpoint, ()> {
    let rest = url
        .trim_end_matches('/')
        .strip_prefix("http://")
        .ok_or(())?;
    let host_port = rest.split('/').next().ok_or(())?;
    let (host, port) = match host_port.rsplit_once(':') {
        Some((host, port)) => (host, port.parse::<u16>().map_err(|_| ())?),
        None => (host_port, 80),
    };

    Ok(Go2rtcEndpoint {
        host: host.to_string(),
        host_header: host_port.to_string(),
        port,
    })
}

fn camera_stream_url(config: &DeviceConfig, port: u16) -> String {
    format!(
        "rtsps://bblp:{}@{}:{}{}",
        percent_encode(&config.access_code),
        config.host,
        port,
        CAMERA_PATH
    )
}

fn camera_port_candidates(config: &DeviceConfig) -> Vec<u16> {
    let preferred = camera_port(config);
    let fallback = if preferred == 322 { 6000 } else { 322 };
    vec![preferred, fallback]
}

fn go2rtc_url() -> String {
    env::var("GO2RTC_URL")
        .unwrap_or_else(|_| DEFAULT_GO2RTC_URL.to_string())
        .trim_end_matches('/')
        .to_string()
}

fn go2rtc_public_url(headers: &HeaderMap) -> String {
    if let Ok(url) = env::var("GO2RTC_PUBLIC_URL") {
        let url = url.trim().trim_end_matches('/');
        if !url.is_empty() {
            return url.to_string();
        }
    }

    let Some(host) = request_hostname(headers) else {
        return go2rtc_url();
    };
    let scheme = headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("http");
    let port = env::var("GO2RTC_PUBLIC_PORT")
        .or_else(|_| env::var("GO2RTC_PORT"))
        .unwrap_or_else(|_| "1984".to_string());

    format!("{scheme}://{host}:{port}")
}

fn request_hostname(headers: &HeaderMap) -> Option<String> {
    let host = headers
        .get("x-forwarded-host")
        .or_else(|| headers.get(header::HOST))?
        .to_str()
        .ok()?
        .split(',')
        .next()?
        .trim();

    if host.is_empty() {
        return None;
    }

    if host.starts_with('[') {
        return host.split(']').next().map(|value| format!("{value}]"));
    }

    Some(host.split(':').next().unwrap_or(host).to_string())
}

fn html_response(html: String, status: StatusCode) -> axum::response::Response {
    (
        status,
        [
            (header::CACHE_CONTROL, "no-store"),
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
        ],
        Html(html),
    )
        .into_response()
}

fn camera_error_html(message: &str) -> String {
    format!(
        r#"<!doctype html>
<html>
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<style>body{{align-items:center;background:#18181b;color:#d4d4d8;display:flex;font:14px system-ui,sans-serif;height:100vh;justify-content:center;margin:0;padding:24px;text-align:center;}}</style>
</head>
<body>{}</body>
</html>"#,
        escape_html(message)
    )
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

async fn events()
-> Result<Sse<impl futures_core::Stream<Item = Result<SseEvent, Infallible>>>, AppError> {
    let devices = DEVICE_MANAGER.list().await?;
    let initial = DeviceEvent::Devices { devices };
    let stream = BroadcastStream::new(DEVICE_MANAGER.subscribe()).filter_map(|event| match event {
        Ok(event) => Some(Ok(to_sse_event(event))),
        Err(_) => None,
    });
    let initial = async_stream::stream! {
        yield Ok(to_sse_event(initial));
    };
    let stream = initial.chain(stream);

    Ok(Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(25))))
}

fn to_sse_event(event: DeviceEvent) -> SseEvent {
    SseEvent::default().data(serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string()))
}

fn normalize_new_device(input: DeviceCreateInput) -> DeviceConfig {
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

fn normalize_stored_device(input: DeviceConfig) -> DeviceConfig {
    DeviceConfig {
        host: sanitize_host(&input.host),
        mqtt_port: parse_port(input.mqtt_port),
        mqtt_use_tls: input.mqtt_use_tls,
        ..input
    }
}

fn snapshot_from_state(config: &DeviceConfig, state: ClientState) -> DeviceSnapshot {
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

fn infer_camera(config: &DeviceConfig) -> DeviceCamera {
    DeviceCamera {
        supported: true,
        relay_url: Some(format!("/devices/{}/camera", config.id)),
        port: Some(camera_port(config)),
        protocol: Some("rtsps".to_string()),
        path: Some(CAMERA_PATH.to_string()),
        mode: "webrtc".to_string(),
        stream_name: Some(stream_name(&config.id)),
        message: "Camera is detected automatically and streamed through WebRTC.".to_string(),
    }
}

fn camera_port(config: &DeviceConfig) -> u16 {
    let model = config.model.as_deref().unwrap_or("").to_lowercase();
    if model.contains("p1") || model.contains("p2") || model.contains("a1") {
        6000
    } else {
        322
    }
}

fn stream_name(id: &str) -> String {
    format!("bambu_{}", id.replace('-', "_"))
}

fn parse_telemetry(payload: &Value) -> Option<DeviceTelemetry> {
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

fn parse_port(port: u16) -> u16 {
    if port == 0 { 8883 } else { port }
}

fn connection_error_message(host: &str, port: u16, error: &str) -> String {
    let lower = error.to_lowercase();
    let target = format!("{host}:{port}");

    if lower.contains("no route to host") || lower.contains("os error 65") {
        return format!(
            "Cannot reach {target}: your computer has no route to that address. Check the printer IP, make sure this app is on the same Wi-Fi/VLAN as the printer, and disconnect VPNs or firewall rules that block local LAN traffic."
        );
    }

    if lower.contains("connection refused") {
        return format!(
            "Reached {target}, but the MQTT port refused the connection. Enable LAN mode on the printer and confirm MQTT port {port} is correct."
        );
    }

    if lower.contains("timed out") || lower.contains("timeout") {
        return format!(
            "Timed out connecting to {target}. Check that the printer is powered on, awake, on the same network, and reachable on MQTT port {port}."
        );
    }

    if lower.contains("nodename nor servname")
        || lower.contains("name or service not known")
        || lower.contains("failed to lookup address")
    {
        return format!(
            "Could not resolve {host}. Enter the printer IP address, for example 192.168.1.25, instead of a hostname."
        );
    }

    if lower.contains("not authorized")
        || lower.contains("bad user name or password")
        || lower.contains("authentication")
    {
        return "MQTT authentication failed. Check the printer access code from LAN mode settings."
            .to_string();
    }

    if lower.contains("tls") || lower.contains("certificate") {
        return format!(
            "Opened {target}, but the TLS MQTT handshake failed. Bambu LAN MQTT usually uses TLS on port 8883; try the Test button again and confirm LAN mode is enabled."
        );
    }

    format!("Could not connect to {target}: {error}")
}

fn sanitize_host(value: &str) -> String {
    let trimmed = value.trim();

    if let Some(rest) = trimmed.split_once("://").map(|(_, rest)| rest) {
        return rest.split(['/', ':']).next().unwrap_or(rest).to_string();
    }

    trimmed.split(':').next().unwrap_or(trimmed).to_string()
}

fn non_empty(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

enum ResponseJsonOrStatus<T> {
    Json(T),
    Status(StatusCode),
}

impl<T> IntoResponse for ResponseJsonOrStatus<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Json(value) => Json(value).into_response(),
            Self::Status(status) => status.into_response(),
        }
    }
}

#[derive(Debug)]
enum AppError {
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let message = match self {
            Self::Io(error) => error.to_string(),
            Self::Json(error) => error.to_string(),
        };

        (StatusCode::INTERNAL_SERVER_ERROR, message).into_response()
    }
}
