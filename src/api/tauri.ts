import { invoke } from "@tauri-apps/api/core";
import {
  RuntimeSnapshot,
  AppSettings,
  ProfilesStore,
  RulesStore,
  MonitorEntry,
} from "../types/models";

export async function getRuntimeSnapshot(): Promise<RuntimeSnapshot> {
  return invoke("get_runtime_snapshot");
}

export async function getSettings(): Promise<AppSettings> {
  return invoke("get_settings");
}

export async function updateSettings(settings: AppSettings): Promise<void> {
  return invoke("update_settings", { settings });
}

export async function getProfiles(): Promise<ProfilesStore> {
  return invoke("get_profiles");
}

export async function updateProfiles(profiles: ProfilesStore): Promise<void> {
  return invoke("update_profiles", { profiles });
}

export async function getRules(): Promise<RulesStore> {
  return invoke("get_rules");
}

export async function updateRules(rules: RulesStore): Promise<void> {
  return invoke("update_rules", { rules });
}

export async function getMonitorEntries(limit?: number): Promise<MonitorEntry[]> {
  return invoke("get_monitor_entries", { limit: limit ?? 100 });
}

export async function setAutomationPaused(paused: boolean): Promise<RuntimeSnapshot> {
  return invoke("set_automation_paused", { paused });
}

export async function setAutoAcceptEnabled(enabled: boolean): Promise<RuntimeSnapshot> {
  return invoke("set_auto_accept_enabled", { enabled });
}

export async function setAutoBanEnabled(enabled: boolean): Promise<RuntimeSnapshot> {
  return invoke("set_auto_ban_enabled", { enabled });
}

export async function setAutoPickEnabled(enabled: boolean): Promise<RuntimeSnapshot> {
  return invoke("set_auto_pick_enabled", { enabled });
}

export async function setAutoHoverEnabled(enabled: boolean): Promise<RuntimeSnapshot> {
  return invoke("set_auto_hover_enabled", { enabled });
}

export async function setActiveProfile(profileId: string): Promise<string> {
  return invoke("set_active_profile", { profileId });
}

export async function setMonitorAutoScroll(enabled: boolean): Promise<RuntimeSnapshot> {
  return invoke("set_monitor_auto_scroll", { enabled });
}

export async function setMonitorScrollTop(top: number): Promise<void> {
  return invoke("set_monitor_scroll_top", { top });
}

export async function getMonitorScrollTop(): Promise<number> {
  return invoke("get_monitor_scroll_top");
}
