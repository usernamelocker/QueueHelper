import { createContext, useContext, useEffect, useState, useCallback } from "react";
import {
  getRuntimeSnapshot,
  getSettings,
  updateSettings,
  getProfiles,
  getRules,
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
  RulesStore,
  MonitorEntry,
} from "../types/models";

interface AppState {
  snapshot: RuntimeSnapshot | null;
  settings: AppSettings | null;
  profiles: ProfilesStore | null;
  rules: RulesStore | null;
  monitor: MonitorEntry[];
  error: string | null;
}

interface AppContextValue extends AppState {
  togglePaused: () => Promise<void>;
  toggleAutoAccept: () => Promise<void>;
  toggleAutoBan: () => Promise<void>;
  toggleAutoPick: () => Promise<void>;
  toggleAutoHover: () => Promise<void>;
  saveSettings: (settings: AppSettings) => Promise<void>;
  selectProfile: (profileId: string) => Promise<void>;
}

const Ctx = createContext<AppContextValue | null>(null);

export function AppProvider({ children }: { children: React.ReactNode }) {
  const [state, setState] = useState<AppState>({
    snapshot: null,
    settings: null,
    profiles: null,
    rules: null,
    monitor: [],
    error: null,
  });
  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const [snapshot, settings, profiles, rules, monitor] = await Promise.all([
          getRuntimeSnapshot(),
          getSettings(),
          getProfiles(),
          getRules(),
          getMonitorEntries(50),
        ]);
        setState({ snapshot, settings, profiles, rules, monitor, error: null });
      } catch {
        // not connected yet
      }
    }, 1000);
    return () => clearInterval(interval);
  }, []);

  const togglePaused = useCallback(async () => {
    if (!state.snapshot) return;
    try {
      const snapshot = await setAutomationPaused(!state.snapshot.automationPaused);
      setState((s) => ({ ...s, snapshot }));
    } catch {}
  }, [state.snapshot]);

  const toggleAutoAccept = useCallback(async () => {
    if (!state.snapshot) return;
    try {
      const snapshot = await setAutoAcceptEnabled(!state.snapshot.autoAcceptEnabled);
      setState((s) => ({ ...s, snapshot }));
    } catch {}
  }, [state.snapshot]);

  const toggleAutoBan = useCallback(async () => {
    if (!state.snapshot) return;
    try {
      const snapshot = await setAutoBanEnabled(!state.snapshot.autoBanEnabled);
      setState((s) => ({ ...s, snapshot }));
    } catch {}
  }, [state.snapshot]);

  const toggleAutoPick = useCallback(async () => {
    if (!state.snapshot) return;
    try {
      const snapshot = await setAutoPickEnabled(!state.snapshot.autoPickEnabled);
      setState((s) => ({ ...s, snapshot }));
    } catch {}
  }, [state.snapshot]);

  const toggleAutoHover = useCallback(async () => {
    if (!state.snapshot) return;
    try {
      const snapshot = await setAutoHoverEnabled(!state.snapshot.autoHoverEnabled);
      setState((s) => ({ ...s, snapshot }));
    } catch {}
  }, [state.snapshot]);

  const saveSettings = useCallback(async (settings: AppSettings) => {
    await updateSettings(settings);
    setState((s) => ({ ...s, settings }));
  }, []);

  const selectProfile = useCallback(async (profileId: string) => {
    const id = await setActiveProfile(profileId);
    setState((s) => {
      if (!s.snapshot) return s;
      return {
        ...s,
        snapshot: { ...s.snapshot, activeProfileId: id },
      };
    });
  }, []);

  return (
    <Ctx.Provider
      value={{
        ...state,
        togglePaused,
        toggleAutoAccept,
        toggleAutoBan,
        toggleAutoPick,
        toggleAutoHover,
        saveSettings,
        selectProfile,
      }}
    >
      {children}
    </Ctx.Provider>
  );
}

export function useApp() {
  const ctx = useContext(Ctx);
  if (!ctx) throw new Error("useApp must be used within AppProvider");
  return ctx;
}
