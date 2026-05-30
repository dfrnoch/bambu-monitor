use axum::{
    Json, Router,
    extract::Path,
    http::{HeaderMap, StatusCode, header},
    response::{
        Html, IntoResponse,
        sse::{Event as SseEvent, KeepAlive, Sse},
    },
    routing::{get, post, put},
};
use chrono::SecondsFormat;
use leptos::prelude::LeptosOptions;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS, Transport};
use rustls::{
    ClientConfig, DigitallySignedStruct, Error as TlsError, SignatureScheme,
    client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    pki_types::{CertificateDer, ServerName, UnixTime},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    convert::Infallible,
    env,
    path::PathBuf,
    process::{Command, Stdio},
    sync::{
        Arc, LazyLock,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{RwLock, broadcast, mpsc},
    task::JoinHandle,
    time::{Instant, timeout},
};
use tokio_stream::{StreamExt, wrappers::BroadcastStream};
use uuid::Uuid;

mod camera;
mod client;
mod error;
mod manager;
mod routes;
mod snapshot;
mod telemetry;
mod types;
mod utils;

use camera::*;
use client::*;
use error::*;
use manager::*;
use snapshot::*;
use telemetry::*;
use types::*;
use utils::*;

pub use routes::router;
