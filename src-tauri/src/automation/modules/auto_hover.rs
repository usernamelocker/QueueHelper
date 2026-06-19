use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use rand::Rng;
use serde_json::json;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::{app::AppContext, core::events::AppEvent, models::MonitorLevel};

pub async fn run_auto_hover(context: Arc<AppContext>, shutdown: CancellationToken) {
    let mut receiver = context.bus.subscribe();
    let mut has_hovered = false;

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => break,
            event_result = receiver.recv() => {
                let Ok(event) = event_result else { continue };

                match event {
                    AppEvent::ChampSelectSessionUpdated { session } => {
                        let is_enabled = {
                            let state = context.state.read().await;
                            if state.settings.automation.paused { false }
                            else { state.settings.automation.auto_hover_enabled }
                        };
                        if !is_enabled {
                            has_hovered = false;
                            continue;
                        }

                        let is_planning = session.timer.phase == "PLANNING";
                        if !is_planning {
                            has_hovered = false;
                            continue;
                        }

                        if has_hovered {
                            continue;
                        }

                        let (target_champion_id, delay_seconds) = {
                            let state = context.state.read().await;
                            let id = state.profiles.active_profile_id.clone();
                            let cid = id.and_then(|pid| {
                                state.profiles.profiles.iter()
                                    .find(|p| p.id == pid)
                                    .and_then(|p| p.pick_priority.iter().find(|e| e.is_hover_target).map(|e| e.champion_id))
                            });
                            (cid, state.settings.automation.auto_hover_delay_seconds)
                        };

                        let Some(champion_id) = target_champion_id else { continue };

                        let local_actions: Vec<_> = session.actions.iter()
                            .flatten()
                            .filter(|a| a.actor_cell_id == session.local_player_cell_id && a.r#type == "pick")
                            .collect();

                        if local_actions.is_empty() { continue; }

                        // delay before first hover
                        if delay_seconds > 0.0 {
                            let delay_ms = {
                                let half = (delay_seconds * 500.0) as u64;
                                let mut rng = rand::rng();
                                let jitter = rng.random_range(0..=half);
                                (delay_seconds * 1000.0) as u64 - half + jitter
                            };
                            sleep(Duration::from_millis(delay_ms)).await;
                        }

                        if shutdown.is_cancelled() { break }

                        let mut all_ok = true;
                        for action in &local_actions {
                            if let Err(e) = perform_hover(&context, action.id, champion_id).await {
                                context.monitor(MonitorLevel::Error, "auto-hover", format!("Failed to hover: {}", e));
                                all_ok = false;
                            }
                        }

                        if all_ok {
                            has_hovered = true;
                            context.monitor(MonitorLevel::Info, "auto-hover", format!("Hovered champion #{}", champion_id));
                        }
                    }
                    AppEvent::GameflowPhaseUpdated { phase } => {
                        if phase != "ChampSelect" && !phase.contains("Game") {
                            has_hovered = false;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

async fn perform_hover(context: &Arc<AppContext>, action_id: i64, champion_id: i64) -> Result<()> {
    let client = context.lcu_client.read().await.clone();
    let Some(client) = client else {
        return Err(anyhow::anyhow!("LCU client not connected"));
    };

    client
        .patch_json(
            &format!("/lol-champ-select/v1/session/actions/{}", action_id),
            json!({ "championId": champion_id }),
        )
        .await?;

    Ok(())
}
