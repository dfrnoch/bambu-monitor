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
