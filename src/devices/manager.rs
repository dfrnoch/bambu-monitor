use super::*;

pub(super) struct DeviceManager {
    devices: RwLock<HashMap<String, DeviceConfig>>,
    clients: RwLock<HashMap<String, Arc<BambuDeviceClient>>>,
    loaded: AtomicBool,
    events: broadcast::Sender<DeviceEvent>,
    devices_path: PathBuf,
}

impl DeviceManager {
    pub(super) fn new() -> Self {
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

    pub(super) async fn list(&self) -> Result<Vec<DeviceSnapshot>, AppError> {
        self.ensure_loaded().await?;
        Ok(self.client_snapshots().await)
    }

    pub(super) async fn create(
        &self,
        input: DeviceCreateInput,
    ) -> Result<DeviceSnapshot, AppError> {
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

    pub(super) async fn update(
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

    pub(super) async fn delete(&self, id: &str) -> Result<bool, AppError> {
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

    pub(super) async fn command(
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

    pub(super) async fn config(&self, id: &str) -> Result<Option<DeviceConfig>, AppError> {
        self.ensure_loaded().await?;
        Ok(self.devices.read().await.get(id).cloned())
    }

    pub(super) fn subscribe(&self) -> broadcast::Receiver<DeviceEvent> {
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
