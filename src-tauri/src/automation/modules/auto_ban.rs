use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use rand::Rng;
use serde_json::json;
use tokio::time::sleep;

use crate::{app::AppContext, core::events::AppEvent, models::MonitorLevel};

const STALE_LOCK_TIMEOUT: Duration = Duration::from_millis(1500);

pub async fn run_auto_ban(context: Arc<AppContext>, shutdown: tokio_util::sync::CancellationToken) {
    let mut receiver = context.bus.subscribe();
    let mut processed_action_ids: HashSet<i64> = HashSet::new();
    let mut pending_hover: HashSet<i64> = HashSet::new();
    let mut pending_lock: HashMap<i64, (i64, Instant)> = HashMap::new();
    let mut failed_attempts: HashMap<i64, HashSet<i64>> = HashMap::new();
    let mut pending_pick_position_request: Option<u32> = None;
    let mut sweep_timer = tokio::time::interval(Duration::from_secs(1));

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => break,
            _ = sweep_timer.tick() => {
                let now = Instant::now();
                let mut timed_out: Vec<i64> = Vec::new();
                for (&action_id, &(_, ts)) in &pending_lock {
                    if now.duration_since(ts) >= STALE_LOCK_TIMEOUT {
                        timed_out.push(action_id);
                    }
                }
                for id in timed_out {
                    if let Some((champ, _)) = pending_lock.remove(&id) {
                        failed_attempts.entry(id).or_default().insert(champ);
                        context.monitor(
                            MonitorLevel::Error,
                            "auto-ban",
                            format!("Lock timed out for champion {} — LCU rejected it", champ),
                        );
                    }
                }
            }
            event_result = receiver.recv() => {
                let Ok(event) = event_result else { continue };

                match event {
                    AppEvent::ChampSelectSessionUpdated { session } => {
                        let has_actions = session.actions.iter().any(|a| !a.is_empty());
                        if !has_actions {
                            processed_action_ids.clear();
                            pending_hover.clear();
                            pending_lock.clear();
                            failed_attempts.clear();
                            pending_pick_position_request = None;
                            continue;
                        }

                        if session.timer.phase != "BAN_PICK" { continue }

                        let mut newly_completed: Vec<(i64, i64)> = Vec::new();
                        for action in session.actions.iter().flatten() {
                            if let Some(&(champ, _)) = pending_lock.get(&action.id) {
                                if action.completed {
                                    newly_completed.push((action.id, champ));
                                } else if action.champion_id != 0 && action.champion_id != champ {
                                    failed_attempts.entry(action.id).or_default().insert(champ);
                                    pending_lock.remove(&action.id);
                                }
                            }
                        }
                        for (id, champion_id) in newly_completed {
                            pending_lock.remove(&id);
                            failed_attempts.remove(&id);
                            processed_action_ids.insert(id);
                            context.monitor(MonitorLevel::Info, "auto-ban", format!("Banned champion #{}", champion_id));
                            if let Some(desired_pos) = pending_pick_position_request.take() {
                                request_pick_position(&context, desired_pos, &session).await;
                            }
                        }

                        {
                            let now = Instant::now();
                            let mut timed_out: Vec<i64> = Vec::new();
                            for (&action_id, &(_, ts)) in &pending_lock {
                                if now.duration_since(ts) >= STALE_LOCK_TIMEOUT {
                                    timed_out.push(action_id);
                                }
                            }
                            for id in timed_out {
                                if let Some((champ, _)) = pending_lock.remove(&id) {
                                    failed_attempts.entry(id).or_default().insert(champ);
                                }
                            }
                        }

                        pending_hover.retain(|id| {
                            session.actions.iter().flatten().any(|a| a.id == *id && a.is_in_progress && !a.completed)
                        });

                        let is_enabled = {
                            let state = context.state.read().await;
                            state.settings.automation.auto_ban_enabled
                        };
                        if !is_enabled { continue }

                        let (profile, delay_seconds) = {
                            let state = context.state.read().await;
                            let id = state.profiles.active_profile_id.clone();
                            let profile = id.and_then(|pid| state.profiles.profiles.iter().find(|p| p.id == pid).cloned());
                            let delay = state.settings.automation.auto_ban_delay_seconds;
                            (profile, delay)
                        };
                        let Some(profile) = profile else { continue };

                        let Some(action) = session
                            .actions
                            .iter()
                            .flatten()
                            .find(|a| {
                                a.r#type == "ban"
                                    && a.actor_cell_id == session.local_player_cell_id
                                    && a.is_in_progress
                                    && !a.completed
                                    && !processed_action_ids.contains(&a.id)
                                    && !pending_hover.contains(&a.id)
                                    && !pending_lock.contains_key(&a.id)
                            })
                        else { continue };

                        let already_banned: Vec<i64> = session
                            .bans
                            .my_team_bans
                            .iter()
                            .chain(session.bans.their_team_bans.iter())
                            .copied()
                            .collect();

                        let team_hovered: Vec<i64> = session
                            .my_team
                            .iter()
                            .map(|p| p.champion_id)
                            .filter(|cid| *cid != 0)
                            .collect();

                        let unavailable: Vec<i64> = already_banned
                            .into_iter()
                            .chain(team_hovered)
                            .collect();

                        let skip = failed_attempts.get(&action.id);

                        let champion_id = profile
                            .ban_priority
                            .iter()
                            .chain(profile.pick_priority.iter())
                            .find(|entry| {
                                if unavailable.contains(&entry.champion_id) {
                                    return false;
                                }
                                if let Some(skip_set) = skip {
                                    if skip_set.contains(&entry.champion_id) {
                                        return false;
                                    }
                                }
                                true
                            })
                            .map(|entry| entry.champion_id);

                        let Some(champion_id) = champion_id else {
                            context.monitor(MonitorLevel::Warn, "auto-ban", "No available champion to ban".to_string());
                            continue;
                        };

                        pending_hover.insert(action.id);

                        if delay_seconds > 0.0 {
                            let _ = perform_hover(&context, action.id, champion_id).await;
                            let delay_ms = {
                                let half = (delay_seconds * 500.0) as u64;
                                let mut rng = rand::rng();
                                let jitter = rng.random_range(0..=half);
                                (delay_seconds * 1000.0) as u64 - half + jitter
                            };
                            sleep(Duration::from_millis(delay_ms)).await;
                        }

                        if shutdown.is_cancelled() { break }

                        match perform_ban(&context, action.id, champion_id).await {
                            Ok(_) => {
                                pending_hover.remove(&action.id);
                                pending_lock.insert(action.id, (champion_id, Instant::now()));
                                if profile.request_pick_position > 0 {
                                    pending_pick_position_request = Some(profile.request_pick_position);
                                }
                            }
                            Err(e) => {
                                pending_hover.remove(&action.id);
                                context.monitor(
                                    MonitorLevel::Error,
                                    "auto-ban",
                                    format!("Failed to ban: {}", e),
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

async fn request_pick_position(context: &Arc<AppContext>, desired_position: u32, session: &crate::lcu::types::ChampSelectSession) {
    if desired_position == 0 { return; }

    let mut team_pick_actions: Vec<(i64, i64)> = session
        .actions
        .iter()
        .flatten()
        .filter(|a| {
            a.r#type == "pick"
                && session.my_team.iter().any(|p| p.cell_id == a.actor_cell_id)
        })
        .map(|a| (a.actor_cell_id, a.pick_turn))
        .collect();

    team_pick_actions.sort_by_key(|&(_, turn)| turn);

    let local_pos = team_pick_actions
        .iter()
        .position(|&(cell, _)| cell == session.local_player_cell_id)
        .map(|i| i + 1);

    let Some(current_position) = local_pos else { return };
    if desired_position == current_position as u32 { return; }
    if desired_position as usize > team_pick_actions.len() { return; }

    let target_cell_id = team_pick_actions[desired_position as usize - 1].0;
    let client = context.lcu_client.read().await.clone();
    let Some(client) = client else { return };

    match client
        .post_json(
            &format!("/lol-champ-select/v1/session/pick-order-swaps/{}/request", target_cell_id),
            None,
        )
        .await
    {
        Ok(_) => {
            context.monitor(
                crate::models::MonitorLevel::Info,
                "auto-ban",
                format!("Requested pick position {} (swap with cell {})", desired_position, target_cell_id),
            );
        }
        Err(e) => {
            context.monitor(
                crate::models::MonitorLevel::Error,
                "auto-ban",
                format!("Failed to request pick position: {}", e),
            );
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

async fn perform_ban(context: &Arc<AppContext>, action_id: i64, champion_id: i64) -> Result<()> {
    let client = context.lcu_client.read().await.clone();
    let Some(client) = client else {
        return Err(anyhow::anyhow!("LCU client not connected"));
    };

    client
        .patch_json(
            &format!("/lol-champ-select/v1/session/actions/{}", action_id),
            json!({
                "championId": champion_id,
                "completed": true
            }),
        )
        .await?;

    Ok(())
}
