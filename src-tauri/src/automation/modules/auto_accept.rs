use std::time::Duration;

use anyhow::Result;
use rand::Rng;
use tokio::time::sleep;

use crate::{
    app::AppContext,
    core::events::AppEvent,
    models::MonitorLevel,
};

pub async fn run_auto_accept(context: std::sync::Arc<AppContext>, shutdown: tokio_util::sync::CancellationToken) {
    let mut receiver = context.bus.subscribe();

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => break,
            event_result = receiver.recv() => {
                let Ok(event) = event_result else {
                    continue;
                };

                match event {
                    AppEvent::ReadyCheckUpdated { state, queue_id } => {
                        let accept_enabled = {
                            let state = context.state.read().await;
                            let global_enabled = state.settings.automation.auto_accept.enabled;
                            let override_enabled = queue_id.and_then(|qid| {
                                state.settings.automation.queue_overrides.iter().find(|qo| qo.queue_id == qid).map(|qo| qo.auto_accept_enabled)
                            });
                            override_enabled.unwrap_or(global_enabled)
                        };

                        if !accept_enabled {
                            continue;
                        }

                        if state == "Found" || state == "InProgress" {
                            let delay = {
                                let state = context.state.read().await;
                                let settings = &state.settings.automation.auto_accept;
                                let mut rng = rand::rng();
                                let min_ms = (settings.delay_min_seconds * 1000.0) as u64;
                                let max_ms = (settings.delay_max_seconds * 1000.0) as u64;
                                let delay_ms = rng.random_range(min_ms..=max_ms.max(min_ms + 100));
                                Duration::from_millis(delay_ms)
                            };

                            context.monitor(
                                MonitorLevel::Info,
                                "auto-accept",
                                format!("Match found – accepting in {:.2}s", delay.as_secs_f32()),
                            );

                            sleep(delay).await;

                            if shutdown.is_cancelled() {
                                break;
                            }

                            match accept_queue(context.clone()).await {
                                Ok(_) => {
                                    context.monitor(
                                        MonitorLevel::Info,
                                        "auto-accept",
                                        "Queue accepted".to_string(),
                                    );
                                }
                                Err(error) => {
                                    context.monitor(
                                        MonitorLevel::Error,
                                        "auto-accept",
                                        format!("Failed to accept queue: {}", error),
                                    );
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

async fn accept_queue(context: std::sync::Arc<AppContext>) -> Result<()> {
    let client = context.lcu_client.read().await.clone();
    let Some(client) = client else {
        return Err(anyhow::anyhow!("LCU client not connected"));
    };

    client
        .post_json("/lol-matchmaking/v1/ready-check/accept", Some(serde_json::Value::Null))
        .await?;

    Ok(())
}

