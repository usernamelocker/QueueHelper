use serde::{Deserialize, Serialize};

pub const SETTINGS_SCHEMA_VERSION: u32 = 1;
pub const PROFILES_SCHEMA_VERSION: u32 = 2;
pub const RULES_SCHEMA_VERSION: u32 = 1;

fn default_language() -> String {
    "en".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub schema_version: u32,
    pub league_install_path: Option<String>,
    #[serde(default = "default_language")]
    pub language: String,
    pub automation: AutomationSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            schema_version: SETTINGS_SCHEMA_VERSION,
            league_install_path: None,
            language: default_language(),
            automation: AutomationSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationSettings {
    pub paused: bool,
    pub auto_accept: AutoAcceptSettings,
    pub auto_ban_enabled: bool,
    #[serde(default)]
    pub auto_ban_delay_seconds: f32,
    pub auto_pick_enabled: bool,
    #[serde(default)]
    pub auto_pick_delay_seconds: f32,
    #[serde(default)]
    pub auto_hover_enabled: bool,
    #[serde(default)]
    pub auto_hover_delay_seconds: f32,
    pub queue_overrides: Vec<QueueOverride>,
}

impl Default for AutomationSettings {
    fn default() -> Self {
        Self {
            paused: false,
            auto_accept: AutoAcceptSettings::default(),
            auto_ban_enabled: false,
            auto_ban_delay_seconds: 2.0,
            auto_pick_enabled: false,
            auto_pick_delay_seconds: 0.0,
            auto_hover_enabled: false,
            auto_hover_delay_seconds: 0.0,
            queue_overrides: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueOverride {
    pub queue_id: i64,
    pub auto_accept_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoAcceptSettings {
    pub enabled: bool,
    pub delay_min_seconds: f32,
    pub delay_max_seconds: f32,
}

impl Default for AutoAcceptSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            delay_min_seconds: 1.5,
            delay_max_seconds: 3.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilesStore {
    pub schema_version: u32,
    pub active_profile_id: Option<String>,
    pub profiles: Vec<ChampionProfile>,
}

impl Default for ProfilesStore {
    fn default() -> Self {
        let default_profile = ChampionProfile {
            id: "mid-main".to_string(),
            name: "Mid Main".to_string(),
            preferred_role: "MIDDLE".to_string(),
            ban_priority: vec![
                ChampionPriorityEntry { champion_id: 157, ignore_teammate_hovers: false, is_hover_target: false },
                ChampionPriorityEntry { champion_id: 238, ignore_teammate_hovers: false, is_hover_target: false },
                ChampionPriorityEntry { champion_id: 55, ignore_teammate_hovers: false, is_hover_target: false },
            ],
            pick_priority: vec![
                ChampionPriorityEntry { champion_id: 103, ignore_teammate_hovers: false, is_hover_target: false },
                ChampionPriorityEntry { champion_id: 3, ignore_teammate_hovers: false, is_hover_target: false },
                ChampionPriorityEntry { champion_id: 1, ignore_teammate_hovers: false, is_hover_target: false },
            ],
            request_role_swap_when_autofilled: true,
            request_pick_position: 0,
        };
        Self {
            schema_version: PROFILES_SCHEMA_VERSION,
            active_profile_id: Some(default_profile.id.clone()),
            profiles: vec![default_profile],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampionPriorityEntry {
    pub champion_id: i64,
    pub ignore_teammate_hovers: bool,
    #[serde(default)]
    pub is_hover_target: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampionProfile {
    pub id: String,
    pub name: String,
    pub preferred_role: String,
    pub ban_priority: Vec<ChampionPriorityEntry>,
    pub pick_priority: Vec<ChampionPriorityEntry>,
    pub request_role_swap_when_autofilled: bool,
    pub request_pick_position: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RulesStore {
    pub schema_version: u32,
    pub rules: Vec<DraftRule>,
}

impl Default for RulesStore {
    fn default() -> Self {
        Self {
            schema_version: RULES_SCHEMA_VERSION,
            rules: vec![
                DraftRule {
                    id: "auto-switch-role".to_string(),
                    enabled: false,
                    trigger: RuleTrigger {
                        event: "champSelectStarted".to_string(),
                        value: serde_json::json!({}),
                    },
                    action: RuleAction {
                        action_type: "useProfile".to_string(),
                        params: serde_json::json!({}),
                    },
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftRule {
    pub id: String,
    pub enabled: bool,
    pub trigger: RuleTrigger,
    pub action: RuleAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleTrigger {
    pub event: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleAction {
    #[serde(rename = "type")]
    pub action_type: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MonitorLevel {
    Info,
    Warn,
    Error,
}

impl MonitorLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }

    pub fn from_db(value: &str) -> Self {
        match value {
            "WARN" => Self::Warn,
            "ERROR" => Self::Error,
            _ => Self::Info,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorEntry {
    pub id: i64,
    pub timestamp: String,
    pub level: MonitorLevel,
    pub category: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSnapshot {
    pub connected: bool,
    pub lcu_port: Option<u16>,
    pub game_phase: Option<String>,
    pub ready_check_state: Option<String>,
    pub active_profile_id: Option<String>,
    pub active_profile_name: Option<String>,
    pub automation_paused: bool,
    pub auto_accept_enabled: bool,
    pub auto_ban_enabled: bool,
    pub auto_pick_enabled: bool,
    pub auto_hover_enabled: bool,
    pub current_queue_id: Option<i64>,
    pub last_action: Option<String>,
    pub monitor_auto_scroll: bool,
    pub monitor_scroll_top: f64,
}

