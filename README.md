# Bambu Monitor Rust

Single-binary Rust setup for a local Bambu LAN monitor.

The intended shape is one deployable app:

- Axum serves HTTP, SSE, static files, and Leptos.
- Leptos owns the browser UI.
- Device/MQTT/config logic lives in `src/devices`.
- No separate API project.

## Run

```bash
cargo run
```

Open `http://localhost:3000`.

## Current Features

- Local JSON printer storage in `data/devices.json`.
- Add, delete, probe, connect, disconnect, refresh, pause, resume, and stop flows.
- Per-printer MQTT workers for Bambu LAN mode reports and commands.
- TLS MQTT support with printer-local certificate verification disabled to match LAN-mode behavior.
- Telemetry parsing for print state, progress, layers, temperatures, fans, Wi-Fi, and AMS trays.
- Server-Sent Events live updates.
- Built-in responsive dashboard served by the Rust binary.
- Camera route metadata and go2rtc WebRTC iframe endpoint.

## API Surface

- `GET /health`
- `GET /devices` and `GET /devices/`
- `POST /devices` and `POST /devices/`
- `PUT /devices/{id}`
- `DELETE /devices/{id}`
- `POST /devices/{id}/command`
- `GET /devices/events`
- `POST /devices/probe`

Device config is stored in `data/devices.json` by default. Access codes are persisted locally but are not returned by device snapshot responses.
