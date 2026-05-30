use super::*;

pub(super) fn parse_port(port: u16) -> u16 {
    if port == 0 { 8883 } else { port }
}

pub(super) fn connection_error_message(host: &str, port: u16, error: &str) -> String {
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

pub(super) fn sanitize_host(value: &str) -> String {
    let trimmed = value.trim();

    if let Some(rest) = trimmed.split_once("://").map(|(_, rest)| rest) {
        return rest.split(['/', ':']).next().unwrap_or(rest).to_string();
    }

    trimmed.split(':').next().unwrap_or(trimmed).to_string()
}

pub(super) fn non_empty(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(super) fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

pub(super) fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}
