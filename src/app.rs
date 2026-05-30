use leptos::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <main class="app-shell">
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
        </main>
    }
}
