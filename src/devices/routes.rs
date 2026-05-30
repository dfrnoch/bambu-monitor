use super::*;

pub(super) static DEVICE_MANAGER: LazyLock<Arc<DeviceManager>> =
    LazyLock::new(|| Arc::new(DeviceManager::new()));

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
