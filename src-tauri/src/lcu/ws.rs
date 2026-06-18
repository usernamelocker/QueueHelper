use std::time::Duration;

use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use futures_util::{SinkExt, StreamExt};
use native_tls::TlsConnector;
use tokio::time::sleep;
use tokio_tungstenite::{
    connect_async_tls_with_config,
    tungstenite::{client::IntoClientRequest, http::HeaderValue, Message},
    Connector,
};
use tokio_util::sync::CancellationToken;

use crate::{
    core::{event_bus::EventBus, events::AppEvent},
    lcu::types::{ChampSelectSession, LockfileInfo},
    models::MonitorLevel,
};

pub async fn run_ws_loop(bus: EventBus, lockfile: LockfileInfo, shutdown: CancellationToken) {
    let mut retry_count = 0u32;

    loop {
        if shutdown.is_cancelled() {
            break;
        }

        if let Err(error) = connect_and_stream(&bus, &lockfile, &shutdown).await {
            retry_count += 1;
            // only log warnings periodically to avoid spam during initial LCU startup
            if retry_count == 1 || retry_count % 10 == 0 {
                bus.publish(AppEvent::Monitor {
                    level: MonitorLevel::Warn,
                    category: "lcu-ws".to_string(),
                    message: format!("WebSocket stream disconnected: {}", error),
                });
            }
        } else {
            retry_count = 0;
        }

        let backoff_secs = (retry_count as u64).saturating_mul(2).min(60);
        let delay = Duration::from_secs(backoff_secs);

        tokio::select! {
            _ = shutdown.cancelled() => break,
            _ = sleep(delay) => {}
        }
    }
}

async fn connect_and_stream(bus: &EventBus, lockfile: &LockfileInfo, shutdown: &CancellationToken) -> Result<()> {
    let auth = format!(
        "Basic {}",
        general_purpose::STANDARD.encode(format!("riot:{}", lockfile.password))
    );

    let url = format!("wss://127.0.0.1:{}/", lockfile.port);
    let mut request = url
        .into_client_request()
        .context("failed constructing websocket request")?;
    request
        .headers_mut()
        .insert("Authorization", HeaderValue::from_str(&auth)?);

    let tls = TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .context("failed building websocket TLS connector")?;
    let connector = Connector::NativeTls(tls);

    let (stream, _) = connect_async_tls_with_config(request, None, false, Some(connector))
        .await
        .context("failed connecting to LCU websocket endpoint")?;

    bus.publish(AppEvent::Monitor {
        level: MonitorLevel::Info,
        category: "lcu-ws".to_string(),
        message: "Connected to LCU websocket".to_string(),
    });

    let (mut write, mut read) = stream.split();

    for topic in [
        "OnJsonApiEvent_lol-gameflow_v1_session",
        "OnJsonApiEvent_lol-champ-select_v1_session",
        "OnJsonApiEvent_lol-matchmaking_v1_ready-check",
    ] {
        let payload = format!("[5,\"{}\"]", topic);
        write
            .send(Message::Text(payload))
            .await
            .context("failed subscribing to LCU websocket topic")?;
    }

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => {
                let _ = write.send(Message::Close(None)).await;
                return Ok(());
            }
            next = read.next() => {
                let Some(frame) = next else {
                    return Ok(());
                };
                match frame {
                    Ok(Message::Text(text)) => handle_ws_text(bus, &text),
                    Ok(Message::Ping(payload)) => {
                        let _ = write.send(Message::Pong(payload)).await;
                    }
                    Ok(Message::Close(_)) => return Ok(()),
                    Err(error) => return Err(anyhow::anyhow!("websocket frame error: {}", error)),
                    _ => {}
                }
            }
        }
    }
}

fn handle_ws_text(bus: &EventBus, raw_text: &str) {
    let Ok(message) = serde_json::from_str::<serde_json::Value>(raw_text) else {
        return;
    };

    let Some(payload_array) = message.as_array() else {
        return;
    };
    if payload_array.len() < 3 {
        return;
    }

    let Some(topic) = payload_array.get(1).and_then(serde_json::Value::as_str) else {
        return;
    };
    let Some(event_payload) = payload_array.get(2) else {
        return;
    };
    let data = event_payload
        .get("data")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    if topic.contains("lol-gameflow_v1_session") {
        let phase = data
            .get("phase")
            .and_then(serde_json::Value::as_str)
            .or_else(|| data.as_str())
            .unwrap_or("Unknown")
            .to_string();
        bus.publish(AppEvent::GameflowPhaseUpdated { phase });
        return;
    }

    if topic.contains("lol-matchmaking_v1_ready-check") {
        let state = data
            .get("playerResponse")
            .and_then(serde_json::Value::as_str)
            .or_else(|| data.get("state").and_then(serde_json::Value::as_str))
            .or_else(|| data.as_str())
            .unwrap_or("Unknown")
            .to_string();
        let queue_id = data
            .get("queueId")
            .and_then(serde_json::Value::as_i64);
        bus.publish(AppEvent::ReadyCheckUpdated { state, queue_id });
        return;
    }

    if topic.contains("lol-champ-select_v1_session") {
        if let Ok(session) = serde_json::from_value::<ChampSelectSession>(data) {
            bus.publish(AppEvent::ChampSelectSessionUpdated { session });
        } else {
            bus.publish(AppEvent::Monitor {
                level: MonitorLevel::Warn,
                category: "champ-select".to_string(),
                message: "Received unexpected champ select payload shape".to_string(),
            });
        }
    }
}

