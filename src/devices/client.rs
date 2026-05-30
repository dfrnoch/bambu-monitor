use super::*;

const MQTT_MAX_PACKET_SIZE_BYTES: usize = 1024 * 1024;
pub(super) struct BambuDeviceClient {
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
pub(super) struct ClientState {
    pub(super) connection: DeviceConnection,
    pub(super) error: Option<String>,
    pub(super) last_seen_at: Option<String>,
    pub(super) telemetry: Option<DeviceTelemetry>,
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
    pub(super) fn new(config: DeviceConfig, events: broadcast::Sender<DeviceEvent>) -> Self {
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

    pub(super) async fn snapshot(&self) -> DeviceSnapshot {
        let state = self.state.read().await.clone();
        snapshot_from_state(&self.config, state)
    }

    pub(super) async fn connect(&self) {
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

    pub(super) async fn disconnect(&self) {
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

    pub(super) async fn send(&self, command: DeviceCommand) {
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
