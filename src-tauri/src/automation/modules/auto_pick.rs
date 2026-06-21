use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use rand::Rng;
use serde_json::json;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::{
    app::AppContext,
    automation::modules::shared::perform_hover,
    core::events::AppEvent,
    lcu::types::ChampSelectSession,
    models::MonitorLevel,
};

const STALE_LOCK_TIMEOUT: Duration = Duration::from_millis(1500);

pub async fn run_auto_pick(context: Arc<AppContext>, shutdown: tokio_util::sync::CancellationToken) {
    let mut receiver = context.bus.subscribe();
    let mut processed_action_ids: HashSet<i64> = HashSet::new();
    let mut pending_hover: HashSet<i64> = HashSet::new();
    let mut pending_lock: HashMap<i64, (i64, Instant)> = HashMap::new();
    let mut failed_attempts: HashMap<i64, HashSet<i64>> = HashMap::new();
    let mut last_session: Option<ChampSelectSession> = None;
    let mut last_cell_id: Option<i64> = None;
    let mut sweep_timer = tokio::time::interval_at(
        tokio::time::Instant::now() + Duration::from_secs(1),
        Duration::from_secs(1),
    );

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
                for &id in &timed_out {
                    if let Some((champ, _)) = pending_lock.remove(&id) {
                        failed_attempts.entry(id).or_default().insert(champ);
                    }
                }
                if !timed_out.is_empty() {
                    if let Some(ref session) = last_session {
                        let _ = try_process_session(
                            &context, session, &shutdown,
                            &mut processed_action_ids,
                            &mut pending_hover,
                            &mut pending_lock,
                            &mut failed_attempts,
                        ).await;
                    }
                }
            }
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
                        let has_actions = session.actions.iter().any(|a| !a.is_empty());
                        if !has_actions {
                            pending_hover.clear();
                            pending_lock.clear();
                            failed_attempts.clear();
                            last_session = None;
                            continue;
                        }

                        if let Some(prev) = last_cell_id {
                            if session.local_player_cell_id != prev && session.local_player_cell_id != 0 {
                                processed_action_ids.clear();
                                pending_hover.clear();
                                pending_lock.clear();
                                failed_attempts.clear();
                                last_session = None;
                            }
                        }
                        if session.local_player_cell_id != 0 {
                            last_cell_id = Some(session.local_player_cell_id);
                        }

                        if session.timer.phase != "BAN_PICK" { continue }

                        last_session = Some(session.clone());

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
                        for (id, _) in newly_completed {
                            pending_lock.remove(&id);
                            failed_attempts.remove(&id);
                            processed_action_ids.insert(id);
                        }

                        {
                            let now = Instant::now();
                            let mut timed_out: Vec<i64> = Vec::new();
                            for (&action_id, &(_, ts)) in &pending_lock {
                                if now.duration_since(ts) >= STALE_LOCK_TIMEOUT {
                                    timed_out.push(action_id);
                                }
                            }
                            for &id in &timed_out {
                                if let Some((champ, _)) = pending_lock.remove(&id) {
                                    failed_attempts.entry(id).or_default().insert(champ);
                                }
                            }
                        }

                        let _ = try_process_session(
                            &context, &session, &shutdown,
                            &mut processed_action_ids,
                            &mut pending_hover,
                            &mut pending_lock,
                            &mut failed_attempts,
                        ).await;
                    }
                    _ => {}
                }
            }
        }
    }
}

async fn try_process_session(
    context: &Arc<AppContext>,
    session: &ChampSelectSession,
    shutdown: &CancellationToken,
    processed_action_ids: &mut HashSet<i64>,
    pending_hover: &mut HashSet<i64>,
    pending_lock: &mut HashMap<i64, (i64, Instant)>,
    failed_attempts: &mut HashMap<i64, HashSet<i64>>,
) -> Result<(), String> {
    pending_hover.retain(|id| {
        session.actions.iter().flatten().any(|a| a.id == *id && a.is_in_progress && !a.completed)
    });

    let is_enabled = {
        let state = context.state.read().await;
        if state.settings.automation.paused { false }
        else { state.settings.automation.auto_pick_enabled }
    };
    if !is_enabled { return Ok(()) }

    let delay_seconds = {
        let state = context.state.read().await;
        state.settings.automation.auto_pick_delay_seconds
    };

    let Some(action) = session
        .actions
        .iter()
        .flatten()
        .find(|a| {
            a.r#type == "pick"
                && a.actor_cell_id == session.local_player_cell_id
                && a.is_in_progress
                && !a.completed
                && !processed_action_ids.contains(&a.id)
                && !pending_hover.contains(&a.id)
                && !pending_lock.contains_key(&a.id)
        })
    else { return Ok(()) };

    let unavailable: Vec<i64> = session
        .bans
        .my_team_bans
        .iter()
        .copied()
        .chain(session.bans.their_team_bans.iter().copied())
        .chain(session.their_team.iter().map(|p| p.champion_id))
        .filter(|cid| *cid != 0)
        .collect();

    let team_hovered: Vec<i64> = session
        .my_team
        .iter()
        .filter(|p| p.cell_id != session.local_player_cell_id)
        .map(|p| p.champion_id)
        .filter(|cid| *cid != 0)
        .collect();

    pending_hover.insert(action.id);

    // brief yield so rules_engine can switch profile
    tokio::select! {
        _ = shutdown.cancelled() => { pending_hover.remove(&action.id); return Ok(()); }
        _ = sleep(Duration::from_millis(50)) => {}
    }

    if context.state.read().await.settings.automation.paused {
        pending_hover.remove(&action.id);
        return Ok(());
    }

    let profile = {
        let state = context.state.read().await;
        let id = state.profiles.active_profile_id.clone();
        id.and_then(|pid| state.profiles.profiles.iter().find(|p| p.id == pid).cloned())
    };
    let Some(profile) = profile else {
        pending_hover.remove(&action.id);
        return Ok(());
    };

    let skip = failed_attempts.get(&action.id);

    let champion_id = profile
        .pick_priority
        .iter()
        .find(|entry| {
            if unavailable.contains(&entry.champion_id) {
                return false;
            }
            if !entry.ignore_teammate_hovers && team_hovered.contains(&entry.champion_id) {
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
        pending_hover.remove(&action.id);
        context.monitor(MonitorLevel::Warn, "auto-pick", "No available champion to pick".to_string());
        return Ok(());
    };

    let _ = perform_hover(context, action.id, champion_id).await;

    // user-configured delay after hover so champion is visible
    if delay_seconds > 0.0 {
        let delay_ms = {
            let half = (delay_seconds * 500.0) as u64;
            let mut rng = rand::rng();
            let jitter = rng.random_range(0..=half);
            (delay_seconds * 1000.0) as u64 - half + jitter
        };
        tokio::select! {
            _ = shutdown.cancelled() => { pending_hover.remove(&action.id); return Ok(()); }
            _ = sleep(Duration::from_millis(delay_ms)) => {}
        }
    }

    if context.state.read().await.settings.automation.paused {
        pending_hover.remove(&action.id);
        return Ok(());
    }

    match perform_pick(context, action.id, champion_id).await {
        Ok(_) => {
            pending_hover.remove(&action.id);
            pending_lock.insert(action.id, (champion_id, Instant::now()));
            processed_action_ids.insert(action.id);
            context.monitor(MonitorLevel::Info, "auto-pick", format!("Picked champion #{}", champion_id));
        }
        Err(e) => {
            pending_hover.remove(&action.id);
            context.monitor(
                MonitorLevel::Error,
                "auto-pick",
                format!("Failed to pick: {}", e),
            );
        }
    }

    Ok(())
}

async fn perform_pick(context: &Arc<AppContext>, action_id: i64, champion_id: i64) -> Result<()> {
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
