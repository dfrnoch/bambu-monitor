use super::*;
use crate::devices::routes::DEVICE_MANAGER;

const CAMERA_PATH: &str = "/streaming/live/1";
const DEFAULT_GO2RTC_URL: &str = "http://127.0.0.1:1984";
static GO2RTC_STARTED: AtomicBool = AtomicBool::new(false);
pub(super) async fn camera_device(
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
pub(super) fn infer_camera(config: &DeviceConfig) -> DeviceCamera {
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
