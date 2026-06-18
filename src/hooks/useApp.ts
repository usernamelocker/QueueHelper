import { useEffect, useState, useCallback } from "react";
import {
  getRuntimeSnapshot,
  getSettings,
  updateSettings,
  getProfiles,
  getMonitorEntries,
  setAutomationPaused,
  setAutoAcceptEnabled,
  setAutoBanEnabled,
  setAutoPickEnabled,
  setAutoHoverEnabled,
  setActiveProfile,
} from "../api/tauri";
import type {
  RuntimeSnapshot,
  AppSettings,
  ProfilesStore,
  MonitorEntry,
} from "../types/models";

interface AppState {
  snapshot: RuntimeSnapshot | null;
  settings: AppSettings | null;
  profiles: ProfilesStore | null;
  monitor: MonitorEntry[];
}

export function useApp() {
  const [state, setState] = useState<AppState>({
    snapshot: null,
    settings: null,
    profiles: null,
    monitor: [],
  });

  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const [snapshot, settings, profiles, monitor] = await Promise.all([
          getRuntimeSnapshot(),
          getSettings(),
          getProfiles(),
          getMonitorEntries(50),
        ]);
        setState({ snapshot, settings, profiles, monitor });
        setError(null);
      } catch {
        // not connected yet
      }
    }, 1000);
    return () => clearInterval(interval);
  }, []);

  const togglePaused = useCallback(async () => {
    if (!state.snapshot) return;
    const next = !state.snapshot.automationPaused;
    await setAutomationPaused(next).catch(() => {});
    setState((s) =>
      s.snapshot
        ? { ...s, snapshot: { ...s.snapshot, automationPaused: next } }
        : s
    );
  }, [state.snapshot]);

  const toggleAutoAccept = useCallback(async () => {
    if (!state.snapshot) return;
    const next = !state.snapshot.autoAcceptEnabled;
    await setAutoAcceptEnabled(next).catch(() => {});
    setState((s) =>
      s.snapshot
        ? { ...s, snapshot: { ...s.snapshot, autoAcceptEnabled: next } }
        : s
    );
  }, [state.snapshot]);

  const toggleAutoBan = useCallback(async () => {
    if (!state.snapshot) return;
    const next = !state.snapshot.autoBanEnabled;
    await setAutoBanEnabled(next).catch(() => {});
    setState((s) =>
      s.snapshot
        ? { ...s, snapshot: { ...s.snapshot, autoBanEnabled: next } }
        : s
    );
  }, [state.snapshot]);

  const toggleAutoPick = useCallback(async () => {
    if (!state.snapshot) return;
    const next = !state.snapshot.autoPickEnabled;
    await setAutoPickEnabled(next).catch(() => {});
    setState((s) =>
      s.snapshot
        ? { ...s, snapshot: { ...s.snapshot, autoPickEnabled: next } }
        : s
    );
  }, [state.snapshot]);

  const toggleAutoHover = useCallback(async () => {
    if (!state.snapshot) return;
    const next = !state.snapshot.autoHoverEnabled;
    await setAutoHoverEnabled(next).catch(() => {});
    setState((s) =>
      s.snapshot
        ? { ...s, snapshot: { ...s.snapshot, autoHoverEnabled: next } }
        : s
    );
  }, [state.snapshot]);

  const saveSettings = useCallback(async (settings: AppSettings) => {
    await updateSettings(settings);
    setState((s) => ({ ...s, settings }));
  }, []);

  const selectProfile = useCallback(async (profileId: string) => {
    await setActiveProfile(profileId);
  }, []);

  return {
    ...state,
    error,
    togglePaused,
    toggleAutoAccept,
    toggleAutoBan,
    toggleAutoPick,
    toggleAutoHover,
    saveSettings,
    selectProfile,
  };
}
