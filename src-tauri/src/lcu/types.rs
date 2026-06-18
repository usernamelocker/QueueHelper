use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct LockfileInfo {
    pub process_name: String,
    pub pid: u32,
    pub port: u16,
    pub password: String,
    pub protocol: String,
    pub lockfile_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ChampSelectTimer {
    #[serde(default)]
    pub phase: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ChampSelectSession {
    #[serde(default)]
    pub local_player_cell_id: i64,
    #[serde(default)]
    pub actions: Vec<Vec<ChampSelectAction>>,
    #[serde(default)]
    pub my_team: Vec<ChampSelectParticipant>,
    #[serde(default)]
    pub their_team: Vec<ChampSelectParticipant>,
    #[serde(default)]
    pub bans: ChampSelectBans,
    #[serde(default)]
    pub timer: ChampSelectTimer,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ChampSelectAction {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub actor_cell_id: i64,
    #[serde(default)]
    pub champion_id: i64,
    #[serde(default)]
    pub completed: bool,
    #[serde(default)]
    pub is_in_progress: bool,
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub pick_turn: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ChampSelectParticipant {
    #[serde(default)]
    pub cell_id: i64,
    #[serde(default)]
    pub champion_id: i64,
    #[serde(default)]
    pub assigned_position: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ChampSelectBans {
    #[serde(default)]
    pub my_team_bans: Vec<i64>,
    #[serde(default)]
    pub their_team_bans: Vec<i64>,
}

