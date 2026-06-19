use std::sync::Arc;
use std::collections::HashSet;

use tokio_util::sync::CancellationToken;

use crate::{
    app::AppContext,
    core::events::AppEvent,
    models::{DraftRule, MonitorLevel, RuleAction},
    lcu::types::ChampSelectSession,
};

pub async fn run_rules_engine(context: Arc<AppContext>, shutdown: CancellationToken) {
    let mut receiver = context.bus.subscribe();
    let mut completed_action_ids: HashSet<i64> = HashSet::new();
    let mut has_entered_champ_select = false;

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => break,
            event_result = receiver.recv() => {
                let Ok(event) = event_result else { continue };

                match event {
                    AppEvent::ChampSelectSessionUpdated { session } => {
                        let paused = {
                            let state = context.state.read().await;
                            state.settings.automation.paused
                        };
                        if paused { continue }

                        let rules = {
                            let state = context.state.read().await;
                            state.rules.rules.clone()
                        };

                        let has_actions = session.actions.iter().any(|a| !a.is_empty());

                        if has_actions && !has_entered_champ_select {
                            has_entered_champ_select = true;
                            process_auto_switch(&context, &session, &rules).await;
                        } else if !has_actions {
                            has_entered_champ_select = false;
                            completed_action_ids.clear();
                        }

                        if has_actions {
                            let new_picks: Vec<(i64, i64, i64)> = session
                                .actions
                                .iter()
                                .flatten()
                                .filter(|a| a.r#type == "pick" && a.completed && a.champion_id != 0 && !completed_action_ids.contains(&a.id))
                                .map(|a| (a.id, a.champion_id, a.actor_cell_id))
                                .collect();

                            for (action_id, champion_id, actor_cell_id) in &new_picks {
                                completed_action_ids.insert(*action_id);

                                let is_teammate = session.my_team.iter().any(|p| p.cell_id == *actor_cell_id);
                                let triggered: Vec<DraftRule> = rules.iter()
                                    .filter(|r| r.enabled)
                                    .filter(|r| {
                                        let event_match = if is_teammate {
                                            r.trigger.event == "teammatePickedChampion"
                                        } else {
                                            r.trigger.event == "enemyPickedChampion"
                                        };
                                        event_match && matches_champion_rule(&r.trigger.value, *champion_id, &session, *actor_cell_id)
                                    })
                                    .cloned()
                                    .collect();

                                for rule in triggered {
                                    execute_action(&context, &rule.action, *champion_id).await;
                                }
                            }
                        }
                    }
                    AppEvent::GameflowPhaseUpdated { phase } => {
                        if phase != "ChampSelect" && !phase.contains("Game") {
                            has_entered_champ_select = false;
                            completed_action_ids.clear();
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn matches_champion_rule(trigger_value: &serde_json::Value, champion_id: i64, session: &ChampSelectSession, actor_cell_id: i64) -> bool {
    let Some(cid) = trigger_value.get("championId").and_then(|v| v.as_i64()) else {
        return false;
    };
    if cid != champion_id {
        return false;
    }
    if let Some(role) = trigger_value.get("role").and_then(|v| v.as_str()) {
        if let Some(participant) = session.my_team.iter().chain(session.their_team.iter()).find(|p| p.cell_id == actor_cell_id) {
            let assigned = participant.assigned_position.to_uppercase();
            if assigned != role.to_uppercase() {
                return false;
            }
        }
    }
    true
}

async fn process_auto_switch(context: &Arc<AppContext>, session: &ChampSelectSession, rules: &[DraftRule]) {
    let switch_rule = rules.iter().find(|r| r.enabled && r.trigger.event == "champSelectStarted");
    let Some(switch_rule) = switch_rule else { return };
    if switch_rule.action.action_type != "useProfile" { return }

    let local_cell_id = session.local_player_cell_id;
    let assigned_role = session
        .my_team
        .iter()
        .find(|p| p.cell_id == local_cell_id)
        .map(|p| p.assigned_position.to_uppercase());

    let Some(assigned_role) = assigned_role else { return };
    if assigned_role.is_empty() { return }

    let (active_id, profiles) = {
        let state = context.state.read().await;
        (state.profiles.active_profile_id.clone(), state.profiles.profiles.clone())
    };

    // Check roleProfileMap first (manual binding from the UI)
    let role_profile_map: Option<serde_json::Map<String, serde_json::Value>> = switch_rule
        .action
        .params
        .get("roleProfileMap")
        .and_then(|v| v.as_object().cloned());

    let mapped_profile_id = role_profile_map.as_ref().and_then(|map| {
        map.get(&assigned_role).and_then(|v| v.as_str().map(String::from))
    });

    // If we have a mapped profile, use it
    if let Some(profile_id) = &mapped_profile_id {
        if active_id.as_ref() == Some(profile_id) {
            return;
        }
        let has_profile = profiles.iter().any(|p| &p.id == profile_id);
        if has_profile {
            {
                let mut state = context.state.write().await;
                state.profiles.active_profile_id = Some(profile_id.clone());
            }
            let profile_name = profiles.iter().find(|p| &p.id == profile_id).map(|p| p.name.as_str()).unwrap_or("Unnamed");
            persist_active_profile(context).await;
            context.monitor(
                MonitorLevel::Info,
                "rules-engine",
                format!("Switched to profile '{}' for role {} (manual map)", profile_name, assigned_role),
            );
            return;
        }
    }

    // Fall back to auto-detect by preferredRole
    let active_preferred = active_id.as_ref().and_then(|id| {
        profiles.iter().find(|p| &p.id == id).map(|p| p.preferred_role.as_str())
    });

    if active_preferred == Some(assigned_role.as_str()) {
        return;
    }

    let target = profiles.iter().find(|p| p.preferred_role.eq_ignore_ascii_case(&assigned_role));

    if let Some(target) = target {
        {
            let mut state = context.state.write().await;
            state.profiles.active_profile_id = Some(target.id.clone());
        }
        persist_active_profile(context).await;
        context.monitor(
            MonitorLevel::Info,
            "rules-engine",
            format!("Auto-switched to profile '{}' for role {}", target.name, assigned_role),
        );
    }
}

async fn persist_active_profile(context: &Arc<AppContext>) {
    let profiles = {
        let state = context.state.read().await;
        state.profiles.clone()
    };
    let _ = context.profiles_store.save(&profiles).await;
}

async fn execute_action(context: &Arc<AppContext>, action: &RuleAction, champion_id: i64) {
    match action.action_type.as_str() {
        "alert" => {
            let default_msg = format!("Champion {} was picked", champion_id);
            let msg = action.params.get("message")
                .and_then(|v| v.as_str())
                .unwrap_or(&default_msg);
            context.monitor(MonitorLevel::Warn, "rules-engine", msg.to_string());
        }
        _ => {}
    }
}
