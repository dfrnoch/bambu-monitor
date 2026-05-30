mod app;
mod devices;

use axum::{Router, routing::get};
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use std::{env, net::SocketAddr};
use tokio::net::TcpListener;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::app::App;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3000);
    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .expect("parse HTTP listener address");
    let leptos_options = LeptosOptions::builder()
        .output_name("bambu-monitor")
        .site_root("target/site")
        .site_pkg_dir("pkg")
        .site_addr(addr)
        .build();
    let routes = generate_route_list(App);

    let app = Router::<LeptosOptions>::new()
        .route("/health", get(health))
        .merge(devices::router())
        .nest_service("/assets", ServeDir::new("assets"))
        .leptos_routes_with_context(&leptos_options, routes, || {}, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .layer(TraceLayer::new_for_http())
        .with_state(leptos_options);

    let listener = TcpListener::bind(addr).await.expect("bind HTTP listener");
    tracing::info!("Bambu Monitor listening on http://{addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("serve HTTP");
}

async fn health() -> &'static str {
    "ok"
}

fn shell(_options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <meta name="theme-color" content="#111113" media="(prefers-color-scheme: light)" />
                <meta name="theme-color" content="#050506" media="(prefers-color-scheme: dark)" />
                <meta name="apple-mobile-web-app-capable" content="yes" />
                <meta name="apple-mobile-web-app-status-bar-style" content="black-translucent" />
                <meta name="apple-mobile-web-app-title" content="Bambu Monitor" />
                <link rel="manifest" href="/assets/manifest.webmanifest" />
                <link rel="stylesheet" href="/assets/app.css" />
                <link rel="icon" href="/assets/icon.svg" type="image/svg+xml" />
                <link rel="apple-touch-icon" href="/assets/icon-192.svg" />
                <script defer src="/assets/app.js"></script>
                <title>"Bambu Monitor"</title>
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install terminate handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
