use std::sync::Arc;
use tauri::State;
use anyhow::Result;
use crate::{app::AppContext, core::events::AppEvent, models::{AppSettings, ProfilesStore, RulesStore, RuntimeSnapshot, MonitorEntry}};

#[tauri::command]
pub async fn get_runtime_snapshot(context: State<'_, Arc<AppContext>>) -> Result<RuntimeSnapshot, String> {
    let state = context.state.read().await;
    Ok(state.snapshot())
}

#[tauri::command]
pub async fn get_settings(context: State<'_, Arc<AppContext>>) -> Result<AppSettings, String> {
    let state = context.state.read().await;
    Ok(state.settings.clone())
}

#[tauri::command]
pub async fn update_settings(context: State<'_, Arc<AppContext>>, settings: AppSettings) -> Result<(), String> {
    context.settings_store.save(&settings).await.map_err(|e| e.to_string())?;
    context.bus.publish(AppEvent::SettingsUpdated(settings));
    Ok(())
}

#[tauri::command]
pub async fn get_profiles(context: State<'_, Arc<AppContext>>) -> Result<ProfilesStore, String> {
    let state = context.state.read().await;
    Ok(state.profiles.clone())
}

#[tauri::command]
pub async fn update_profiles(context: State<'_, Arc<AppContext>>, profiles: ProfilesStore) -> Result<(), String> {
    context.profiles_store.save(&profiles).await.map_err(|e| e.to_string())?;
    context.bus.publish(AppEvent::ProfilesUpdated(profiles));
    Ok(())
}

#[tauri::command]
pub async fn get_rules(context: State<'_, Arc<AppContext>>) -> Result<RulesStore, String> {
    let state = context.state.read().await;
    Ok(state.rules.clone())
}

#[tauri::command]
pub async fn update_rules(context: State<'_, Arc<AppContext>>, rules: RulesStore) -> Result<(), String> {
    context.rules_store.save(&rules).await.map_err(|e| e.to_string())?;
    context.bus.publish(AppEvent::RulesUpdated(rules));
    Ok(())
}

#[tauri::command]
pub async fn get_monitor_entries(context: State<'_, Arc<AppContext>>, limit: Option<u64>) -> Result<Vec<MonitorEntry>, String> {
    let limit = limit.unwrap_or(100) as usize;
    context.monitor_db.list_recent(limit).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_automation_paused(context: State<'_, Arc<AppContext>>, paused: bool) -> Result<RuntimeSnapshot, String> {
    let (snapshot, settings) = {
        let mut state = context.state.write().await;
        state.settings.automation.paused = paused;
        (state.snapshot(), state.settings.clone())
    };
    context.settings_store.save(&settings).await.map_err(|e| e.to_string())?;
    Ok(snapshot)
}

#[tauri::command]
pub async fn set_auto_accept_enabled(context: State<'_, Arc<AppContext>>, enabled: bool) -> Result<RuntimeSnapshot, String> {
    let (snapshot, settings) = {
        let mut state = context.state.write().await;
        state.settings.automation.auto_accept.enabled = enabled;
        (state.snapshot(), state.settings.clone())
    };
    context.settings_store.save(&settings).await.map_err(|e| e.to_string())?;
    Ok(snapshot)
}

#[tauri::command]
pub async fn set_auto_ban_enabled(context: State<'_, Arc<AppContext>>, enabled: bool) -> Result<RuntimeSnapshot, String> {
    let (snapshot, settings) = {
        let mut state = context.state.write().await;
        state.settings.automation.auto_ban_enabled = enabled;
        (state.snapshot(), state.settings.clone())
    };
    context.settings_store.save(&settings).await.map_err(|e| e.to_string())?;
    Ok(snapshot)
}

#[tauri::command]
pub async fn set_auto_pick_enabled(context: State<'_, Arc<AppContext>>, enabled: bool) -> Result<RuntimeSnapshot, String> {
    let (snapshot, settings) = {
        let mut state = context.state.write().await;
        state.settings.automation.auto_pick_enabled = enabled;
        (state.snapshot(), state.settings.clone())
    };
    context.settings_store.save(&settings).await.map_err(|e| e.to_string())?;
    Ok(snapshot)
}

#[tauri::command]
pub async fn set_auto_hover_enabled(context: State<'_, Arc<AppContext>>, enabled: bool) -> Result<RuntimeSnapshot, String> {
    let (snapshot, settings) = {
        let mut state = context.state.write().await;
        state.settings.automation.auto_hover_enabled = enabled;
        (state.snapshot(), state.settings.clone())
    };
    context.settings_store.save(&settings).await.map_err(|e| e.to_string())?;
    Ok(snapshot)
}

#[tauri::command]
pub async fn set_active_profile(context: State<'_, Arc<AppContext>>, profile_id: String) -> Result<String, String> {
    let profiles = {
        let mut state = context.state.write().await;
        state.profiles.active_profile_id = Some(profile_id.clone());
        state.profiles.clone()
    };
    context.profiles_store.save(&profiles).await.map_err(|e| e.to_string())?;
    Ok(profile_id)
}
