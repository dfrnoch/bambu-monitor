# Bambu LAN Monitor

A local-first web app for monitoring Bambu Lab printers in LAN / Developer mode. It is intended to run on a small always-on machine, such as a mini PC, and expose a responsive PWA dashboard for phones, tablets, and desktops.

The app is a Bun monorepo:

```txt
apps/
  api/       Elysia backend, device storage, LAN MQTT connections
  ui/        React + Vite frontend, PWA dashboard
packages/
  contracts/ Shared API and telemetry types
```

## Why Split Frontend And Backend?

Bambu LAN mode talks MQTT over TLS on the printer LAN address, usually port `8883`. Browsers cannot open that raw MQTT/TLS socket directly, and Cloudflare Tunnel should not expose printer credentials to the client. The backend owns all printer connections and stores the LAN access codes locally. The frontend only uses HTTP and Server-Sent Events.

This keeps everything in one codebase while preserving the right runtime boundary:

- React PWA for mobile installation and fast UI.
- Elysia API for LAN MQTT, persistence, and command dispatch.
- SSE stream for live updates through LAN or Cloudflare Tunnel.

## Current Features

- Add and remove printers by name, host/IP, serial, model, and LAN access code.
- Auto-connect to configured printers on backend startup.
- Live device stream over `/devices/events`.
- Dashboard cards with connection state, progress, task, layers, temperatures, Wi-Fi, and last-seen time.
- Commands for refresh, pause, resume, stop, connect, and disconnect.
- PWA manifest and generated service worker from `vite-plugin-pwa`.
- JSON config storage in `apps/api/data/devices.json` by default.

## Requirements

- Bun `>=1.3.5`
- Bambu printer with LAN / Developer mode enabled
- Printer IP/hostname, serial number, and LAN access code

## Local Development

```bash
bun install
bun dev
```

Services:

- API: `http://localhost:3000`
- UI: `http://localhost:5173`

Optional environment files:

`apps/api/.env`

```env
PORT=3000
FRONTEND_URL=http://localhost:5173
DATA_DIR=./data
AUTO_CONNECT=true
```

`apps/ui/.env`

```env
VITE_API_URL=http://localhost:3000
```

## Production On A Mini PC

Build both apps:

```bash
bun run --filter @bambu-monitor/api build
bun run --filter @bambu-monitor/ui build
```

Run the backend:

```bash
cd apps/api
bun start
```

Serve the UI build with any static server. For example:

```bash
cd apps/ui
bunx --bun serve -s dist -l tcp://0.0.0.0:5173
```

Set `VITE_API_URL` before building if the browser will reach the API at a different URL.

## Cloudflare Tunnel

Cloudflare Tunnel works well with this split because the public browser only talks to HTTP endpoints:

- Forward the UI host, for example `https://bambu.example.com`.
- Forward the API host, for example `https://bambu-api.example.com`.
- Build the UI with `VITE_API_URL=https://bambu-api.example.com`.
- Set API `FRONTEND_URL=https://bambu.example.com`.

Keep the backend on the same LAN as the printers. Do not expose printer MQTT ports directly.

## API Shape

- `GET /health`
- `GET /devices/`
- `POST /devices/`
- `PUT /devices/:id`
- `DELETE /devices/:id`
- `POST /devices/:id/command`
- `GET /devices/events`

Device credentials are never returned to the UI.

## Validation

```bash
bun run lint
bunx --bun tsc -b
bun run --filter @bambu-monitor/ui build
bun run --filter @bambu-monitor/api build
```
