use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use anyhow::Result;

use crate::{core::{events::AppEvent, state::RuntimeState, event_bus::EventBus}, lcu::http::LcuHttpClient, persistence::{json_store::JsonStore, monitor_db::MonitorDb}, models::{AppSettings, ProfilesStore, RulesStore, MonitorLevel}};

#[derive(Clone)]
pub struct AppContext {
    pub bus: EventBus,
    pub state: Arc<RwLock<RuntimeState>>,
    pub lcu_client: Arc<RwLock<Option<LcuHttpClient>>>,
    pub settings_store: JsonStore<AppSettings>,
    pub profiles_store: JsonStore<ProfilesStore>,
    pub rules_store: JsonStore<RulesStore>,
    pub monitor_db: MonitorDb,
}

/* Context Singleton */
impl AppContext {
    pub async fn new(data_dir: PathBuf) -> Result<Self> {
        let settings_store = JsonStore::new(data_dir.join("settings.json"));
        let profiles_store = JsonStore::new(data_dir.join("profiles.json"));
        let rules_store = JsonStore::new(data_dir.join("rules.json"));
        let monitor_db = MonitorDb::new(data_dir.join("monitor.db"))?;

        let bus = EventBus::new(256);

        let runtime_state = RuntimeState::new(
            settings_store.load_or_default_with_expected_version(crate::models::SETTINGS_SCHEMA_VERSION).await?,
            profiles_store.load_or_default_with_expected_version(crate::models::PROFILES_SCHEMA_VERSION).await?,
            rules_store.load_or_default_with_expected_version(crate::models::RULES_SCHEMA_VERSION).await.unwrap_or_default(),
        );

        Ok(Self {
            bus,
            state: Arc::new(RwLock::new(runtime_state)),
            lcu_client: Arc::new(RwLock::new(None)),
            settings_store,
            profiles_store,
            rules_store,
            monitor_db,
        })
    }

    /* Helper */
    pub async fn set_lcu_client(&self, client: Option<LcuHttpClient>) {
        *self.lcu_client.write().await = client;
    }

    pub fn monitor(&self, level: MonitorLevel, category: &str, message: String) {
        self.bus.publish(AppEvent::Monitor {
            level,
            category: category.to_string(),
            message,
        });
    }
}
