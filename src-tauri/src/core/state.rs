use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use crate::{
    app::AppContext,
    core::events::AppEvent,
    lcu::types::ChampSelectSession,
    models::{AppSettings, ProfilesStore, RulesStore, RuntimeSnapshot},
};

#[derive(Debug, Clone)]
pub struct RuntimeState {
    pub connected: bool,
    pub lcu_port: Option<u16>,
    pub game_phase: Option<String>,
    pub ready_check_state: Option<String>,
    pub champ_select_session: Option<ChampSelectSession>,
    pub settings: AppSettings,
    pub profiles: ProfilesStore,
    pub rules: RulesStore,
    pub last_action: Option<String>,
    pub current_queue_id: Option<i64>,
}

impl RuntimeState {
    pub fn new(settings: AppSettings, profiles: ProfilesStore, rules: RulesStore) -> Self {
        Self {
            connected: false,
            lcu_port: None,
            game_phase: None,
            ready_check_state: None,
            champ_select_session: None,
            settings,
            profiles,
            rules,
            last_action: None,
            current_queue_id: None,
        }
    }

    pub fn snapshot(&self) -> RuntimeSnapshot {
        let active_profile_id = self.profiles.active_profile_id.clone();
        let active_profile_name = active_profile_id.as_ref().and_then(|profile_id| {
            self.profiles
                .profiles
                .iter()
                .find(|profile| &profile.id == profile_id)
                .map(|profile| profile.name.clone())
        });

        RuntimeSnapshot {
            connected: self.connected,
            lcu_port: self.lcu_port,
            game_phase: self.game_phase.clone(),
            ready_check_state: self.ready_check_state.clone(),
            active_profile_id,
            active_profile_name,
            automation_paused: self.settings.automation.paused,
            auto_accept_enabled: self.settings.automation.auto_accept.enabled,
            auto_ban_enabled: self.settings.automation.auto_ban_enabled,
            auto_pick_enabled: self.settings.automation.auto_pick_enabled,
            auto_hover_enabled: self.settings.automation.auto_hover_enabled,
            last_action: self.last_action.clone(),
            current_queue_id: self.current_queue_id,
        }
    }
}

pub async fn run_state_reducer(context: Arc<AppContext>, shutdown: CancellationToken) {
    let mut receiver = context.bus.subscribe();

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => break,
            event_result = receiver.recv() => {
                let Ok(event) = event_result else {
                    continue;
                };
                apply_event(&context, event).await;
            }
        }
    }
}

async fn apply_event(context: &Arc<AppContext>, event: AppEvent) {
    match event {
        AppEvent::ClientConnected { port } => {
            let mut state = context.state.write().await;
            state.connected = true;
            state.lcu_port = Some(port);
        }
        AppEvent::ClientDisconnected => {
            let mut state = context.state.write().await;
            state.connected = false;
            state.lcu_port = None;
            state.game_phase = None;
            state.ready_check_state = None;
            state.champ_select_session = None;
        }
        AppEvent::GameflowPhaseUpdated { phase } => {
            let mut state = context.state.write().await;
            state.game_phase = Some(phase);
        }
        AppEvent::ReadyCheckUpdated { state: ready_check_state, queue_id } => {
            let mut state = context.state.write().await;
            state.ready_check_state = Some(ready_check_state);
            state.current_queue_id = queue_id;
        }
        AppEvent::ChampSelectSessionUpdated { session } => {
            let mut state = context.state.write().await;
            state.champ_select_session = Some(session);
        }
        AppEvent::SettingsUpdated(settings) => {
            let mut state = context.state.write().await;
            state.settings = settings;
        }
        AppEvent::ProfilesUpdated(profiles) => {
            let mut state = context.state.write().await;
            state.profiles = profiles;
        }
        AppEvent::RulesUpdated(rules) => {
            let mut state = context.state.write().await;
            state.rules = rules;
        }
        AppEvent::Monitor {
            level,
            category,
            message,
        } => {
            let _ = context.monitor_db.append(level, &category, &message);
            let mut state = context.state.write().await;
            state.last_action = Some(message);
        }
        AppEvent::LockfileDetected(_) | AppEvent::LockfileMissing => {}
    }
}

