use leptos::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <main class="app-shell">
            <style>{APP_CSS}</style>

            <div class="toast-region" id="toast-region" aria-live="polite" aria-atomic="true"></div>

            <header class="topbar">
                <div class="printer-picker">
                    <label class="sr-only" for="printer-switcher">"Selected printer"</label>
                    <select id="printer-switcher" disabled>
                        <option value="">"No printers"</option>
                    </select>
                    <span class="select-icon icon" data-icon="chevron-down" aria-hidden="true"></span>
                </div>
                <button class="tool-button" id="refresh" type="button" aria-label="Refresh printers" title="Refresh printers">
                    <span class="icon" data-icon="activity" aria-hidden="true"></span>
                </button>
                <button class="tool-button" id="theme-toggle" type="button" aria-label="Toggle dark mode" title="Toggle dark mode">
                    <span class="icon" data-icon="moon" aria-hidden="true"></span>
                </button>
                <button class="tool-button primary" id="open-add" type="button" aria-label="Add printer" title="Add printer">
                    <span class="icon" data-icon="plus" aria-hidden="true"></span>
                </button>
            </header>

            <section class="content" id="content">
                <div class="empty-state" id="empty-state" hidden>
                    <div class="empty-icon" aria-hidden="true">"+"</div>
                    <h1>"Add your printer"</h1>
                    <p>"Connect a Bambu printer on your LAN to see print progress, camera, AMS, temperatures, and controls."</p>
                    <button class="primary-action" id="empty-add" type="button">"Add printer"</button>
                </div>
                <section id="device-view" class="device-view" hidden></section>
            </section>

            <nav class="control-dock" id="control-dock" hidden aria-label="Printer controls">
                <div class="dock-inner">
                    <button class="dock-button" data-dock-command="refresh" type="button">
                        <span class="icon" data-icon="refresh" aria-hidden="true"></span>
                        <span>"Refresh"</span>
                    </button>
                    <button class="dock-button" id="primary-command" data-dock-command="pause" type="button">
                        <span class="icon" id="primary-command-icon" data-icon="pause" aria-hidden="true"></span>
                        <span id="primary-command-label">"Pause"</span>
                    </button>
                    <button class="dock-button" data-dock-command="resume" type="button">
                        <span class="icon" data-icon="play" aria-hidden="true"></span>
                        <span>"Resume"</span>
                    </button>
                    <button class="dock-button danger" data-dock-command="stop" type="button">
                        <span class="icon" data-icon="square" aria-hidden="true"></span>
                        <span>"Stop"</span>
                    </button>
                </div>
            </nav>

            <div class="sheet-backdrop" id="add-sheet" hidden>
                <section class="sheet" role="dialog" aria-modal="true" aria-labelledby="add-title">
                    <div class="sheet-head">
                        <div>
                            <h2 id="add-title">"Add printer"</h2>
                            <p>"Camera and LAN settings are detected automatically."</p>
                        </div>
                        <button class="tool-button subtle" id="close-add" type="button" aria-label="Close add printer">
                            <span class="icon" data-icon="x" aria-hidden="true"></span>
                        </button>
                    </div>
                    <form class="add-form" id="add-form">
                        <label>"Name"<input name="name" required placeholder="Living room printer" /></label>
                        <label>"Printer type"
                            <select name="model">
                                <option>"A1 mini"</option>
                                <option>"A1"</option>
                                <option>"P1P"</option>
                                <option>"P1S"</option>
                                <option>"X1 Carbon"</option>
                                <option>"X1E"</option>
                                <option>"H2D"</option>
                                <option>"Other Bambu printer"</option>
                            </select>
                        </label>
                        <label>"Printer IP"<input name="host" required placeholder="192.168.10.13" inputmode="decimal" /></label>
                        <label>"Serial number"<input name="serial" required placeholder="01P00A..." autocapitalize="characters" /></label>
                        <label>"Access code"<input name="accessCode" required type="password" placeholder="LAN access code" /></label>
                        <p class="message" id="form-message" hidden></p>
                        <div class="form-actions">
                            <button class="secondary-action" id="probe" type="button">"Test"</button>
                            <button class="primary-action" type="submit">"Add printer"</button>
                        </div>
                    </form>
                </section>
            </div>

            <script inner_html=APP_JS></script>
        </main>
    }
}

const APP_CSS: &str = r#"
:root {
  color-scheme: light;
  --bg: #f4f4f5;
  --surface: #ffffff;
  --surface-muted: #f4f4f5;
  --hero: #111113;
  --hero-soft: #232326;
  --text: #18181b;
  --muted: #71717a;
  --faint: #a1a1aa;
  --line: #e4e4e7;
  --line-strong: #d4d4d8;
  --accent: #10b981;
  --accent-strong: #059669;
  --danger: #dc2626;
  --danger-bg: #fee2e2;
  --shadow: 0 1px 2px rgb(24 24 27 / .08);
  font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
}

:root[data-theme="dark"] {
  color-scheme: dark;
  --bg: #101113;
  --surface: #181a1d;
  --surface-muted: #222529;
  --hero: #050506;
  --hero-soft: #15171a;
  --text: #f4f4f5;
  --muted: #a1a1aa;
  --faint: #71717a;
  --line: #2d3035;
  --line-strong: #3f4248;
  --accent: #34d399;
  --accent-strong: #10b981;
  --danger: #f87171;
  --danger-bg: #3f1518;
  --shadow: 0 1px 2px rgb(0 0 0 / .28);
}

* { box-sizing: border-box; }
html { background: var(--bg); }
body { margin: 0; min-width: 320px; background: var(--bg); color: var(--text); }
button, input, select { font: inherit; }
button { cursor: pointer; }
button:disabled, select:disabled { cursor: not-allowed; opacity: .55; }
.sr-only { position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border: 0; }
.app-shell { width: min(768px, 100%); min-height: 100dvh; margin: 0 auto; padding: 12px 12px 120px; }
.topbar { position: sticky; top: 0; z-index: 20; display: grid; grid-template-columns: minmax(0, 1fr) 48px 48px 48px; gap: 8px; margin: 0 -12px 12px; padding: 12px; background: color-mix(in srgb, var(--bg) 90%, transparent); backdrop-filter: blur(18px); }
.printer-picker { position: relative; min-width: 0; }
.printer-picker select, .add-form input, .add-form select { width: 100%; min-height: 48px; border: 1px solid var(--line); border-radius: 16px; background: var(--surface); color: var(--text); outline: none; box-shadow: var(--shadow); }
.printer-picker select { appearance: none; padding: 0 42px 0 14px; font-size: 16px; font-weight: 800; overflow: hidden; text-overflow: ellipsis; }
.select-icon { position: absolute; right: 16px; top: 50%; transform: translateY(-50%); color: var(--muted); pointer-events: none; }
.tool-button { display: grid; place-items: center; width: 48px; height: 48px; border: 1px solid var(--line); border-radius: 16px; background: var(--surface); color: var(--text); box-shadow: var(--shadow); font-weight: 900; }
.icon { display: inline-grid; place-items: center; width: 20px; height: 20px; color: currentColor; }
.icon svg { width: 100%; height: 100%; stroke: currentColor; stroke-width: 2.2; stroke-linecap: round; stroke-linejoin: round; fill: none; }
.tool-button.primary, .primary-action { border: 0; background: var(--hero); color: white; }
:root[data-theme="dark"] .tool-button.primary, :root[data-theme="dark"] .primary-action { background: #f4f4f5; color: #18181b; }
.tool-button.subtle { background: var(--surface-muted); box-shadow: none; }
.content { display: grid; gap: 12px; }
.empty-state { min-height: calc(100dvh - 168px); display: grid; place-items: center; align-content: center; gap: 10px; padding: 28px; border: 1px solid var(--line); border-radius: 28px; background: var(--surface); text-align: center; box-shadow: var(--shadow); }
.empty-state[hidden], .device-view[hidden], .control-dock[hidden], .sheet-backdrop[hidden] { display: none; }
.empty-icon { display: grid; place-items: center; width: 64px; height: 64px; border-radius: 999px; background: color-mix(in srgb, var(--accent) 14%, transparent); color: var(--accent-strong); font-size: 32px; font-weight: 900; }
.empty-state h1 { margin: 0; font-size: 28px; line-height: 1.05; }
.empty-state p { max-width: 420px; margin: 0; color: var(--muted); line-height: 1.55; }
.device-view { display: grid; gap: 12px; }
.hero { overflow: hidden; border-radius: 28px; background: var(--hero); color: white; box-shadow: var(--shadow); }
.hero-head { display: flex; justify-content: space-between; align-items: flex-start; gap: 12px; padding: 16px 16px 12px; }
.meta { color: #a1a1aa; font-size: 12px; font-weight: 800; letter-spacing: .04em; text-transform: uppercase; }
.hero h1 { margin: 3px 0 0; font-size: clamp(26px, 7vw, 36px); line-height: 1.05; overflow-wrap: anywhere; letter-spacing: 0; }
.status { display: inline-flex; flex-shrink: 0; align-items: center; gap: 8px; border-radius: 999px; background: rgb(255 255 255 / .12); padding: 8px 11px; font-size: 12px; font-weight: 900; }
.dot { width: 8px; height: 8px; border-radius: 999px; background: #a1a1aa; }
.dot.online { background: #10b981; }
.dot.connecting { background: #f59e0b; }
.dot.error { background: #ef4444; }
.camera { position: relative; margin: 0 12px; aspect-ratio: 4 / 3; overflow: hidden; border-radius: 22px; background: #09090b; border: 1px solid rgb(255 255 255 / .1); }
.camera iframe { display: block; width: 100%; height: 100%; border: 0; }
.camera-empty { display: grid; height: 100%; place-items: center; padding: 24px; text-align: center; color: #a1a1aa; }
.camera-empty .icon { width: 36px; height: 36px; margin: 0 auto 12px; color: #d4d4d8; }
.camera-empty strong { display: block; margin-bottom: 6px; color: #d4d4d8; }
.camera-pill { position: absolute; left: 12px; top: 12px; border-radius: 999px; background: rgb(0 0 0 / .58); padding: 6px 10px; color: white; font-size: 12px; font-weight: 800; backdrop-filter: blur(10px); }
.hero-body { display: grid; gap: 14px; padding: 16px; }
.print-row { display: grid; grid-template-columns: minmax(0, 1fr) auto; gap: 16px; align-items: start; }
.task { margin: 4px 0 0; font-size: 22px; font-weight: 900; line-height: 1.12; overflow-wrap: anywhere; }
.progress-value { font-size: 42px; font-weight: 900; line-height: .95; font-variant-numeric: tabular-nums; }
.progress-label { color: #a1a1aa; font-size: 12px; font-weight: 700; text-align: right; }
.bar { height: 12px; border-radius: 999px; overflow: hidden; background: rgb(255 255 255 / .12); }
.bar span { display: block; height: 100%; border-radius: inherit; background: var(--accent); transition: width .24s ease; }
.hero-stats, .two-col, .compact-grid { display: grid; gap: 8px; }
.hero-stats { grid-template-columns: repeat(2, minmax(0, 1fr)); }
.two-col { grid-template-columns: repeat(2, minmax(0, 1fr)); }
.mini-stat { min-width: 0; border-radius: 16px; background: rgb(255 255 255 / .1); padding: 10px 12px; }
.mini-stat span, .compact span { display: block; margin-bottom: 5px; color: var(--muted); font-size: 12px; font-weight: 700; }
.mini-stat strong, .compact strong { display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 17px; font-variant-numeric: tabular-nums; }
.panel { border: 1px solid var(--line); border-radius: 24px; background: var(--surface); padding: 16px; box-shadow: var(--shadow); }
.panel h2 { margin: 0 0 12px; font-size: 15px; line-height: 1.2; }
.value-row { display: flex; justify-content: space-between; gap: 12px; padding: 7px 0; color: var(--muted); }
.value-row strong { color: var(--text); font-variant-numeric: tabular-nums; text-align: right; }
.compact-grid { grid-template-columns: repeat(3, minmax(0, 1fr)); }
.cooling-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
.compact { min-width: 0; border-radius: 16px; background: var(--surface-muted); padding: 10px 12px; }
.tray-list { display: grid; gap: 8px; margin-top: 12px; }
.tray { display: flex; align-items: center; justify-content: space-between; gap: 10px; min-width: 0; border-radius: 16px; background: var(--surface-muted); padding: 10px 12px; color: var(--muted); font-size: 14px; }
.tray-name { display: inline-flex; align-items: center; gap: 8px; min-width: 0; color: var(--text); font-weight: 800; }
.tray-name span:last-child, .tray > span:last-child { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.swatch { width: 16px; height: 16px; flex: 0 0 16px; border-radius: 999px; border: 1px solid var(--line-strong); background: #e5e7eb; }
.connection-panel { display: flex; align-items: center; justify-content: space-between; gap: 14px; }
.connection-panel p { margin: 3px 0 0; color: var(--muted); font-size: 13px; overflow-wrap: anywhere; }
.inline-actions { display: flex; gap: 8px; }
.round-button { display: grid; place-items: center; width: 42px; height: 42px; border: 1px solid var(--line); border-radius: 999px; background: var(--surface-muted); color: var(--text); font-weight: 900; }
.round-button.danger { border: 0; background: var(--danger-bg); color: var(--danger); }
.error { border-radius: 18px; background: var(--danger-bg); color: var(--danger); padding: 12px 14px; font-size: 14px; line-height: 1.45; }
.control-dock { position: fixed; inset: auto 0 0; z-index: 30; padding: 10px 12px max(12px, env(safe-area-inset-bottom)); background: color-mix(in srgb, var(--bg) 88%, transparent); backdrop-filter: blur(18px); }
.dock-inner { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 8px; width: min(744px, 100%); margin: 0 auto; border: 1px solid var(--line); border-radius: 24px; background: var(--surface); padding: 8px; box-shadow: 0 16px 40px rgb(0 0 0 / .16); }
.dock-button { display: grid; place-items: center; gap: 5px; min-width: 0; min-height: 64px; border: 0; border-radius: 18px; background: var(--surface-muted); color: var(--text); font-size: 12px; font-weight: 800; }
.dock-button span:last-child { max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.dock-button.danger { background: var(--danger-bg); color: var(--danger); }
.sheet-backdrop { position: fixed; inset: 0; z-index: 40; display: flex; align-items: flex-end; padding: 8px; background: rgb(9 9 11 / .42); backdrop-filter: blur(8px); }
.sheet { width: 100%; max-height: 94dvh; overflow: auto; border-radius: 28px; background: var(--bg); padding: 16px; box-shadow: 0 24px 60px rgb(0 0 0 / .32); }
.sheet-head { display: flex; align-items: flex-start; justify-content: space-between; gap: 12px; margin-bottom: 16px; }
.sheet h2 { margin: 0; font-size: 24px; line-height: 1.1; }
.sheet p { margin: 4px 0 0; color: var(--muted); font-size: 14px; line-height: 1.45; }
.add-form { display: grid; gap: 12px; }
.add-form label { display: grid; gap: 6px; color: var(--muted); font-size: 14px; font-weight: 800; }
.add-form input, .add-form select { padding: 0 14px; font-size: 16px; }
.add-form input:focus, .add-form select:focus, .printer-picker select:focus { border-color: var(--accent-strong); box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 20%, transparent); }
.message { margin: 0; border-radius: 16px; background: var(--hero); color: white; padding: 12px 14px; font-size: 14px; line-height: 1.45; }
.form-actions { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; padding-top: 2px; }
.primary-action, .secondary-action { min-height: 48px; border-radius: 16px; padding: 0 16px; font-weight: 900; }
.secondary-action { border: 1px solid var(--line); background: var(--surface); color: var(--text); box-shadow: var(--shadow); }
.toast-region { position: fixed; top: 12px; left: 50%; z-index: 60; display: grid; gap: 8px; width: min(420px, calc(100% - 24px)); transform: translateX(-50%); pointer-events: none; }
.toast { border-radius: 16px; background: var(--hero); color: white; box-shadow: 0 18px 50px rgb(0 0 0 / .22); padding: 12px 14px; font-size: 14px; font-weight: 700; }

@media (min-width: 680px) {
  .app-shell { padding-inline: 20px; }
  .topbar { margin-inline: -20px; padding-inline: 20px; }
  .sheet-backdrop { align-items: center; justify-content: center; }
  .sheet { max-width: 520px; }
}

@media (max-width: 520px) {
  .topbar { grid-template-columns: minmax(0, 1fr) 46px 46px 46px; gap: 6px; }
  .tool-button { width: 46px; height: 46px; border-radius: 15px; }
  .hero-head { padding: 14px 14px 10px; }
  .hero h1 { font-size: 25px; }
  .status { padding: 7px 9px; }
  .camera { margin-inline: 10px; border-radius: 20px; }
  .hero-body, .panel { padding: 14px; }
  .print-row { gap: 10px; }
  .task { font-size: 19px; }
  .progress-value { font-size: 36px; }
  .two-col { grid-template-columns: 1fr 1fr; }
  .compact-grid { grid-template-columns: repeat(3, minmax(0, 1fr)); }
  .compact { padding: 9px 10px; }
}
"#;

const APP_JS: &str = r##"
const state = {
  devices: [],
  loading: true,
  renderedDeviceId: "",
  selectedId: localStorage.getItem("selectedPrinterId") || "",
};
const root = document.documentElement;
const deviceView = document.getElementById("device-view");
const emptyState = document.getElementById("empty-state");
const switcher = document.getElementById("printer-switcher");
const dock = document.getElementById("control-dock");
const primaryCommand = document.getElementById("primary-command");
const primaryCommandIcon = document.getElementById("primary-command-icon");
const primaryCommandLabel = document.getElementById("primary-command-label");
const sheet = document.getElementById("add-sheet");
const form = document.getElementById("add-form");
const formMessage = document.getElementById("form-message");
const toastRegion = document.getElementById("toast-region");
const themeToggle = document.getElementById("theme-toggle");

const icons = {
  activity: '<svg viewBox="0 0 24 24"><path d="M22 12h-4l-3 8L9 4l-3 8H2"/></svg>',
  "chevron-down": '<svg viewBox="0 0 24 24"><path d="m6 9 6 6 6-6"/></svg>',
  "cloud-off": '<svg viewBox="0 0 24 24"><path d="m2 2 20 20"/><path d="M5.8 5.8A7 7 0 0 0 9 19h8.7"/><path d="M10.8 4.1A7 7 0 0 1 19 11h1a4 4 0 0 1 2.5 7.1"/></svg>',
  moon: '<svg viewBox="0 0 24 24"><path d="M12 3a6 6 0 0 0 9 7.5A9 9 0 1 1 12 3Z"/></svg>',
  pause: '<svg viewBox="0 0 24 24"><path d="M10 5v14"/><path d="M14 5v14"/></svg>',
  play: '<svg viewBox="0 0 24 24"><path d="m8 5 11 7-11 7Z"/></svg>',
  plus: '<svg viewBox="0 0 24 24"><path d="M12 5v14"/><path d="M5 12h14"/></svg>',
  power: '<svg viewBox="0 0 24 24"><path d="M12 2v10"/><path d="M18.4 6.6a9 9 0 1 1-12.8 0"/></svg>',
  refresh: '<svg viewBox="0 0 24 24"><path d="M21 12a9 9 0 0 1-15 6.7L3 16"/><path d="M3 21v-5h5"/><path d="M3 12a9 9 0 0 1 15-6.7L21 8"/><path d="M21 3v5h-5"/></svg>',
  square: '<svg viewBox="0 0 24 24"><path d="M6 6h12v12H6z"/></svg>',
  sun: '<svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="4"/><path d="M12 2v2"/><path d="M12 20v2"/><path d="m4.9 4.9 1.4 1.4"/><path d="m17.7 17.7 1.4 1.4"/><path d="M2 12h2"/><path d="M20 12h2"/><path d="m6.3 17.7-1.4 1.4"/><path d="m19.1 4.9-1.4 1.4"/></svg>',
  trash: '<svg viewBox="0 0 24 24"><path d="M3 6h18"/><path d="M8 6V4h8v2"/><path d="M19 6l-1 14H6L5 6"/><path d="M10 11v5"/><path d="M14 11v5"/></svg>',
  x: '<svg viewBox="0 0 24 24"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>',
};

function icon(name) {
  return `<span class="icon" aria-hidden="true">${icons[name] || ""}</span>`;
}

function setIcon(element, name) {
  if (element) element.innerHTML = icons[name] || "";
}

document.querySelectorAll("[data-icon]").forEach((element) => setIcon(element, element.dataset.icon));

const storedTheme = localStorage.getItem("theme");
if (storedTheme) {
  root.dataset.theme = storedTheme;
} else if (window.matchMedia && window.matchMedia("(prefers-color-scheme: dark)").matches) {
  root.dataset.theme = "dark";
}
setIcon(themeToggle.querySelector(".icon"), root.dataset.theme === "dark" ? "sun" : "moon");

const commandLabels = {
  refresh: "Refresh",
  pause: "Pause",
  resume: "Resume",
  stop: "Stop",
  connect: "Connect",
  disconnect: "Disconnect",
};

function api(path, options = {}) {
  return fetch(path, {
    ...options,
    headers: { "Content-Type": "application/json", ...(options.headers || {}) },
  }).then(async (response) => {
    if (!response.ok) throw new Error(await response.text());
    if (response.status === 204) return null;
    return response.json();
  });
}

function escapeHtml(value) {
  return String(value ?? "").replace(/[&<>"']/g, (char) => ({
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    '"': "&quot;",
    "'": "&#039;",
  }[char]));
}

function formValues() {
  const data = new FormData(form);
  return {
    name: String(data.get("name") || ""),
    host: String(data.get("host") || ""),
    serial: String(data.get("serial") || ""),
    accessCode: String(data.get("accessCode") || ""),
    model: String(data.get("model") || ""),
    mqttPort: 8883,
    mqttUseTls: true,
  };
}

function toast(text) {
  const item = document.createElement("div");
  item.className = "toast";
  item.textContent = text;
  toastRegion.appendChild(item);
  setTimeout(() => item.remove(), 3600);
}

function showMessage(text) {
  formMessage.textContent = text || "";
  formMessage.hidden = !text;
}

function fmt(value, suffix = "") {
  return value == null ? "--" : `${value}${suffix}`;
}

function temp(value, target) {
  if (value == null) return "--";
  return target == null ? `${value}C` : `${value}/${target}C`;
}

function remaining(minutes) {
  if (minutes == null) return "--";
  const hours = Math.floor(minutes / 60);
  const rest = Math.floor(minutes % 60);
  return hours ? `${hours}h ${rest}m` : `${rest}m`;
}

function time(value) {
  return value ? new Date(value).toLocaleTimeString() : "never";
}

function color(value) {
  const hex = String(value || "").replace("#", "").slice(0, 6);
  return /^[0-9a-f]{6}$/i.test(hex) ? `#${hex}` : "#e5e7eb";
}

function connectionLabel(value) {
  if (value === "online") return "Online";
  if (value === "connecting") return "Connecting";
  if (value === "error") return "Attention";
  return "Offline";
}

function speedLabel(speed) {
  if (speed === 1) return "Silent";
  if (speed === 2) return "Standard";
  if (speed === 3) return "Sport";
  if (speed === 4) return "Ludicrous";
  return "--";
}

function selectedDevice() {
  return state.devices.find((device) => device.config.id === state.selectedId) || state.devices[0] || null;
}

function setDevices(devices) {
  state.devices = devices;
  if (!selectedDevice() && devices.length) {
    state.selectedId = devices[0].config.id;
  }
  render();
}

function upsertDevice(device) {
  const index = state.devices.findIndex((item) => item.config.id === device.config.id);
  if (index >= 0) state.devices[index] = device;
  else state.devices.push(device);
  if (!state.selectedId) state.selectedId = device.config.id;
  render();
}

function miniStat(label, value) {
  return `<div class="mini-stat"><span>${label}</span><strong>${escapeHtml(value)}</strong></div>`;
}

function valueRow(label, value) {
  return `<div class="value-row"><span>${label}</span><strong>${escapeHtml(value)}</strong></div>`;
}

function compact(label, value) {
  return `<div class="compact"><span>${label}</span><strong>${escapeHtml(value)}</strong></div>`;
}

function panel(title, body) {
  return `<section class="panel"><h2>${title}</h2>${body}</section>`;
}

function renderSwitcher(device) {
  switcher.innerHTML = state.devices.length
    ? state.devices.map((item) => `<option value="${escapeHtml(item.config.id)}">${escapeHtml(item.config.name)}</option>`).join("")
    : `<option value="">No printers</option>`;
  switcher.disabled = state.devices.length === 0;
  switcher.value = device?.config.id || "";
}

function renderCamera(device) {
  const camera = device.camera || {};
  const meta = `${String(camera.mode || "").toUpperCase()} / ${camera.protocol || "rtsps"}:${camera.port || "--"}`;
  const content = camera.supported && camera.relayUrl
    ? `<iframe src="/devices/${encodeURIComponent(device.config.id)}/camera" title="${escapeHtml(device.config.name)} camera" allow="autoplay; fullscreen; microphone; camera"></iframe>`
    : `<div class="camera-empty"><div>${icon("activity")}<strong>Camera auto-detected</strong><div>${escapeHtml(camera.message || "No camera stream is available yet.")}</div></div></div>`;
  return `<div class="camera">${content}<div class="camera-pill">${escapeHtml(meta)}</div></div>`;
}

function renderDevice(device) {
  const t = device.telemetry || {};
  const progress = Math.max(0, Math.min(100, Math.round(Number(t.progress || 0))));
  const layer = t.layerCurrent && t.layerTotal ? `${t.layerCurrent}/${t.layerTotal}` : "--";
  const ams = t.ams || {};
  const trays = (ams.trays || []).slice(0, 8);
  const isOnline = device.connection === "online";
  primaryCommandLabel.textContent = isOnline ? "Pause" : "Connect";
  primaryCommand.dataset.dockCommand = isOnline ? "pause" : "connect";
  setIcon(primaryCommandIcon, isOnline ? "pause" : "power");

  const heroHead = `
        <div>
          <div class="meta">${escapeHtml(device.config.model || "Bambu printer")} / ${escapeHtml(t.state || connectionLabel(device.connection))}</div>
          <h1>${escapeHtml(device.config.name)}</h1>
        </div>
        <div class="status"><span class="dot ${escapeHtml(device.connection)}"></span>${connectionLabel(device.connection)}</div>
  `;
  const heroBody = `
        <div class="print-row">
          <div>
            <div class="meta">Current print</div>
            <div class="task">${escapeHtml(t.taskName || "No active print")}</div>
          </div>
          <div>
            <div class="progress-value">${progress}%</div>
            <div class="progress-label">complete</div>
          </div>
        </div>
        <div class="bar"><span style="width:${progress}%"></span></div>
        <div class="hero-stats">
          ${miniStat("Layer", layer)}
          ${miniStat("Remaining", remaining(t.remainingMinutes))}
        </div>
  `;
  const details = `
    ${device.error ? `<div class="error">${escapeHtml(device.error)}</div>` : ""}

    <div class="two-col">
      ${panel("Temperatures", `
        ${valueRow("Nozzle", temp(t.nozzleTemp, t.nozzleTarget))}
        ${valueRow("Bed", temp(t.bedTemp, t.bedTarget))}
        ${valueRow("Chamber", temp(t.chamberTemp))}
      `)}
      ${panel("Motion", `
        ${valueRow("Mode", speedLabel(t.speedLevel))}
        ${valueRow("Speed level", fmt(t.speedLevel))}
        ${valueRow("State", t.state || "--")}
      `)}
    </div>

    ${panel("AMS & material", `
      <div class="compact-grid">
        ${compact("Active", ams.activeTray || "--")}
        ${compact("Target", ams.targetTray || "--")}
        ${compact("Humidity", ams.humidity == null ? "--" : String(ams.humidity))}
      </div>
      <div class="tray-list">
        ${trays.length ? trays.map((tray) => `
          <div class="tray">
            <span class="tray-name"><span class="swatch" style="background:${color(tray.color)}"></span><span>${escapeHtml(tray.id)}</span></span>
            <span>${escapeHtml(tray.material || "Material")}</span>
          </div>
        `).join("") : `<div class="tray"><span>No AMS tray data reported yet.</span></div>`}
      </div>
    `)}

    ${panel("Cooling", `
      <div class="compact-grid cooling-grid">
        ${compact("Part", fmt(t.fanSpeed, "%"))}
        ${compact("Aux", fmt(t.auxiliaryFanSpeed, "%"))}
        ${compact("Chamber", fmt(t.chamberFanSpeed, "%"))}
        ${compact("Hotend", fmt(t.heatbreakFanSpeed, "%"))}
      </div>
    `)}

    <section class="panel connection-panel">
      <div>
        <h2>Connection</h2>
        <p>${escapeHtml(device.config.host)} / last update ${escapeHtml(time(device.lastSeenAt))}</p>
      </div>
      <div class="inline-actions">
        <button class="round-button" data-command="${device.connection === "offline" ? "connect" : "disconnect"}" type="button" aria-label="${device.connection === "offline" ? "Connect printer" : "Disconnect printer"}">${icon(device.connection === "offline" ? "power" : "cloud-off")}</button>
        <button class="round-button danger" data-delete="${escapeHtml(device.config.id)}" type="button" aria-label="Delete ${escapeHtml(device.config.name)}">${icon("trash")}</button>
      </div>
    </section>
  `;

  if (state.renderedDeviceId === device.config.id && deviceView.querySelector(".hero")) {
    deviceView.querySelector(".hero-head").innerHTML = heroHead;
    deviceView.querySelector(".hero-body").innerHTML = heroBody;
    deviceView.querySelector(".device-details").innerHTML = details;
  } else {
    deviceView.innerHTML = `
      <section class="hero">
        <div class="hero-head">${heroHead}</div>
        ${renderCamera(device)}
        <div class="hero-body">${heroBody}</div>
      </section>
      <div class="device-details">${details}</div>
    `;
  }

  state.renderedDeviceId = device.config.id;
}

function render() {
  const device = selectedDevice();
  renderSwitcher(device);
  emptyState.hidden = Boolean(device) || state.loading;
  deviceView.hidden = !device;
  dock.hidden = !device;
  if (device) {
    state.selectedId = device.config.id;
    localStorage.setItem("selectedPrinterId", state.selectedId);
    renderDevice(device);
  }
}

async function loadDevices() {
  const devices = await api("/devices/");
  state.loading = false;
  setDevices(devices);
}

async function sendCommand(command) {
  const device = selectedDevice();
  if (!device) return;
  try {
    const next = await api(`/devices/${encodeURIComponent(device.config.id)}/command`, {
      method: "POST",
      body: JSON.stringify({ command }),
    });
    if (next) upsertDevice(next);
  } catch (error) {
    toast(error.message || "Command failed");
  }
}

async function deleteSelected(id) {
  try {
    await api(`/devices/${encodeURIComponent(id)}`, { method: "DELETE" });
    state.devices = state.devices.filter((device) => device.config.id !== id);
    if (state.selectedId === id) state.selectedId = "";
    render();
  } catch (error) {
    toast(error.message || "Delete failed");
  }
}

function openSheet() {
  sheet.hidden = false;
  setTimeout(() => form.elements.name?.focus(), 0);
}

function closeSheet() {
  sheet.hidden = true;
  showMessage("");
}

document.getElementById("refresh").addEventListener("click", () => loadDevices().catch((error) => toast(error.message || "Could not load devices")));
document.getElementById("open-add").addEventListener("click", openSheet);
document.getElementById("empty-add").addEventListener("click", openSheet);
document.getElementById("close-add").addEventListener("click", closeSheet);
sheet.addEventListener("click", (event) => {
  if (event.target === sheet) closeSheet();
});

document.getElementById("theme-toggle").addEventListener("click", () => {
  const next = root.dataset.theme === "dark" ? "light" : "dark";
  root.dataset.theme = next;
  localStorage.setItem("theme", next);
  setIcon(themeToggle.querySelector(".icon"), next === "dark" ? "sun" : "moon");
});

switcher.addEventListener("change", (event) => {
  state.selectedId = event.target.value;
  render();
});

form.addEventListener("submit", async (event) => {
  event.preventDefault();
  showMessage("");
  const submitButton = form.querySelector('[type="submit"]');
  submitButton.disabled = true;
  submitButton.textContent = "Adding...";
  try {
    await api("/devices/", { method: "POST", body: JSON.stringify(formValues()) });
    form.reset();
    closeSheet();
    toast("Printer added");
    await loadDevices();
  } catch (error) {
    toast(error.message || "Could not add printer");
  } finally {
    submitButton.disabled = false;
    submitButton.textContent = "Add printer";
  }
});

document.getElementById("probe").addEventListener("click", async (event) => {
  showMessage("");
  const probeButton = event.currentTarget;
  probeButton.disabled = true;
  probeButton.textContent = "Testing...";
  try {
    const values = formValues();
    const result = await api("/devices/probe", {
      method: "POST",
      body: JSON.stringify({ host: values.host, mqttPort: 8883 }),
    });
    showMessage(result.ok ? `Printer reachable in ${result.latencyMs} ms` : (result.error || "Connection test failed"));
  } catch (error) {
    showMessage(error.message || "Probe failed");
  } finally {
    probeButton.disabled = false;
    probeButton.textContent = "Test";
  }
});

dock.addEventListener("click", (event) => {
  const button = event.target.closest("button[data-dock-command]");
  if (button) sendCommand(button.dataset.dockCommand);
});

deviceView.addEventListener("click", (event) => {
  const commandButton = event.target.closest("button[data-command]");
  if (commandButton) sendCommand(commandButton.dataset.command);
  const deleteButton = event.target.closest("button[data-delete]");
  if (deleteButton) deleteSelected(deleteButton.dataset.delete);
});

loadDevices().catch((error) => {
  state.loading = false;
  toast(error.message || "Could not load devices");
  render();
});

const source = new EventSource("/devices/events");
source.onmessage = (message) => {
  const event = JSON.parse(message.data);
  if (event.type === "devices") setDevices(event.devices);
  if (event.type === "snapshot") upsertDevice(event.device);
};
source.onerror = () => {
  toast("Live updates disconnected");
  source.close();
};

if ("serviceWorker" in navigator) {
  window.addEventListener("load", () => {
    navigator.serviceWorker.register("/assets/sw.js").catch(() => {});
  });
}
"##;
