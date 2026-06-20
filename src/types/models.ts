export type MonitorLevel = "INFO" | "WARN" | "ERROR";

export interface RuntimeSnapshot {
  connected: boolean;
  lcuPort?: number | null;
  gamePhase?: string | null;
  readyCheckState?: string | null;
  activeProfileId?: string | null;
  activeProfileName?: string | null;
  automationPaused: boolean;
  autoAcceptEnabled: boolean;
  autoBanEnabled: boolean;
  autoPickEnabled: boolean;
  autoHoverEnabled: boolean;
  lastAction?: string | null;
  currentQueueId?: number | null;
  monitorAutoScroll: boolean;
  monitorScrollTop: number;
}

export interface AppSettings {
  schemaVersion: number;
  leagueInstallPath?: string | null;
  language?: string;
  automation: AutomationSettings;
}

export interface AutomationSettings {
  paused: boolean;
  autoAccept: AutoAcceptSettings;
  autoBanEnabled: boolean;
  autoBanDelaySeconds: number;
  autoPickEnabled: boolean;
  autoPickDelaySeconds: number;
  autoHoverEnabled: boolean;
  autoHoverDelaySeconds: number;
  queueOverrides: QueueOverride[];
}

export interface QueueOverride {
  queueId: number;
  autoAcceptEnabled: boolean;
}

export interface AutoAcceptSettings {
  enabled: boolean;
  delayMinSeconds: number;
  delayMaxSeconds: number;
}

export interface ProfilesStore {
  schemaVersion: number;
  activeProfileId?: string | null;
  profiles: ChampionProfile[];
}

export interface ChampionPriorityEntry {
  championId: number;
  ignoreTeammateHovers: boolean;
  isHoverTarget: boolean;
}

export interface ChampionProfile {
  id: string;
  name: string;
  preferredRole: string;
  banPriority: ChampionPriorityEntry[];
  pickPriority: ChampionPriorityEntry[];
  requestRoleSwapWhenAutofilled: boolean;
  requestPickPosition: number;
}

export interface RulesStore {
  schemaVersion: number;
  rules: DraftRule[];
}

export interface DraftRule {
  id: string;
  enabled: boolean;
  trigger: RuleTrigger;
  action: RuleAction;
}

export interface RuleTrigger {
  event: string;
  value: unknown;
}

export interface RuleAction {
  type: string;
  params: Record<string, unknown>;
}

export interface MonitorEntry {
  id: number;
  timestamp: string;
  level: MonitorLevel;
  category: string;
  message: string;
}
