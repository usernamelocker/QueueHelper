use crate::{
    lcu::types::{ChampSelectSession, LockfileInfo},
    models::{AppSettings, MonitorLevel, ProfilesStore, RulesStore},
};

#[derive(Debug, Clone)]
pub enum AppEvent {
    LockfileDetected(LockfileInfo),
    LockfileMissing,
    ClientConnected { port: u16 },
    ClientDisconnected,
    GameflowPhaseUpdated { phase: String },
    ReadyCheckUpdated { state: String, queue_id: Option<i64> },
    ChampSelectSessionUpdated { session: ChampSelectSession },
    SettingsUpdated(AppSettings),
    ProfilesUpdated(ProfilesStore),
    RulesUpdated(RulesStore),
    Monitor {
        level: MonitorLevel,
        category: String,
        message: String,
    },
}

