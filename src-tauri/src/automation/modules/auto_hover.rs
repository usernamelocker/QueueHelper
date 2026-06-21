use std::sync::Arc;
use std::time::Duration;

use rand::Rng;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::{app::AppContext, automation::modules::shared::perform_hover, core::events::AppEvent, models::MonitorLevel};

pub async fn run_auto_hover(context: Arc<AppContext>, shutdown: CancellationToken) {
    let mut receiver = context.bus.subscribe();
    let mut has_hovered = false;

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => break,
            event_result = receiver.recv() => {
                let event = match event_result {
                    Ok(event) => event,
                    Err(RecvError::Lagged(n)) => {
                        context.monitor(MonitorLevel::Warn, "event-bus",
                            format!("Dropped {} events (bus overflow)", n));
                        continue;
                    }
                    Err(_) => break,
                };

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

                        let delay_seconds = {
                            let state = context.state.read().await;
                            state.settings.automation.auto_hover_delay_seconds
                        };

                        let local_actions: Vec<_> = session.actions.iter()
                            .flatten()
                            .filter(|a| a.actor_cell_id == session.local_player_cell_id && a.r#type == "pick")
                            .collect();

                        if local_actions.is_empty() { continue; }

                        // delay before first hover (also gives rules_engine time to switch profile)
                        if delay_seconds > 0.0 {
                            let delay_ms = {
                                let half = (delay_seconds * 500.0) as u64;
                                let mut rng = rand::rng();
                                let jitter = rng.random_range(0..=half);
                                (delay_seconds * 1000.0) as u64 - half + jitter
                            };
                            tokio::select! {
                                _ = shutdown.cancelled() => break,
                                _ = sleep(Duration::from_millis(delay_ms)) => {}
                            }
                        } else {
                            tokio::select! {
                                _ = shutdown.cancelled() => break,
                                _ = sleep(Duration::from_millis(50)) => {}
                            }
                        }

                        // read profile AFTER delay so rules_engine's auto-switch has taken effect
                        let target_champion_id = {
                            let state = context.state.read().await;
                            let id = state.profiles.active_profile_id.clone();
                            id.and_then(|pid| {
                                state.profiles.profiles.iter()
                                    .find(|p| p.id == pid)
                                    .and_then(|p| p.pick_priority.iter().find(|e| e.is_hover_target).map(|e| e.champion_id))
                            })
                        };

                        let Some(champion_id) = target_champion_id else { continue };

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


