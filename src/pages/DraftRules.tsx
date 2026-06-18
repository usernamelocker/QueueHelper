import { useState, useEffect, useRef } from "react";
import { getRules, updateRules, getProfiles } from "../api/tauri";
import { useTranslation } from "../i18n";
import type { RulesStore, DraftRule, ProfilesStore } from "../types/models";
import { useChampions } from "../hooks/useChampions";

const ROLES = ["TOP", "JUNGLE", "MIDDLE", "BOTTOM", "UTILITY"];

function DraftRules() {
  const { t } = useTranslation();
  const [store, setStore] = useState<RulesStore | null>(null);
  const [profilesStore, setProfilesStore] = useState<ProfilesStore | null>(null);
  const { champions } = useChampions();

  useEffect(() => {
    getRules().then(setStore).catch(console.error);
    getProfiles().then(setProfilesStore).catch(console.error);
  }, []);

  const toggleRule = async (ruleId: string) => {
    if (!store) return;
    const next: RulesStore = {
      ...store,
      rules: store.rules.map((r) =>
        r.id === ruleId ? { ...r, enabled: !r.enabled } : r
      ),
    };
    await updateRules(next);
    setStore(next);
  };

  const updateAlertRule = async (
    ruleId: string,
    patch: Partial<DraftRule>
  ) => {
    if (!store) return;
    const next: RulesStore = {
      ...store,
      rules: store.rules.map((r) =>
        r.id === ruleId ? { ...r, ...patch } : r
      ),
    };
    await updateRules(next);
    setStore(next);
  };

  const addAlertRule = async (event: "enemyPickedChampion" | "teammatePickedChampion") => {
    if (!store) return;
    const prefix = event === "enemyPickedChampion" ? "alert-enemy" : "alert-teammate";
    const id = `${prefix}-${crypto.randomUUID().slice(0, 8)}`;
    const newRule: DraftRule = {
      id,
      enabled: true,
      trigger: {
        event,
        value: { championId: 0 },
      },
      action: {
        type: "alert",
        params: { message: event === "enemyPickedChampion" ? t("rules.alertEnemyDesc") : t("rules.alertTeammateDesc") },
      },
    };
    const next: RulesStore = {
      ...store,
      rules: [...store.rules, newRule],
    };
    await updateRules(next);
    setStore(next);
  };

  const removeRule = async (ruleId: string) => {
    if (!store) return;
    const next: RulesStore = {
      ...store,
      rules: store.rules.filter((r) => r.id !== ruleId),
    };
    await updateRules(next);
    setStore(next);
  };

  if (!store) {
    return (
      <div className="flex items-center justify-center h-64">
        <p className="text-text-dim text-sm">{t("rules.loading")}</p>
      </div>
    );
  }

  const getRuleMeta = (rule: DraftRule) => {
    if (rule.id.startsWith("alert-enemy-")) {
      return {
        title: t("rules.alertEnemyTitle"),
        description: t("rules.alertEnemyDesc"),
        editable: true as const,
      };
    }
    if (rule.id.startsWith("alert-teammate-")) {
      return {
        title: t("rules.alertTeammateTitle"),
        description: t("rules.alertTeammateDesc"),
        editable: true as const,
      };
    }
    if (rule.id === "auto-switch-role") {
      return {
        title: t("rules.autoSwitchTitle"),
        description: t("rules.autoSwitchDesc"),
        editable: false as const,
      };
    }
    return {
      title: rule.id,
      description: `${rule.trigger.event} → ${rule.action.type}`,
      editable: true as const,
    };
  };

  return (
    <div className="max-w-2xl space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold text-text">{t("rules.title")}</h2>
        <div className="flex gap-2">
          <button
            onClick={() => addAlertRule("enemyPickedChampion")}
            className="px-3 py-1.5 bg-surface-3 text-text rounded-lg text-sm font-medium hover:bg-surface-3/80 transition-colors"
          >
            {t("rules.enemyPick")}
          </button>
          <button
            onClick={() => addAlertRule("teammatePickedChampion")}
            className="px-3 py-1.5 bg-surface-3 text-text rounded-lg text-sm font-medium hover:bg-surface-3/80 transition-colors"
          >
            {t("rules.teammatePick")}
          </button>
        </div>
      </div>

      {store.rules.length === 0 && (
        <p className="text-sm text-text-dim">{t("rules.noRules")}</p>
      )}

      <div className="space-y-2">
        {store.rules.map((rule) => {
          const meta = getRuleMeta(rule);
          return (
            <div
              key={rule.id}
              className="bg-surface-2 border border-border rounded-lg p-4"
            >
              <div className="flex items-start justify-between">
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    <p className="text-sm font-medium text-text">
                      {meta.title}
                    </p>
                    <span
                      className={`text-[10px] px-1.5 py-0.5 rounded ${
                        rule.enabled
                          ? "bg-green/20 text-green"
                          : "bg-surface-3 text-text-dim"
                      }`}
                    >
                      {rule.enabled ? t("rules.enabled") : t("rules.disabled")}
                    </span>
                    {meta.editable && (
                      <button
                        onClick={() => removeRule(rule.id)}
                        className="text-xs text-red hover:text-red/80 ml-auto"
                      >
                        {t("rules.remove")}
                      </button>
                    )}
                  </div>
                  <p className="text-xs text-text-dim mt-0.5">
                    {meta.description}
                  </p>

                  {rule.id === "auto-switch-role" && (
                    <EditableAutoSwitchRule
                      rule={rule}
                      profiles={profilesStore?.profiles ?? []}
                      onUpdate={(patch) =>
                        updateAlertRule(rule.id, patch)
                      }
                    />
                  )}
                  {meta.editable && rule.id.startsWith("alert-") && (
                    <EditableAlertRule
                      rule={rule}
                      champions={champions}
                      onUpdate={(patch) =>
                        updateAlertRule(rule.id, patch)
                      }
                    />
                  )}
                </div>
                <button
                  onClick={() => toggleRule(rule.id)}
                  className={`relative w-10 h-5 rounded-full transition-colors shrink-0 ml-3 ${
                    rule.enabled ? "bg-accent" : "bg-surface-3"
                  }`}
                >
                  <span
                    className={`absolute top-0.5 left-0.5 w-4 h-4 bg-white rounded-full transition-transform ${
                      rule.enabled ? "translate-x-5" : "translate-x-0"
                    }`}
                  />
                </button>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}

function EditableAlertRule({
  rule,
  champions,
  onUpdate,
}: {
  rule: DraftRule;
  champions: { key: string; name: string }[];
  onUpdate: (patch: Partial<DraftRule>) => void;
}) {
  const { t } = useTranslation();
  const championId =
    (rule.trigger.value as any)?.championId ?? 0;
  const role = (rule.trigger.value as any)?.role ?? "";
  const message =
    ((rule.action.params as any)?.message as string) ?? "";

  const isTeammate = rule.trigger.event === "teammatePickedChampion";

  const setChampionId = (id: number) => {
    const name = champions.find((c) => Number(c.key) === id)?.name ?? `#${id}`;
    onUpdate({
      trigger: { ...rule.trigger, value: { ...(rule.trigger.value as any), championId: id } },
      action: {
        ...rule.action,
        params: {
          ...(rule.action.params as any),
          message: isTeammate
            ? `Teammate picked ${name}!`
            : `Enemy picked ${name}!`,
        },
      },
    });
  };

  const setRole = (r: string) => {
    onUpdate({
      trigger: { ...rule.trigger, value: { ...(rule.trigger.value as any), role: r || undefined } },
    });
  };

  const setMessage = (msg: string) => {
    onUpdate({
      action: { ...rule.action, params: { ...(rule.action.params as any), message: msg } },
    });
  };

  return (
    <div className="mt-3 space-y-2">
      <div className="flex items-center gap-3">
        <div className="flex-1">
          <label className="text-[10px] text-text-dim block mb-0.5">
            {t("rules.champion")}
          </label>
          <ChampionSelect
            championId={championId}
            champions={champions}
            onChange={setChampionId}
          />
        </div>
        {isTeammate && (
          <div>
            <label className="text-[10px] text-text-dim block mb-0.5">
              {t("rules.roleOptional")}
            </label>
            <select
              value={role}
              onChange={(e) => setRole(e.target.value)}
              className="bg-surface-3 border border-border rounded px-2 py-1.5 text-sm text-text outline-none focus:border-accent"
            >
              <option value="">{t("rules.anyRole")}</option>
              {ROLES.map((r) => (
                <option key={r} value={r}>
                  {t(`role.${r}` as any)}
                </option>
              ))}
            </select>
          </div>
        )}
      </div>
      <div>
        <label className="text-[10px] text-text-dim block mb-0.5">
          {t("rules.alertMessage")}
        </label>
        <input
          type="text"
          value={message}
          onChange={(e) => setMessage(e.target.value)}
          className="w-full bg-surface-3 border border-border rounded px-2 py-1.5 text-sm text-text outline-none focus:border-accent"
        />
      </div>
    </div>
  );
}

function EditableAutoSwitchRule({
  rule,
  profiles,
  onUpdate,
}: {
  rule: DraftRule;
  profiles: { id: string; name: string; preferredRole: string }[];
  onUpdate: (patch: Partial<DraftRule>) => void;
}) {
  const { t } = useTranslation();
  const roleProfileMap: Record<string, string> =
    ((rule.action.params as any)?.roleProfileMap as Record<string, string>) ?? {};

  const setProfileForRole = (role: string, profileId: string) => {
    const next = { ...roleProfileMap };
    if (profileId) {
      next[role] = profileId;
    } else {
      delete next[role];
    }
    onUpdate({
      action: {
        ...rule.action,
        params: { roleProfileMap: next },
      },
    });
  };

  const roleLabel = (r: string) => {
    const key = `role.${r}` as any;
    const val = t(key);
    return val !== key ? val : r;
  };

  return (
    <div className="mt-3 space-y-2">
      <p className="text-[10px] text-text-dim">{t("rules.mapRolesToProfiles")}</p>
      {ROLES.map((role) => {
        const selectedId = roleProfileMap[role] ?? "";
        return (
          <div key={role} className="flex items-center gap-2">
            <span className="text-xs text-text w-16 shrink-0">{roleLabel(role)}</span>
            <select
              value={selectedId}
              onChange={(e) => setProfileForRole(role, e.target.value)}
              className="flex-1 bg-surface-3 border border-border rounded px-2 py-1.5 text-sm text-text outline-none focus:border-accent"
            >
              <option value="">{t("rules.autoDetect")}</option>
              {profiles.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name || t("profiles.unnamed")} ({roleLabel(p.preferredRole)})
                </option>
              ))}
            </select>
          </div>
        );
      })}
    </div>
  );
}

function ChampionSelect({
  championId,
  champions,
  onChange,
}: {
  championId: number;
  champions: { key: string; name: string }[];
  onChange: (id: number) => void;
}) {
  const [query, setQuery] = useState("");
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, []);

  const selected = champions.find((c) => Number(c.key) === championId);

  const filtered = query
    ? champions.filter(
        (c) =>
          c.name.toLowerCase().includes(query.toLowerCase()) ||
          c.key.includes(query)
      )
    : champions;

  return (
    <div ref={ref} className="relative">
      <input
        type="text"
        value={selected && !open ? selected.name : query}
        onChange={(e) => {
          setQuery(e.target.value);
          setOpen(true);
        }}
        onFocus={() => {
          setQuery("");
          setOpen(true);
        }}
        placeholder="Search champion..."
        className="w-full bg-surface-3 border border-border rounded px-2 py-1.5 text-sm text-text placeholder:text-text-dim/40 outline-none focus:border-accent"
      />
      {open && (
        <div className="absolute top-full left-0 right-0 mt-1 bg-surface-2 border border-border rounded max-h-48 overflow-y-auto z-10">
          {filtered.length === 0 ? (
            <p className="px-3 py-2 text-xs text-text-dim">No results</p>
          ) : (
            filtered.map((c) => (
              <button
                key={c.key}
                onClick={() => {
                  onChange(Number(c.key));
                  setQuery("");
                  setOpen(false);
                }}
                className="w-full text-left px-3 py-1.5 text-sm text-text hover:bg-surface-3 transition-colors"
              >
                {c.name}
              </button>
            ))
          )}
        </div>
      )}
    </div>
  );
}

export default DraftRules;
