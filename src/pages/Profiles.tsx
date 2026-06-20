import { useState } from "react";
import { updateProfiles } from "../api/tauri";
import { useApp } from "../hooks/AppContext";
import { useTranslation } from "../i18n";
import type { ProfilesStore, ChampionProfile, ChampionPriorityEntry } from "../types/models";
import ChampionPicker from "../components/ChampionPicker";

const ROLES = ["TOP", "JUNGLE", "MIDDLE", "BOTTOM", "UTILITY"];

function emptyProfile(): ChampionProfile {
  return {
    id: crypto.randomUUID(),
    name: "",
    preferredRole: "MIDDLE",
    banPriority: [] as ChampionPriorityEntry[],
    pickPriority: [] as ChampionPriorityEntry[],
    requestRoleSwapWhenAutofilled: true,
    requestPickPosition: 0,
  };
}

function Profiles() {
  const { t } = useTranslation();
  const app = useApp();
  const [store, setStore] = useState<ProfilesStore | null>(app.profiles);
  const [editing, setEditing] = useState<string | null>(null);
  const [draft, setDraft] = useState<ChampionProfile | null>(null);

  const save = async (profile: ChampionProfile) => {
    if (!store) return;
    const next: ProfilesStore = { ...store };
    const idx = next.profiles.findIndex((p) => p.id === profile.id);
    if (idx >= 0) {
      next.profiles[idx] = profile;
    } else {
      next.profiles.push(profile);
    }
    await updateProfiles(next);
    setStore(next);
    setEditing(null);
    setDraft(null);
  };

  const remove = async (id: string) => {
    if (!store) return;
    const next: ProfilesStore = {
      ...store,
      profiles: store.profiles.filter((p) => p.id !== id),
    };
    if (next.activeProfileId === id) {
      next.activeProfileId = next.profiles[0]?.id ?? null;
    }
    await updateProfiles(next);
    setStore(next);
  };

  const startEdit = (profile: ChampionProfile) => {
    setDraft({ ...profile });
    setEditing(profile.id);
  };

  const startNew = () => {
    const p = emptyProfile();
    setDraft(p);
    setEditing("__new__");
  };

  const setActive = async (id: string) => {
    if (!store) return;
    const next: ProfilesStore = { ...store, activeProfileId: id };
    await updateProfiles(next);
    setStore(next);
  };

  if (!store) {
    return (
      <div className="flex items-center justify-center h-64">
        <p className="text-text-dim text-sm">{t("profiles.loading")}</p>
      </div>
    );
  }

  const roleLabel = (r: string) => {
    const key = `role.${r}` as any;
    const val = t(key);
    return val !== key ? val : r;
  };

  return (
    <div className="max-w-2xl space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold text-text">{t("profiles.title")}</h2>
        <button
          onClick={startNew}
          className="px-3 py-1.5 bg-accent text-white rounded-lg text-sm font-medium hover:bg-accent/90 transition-colors"
        >
          {t("profiles.newProfile")}
        </button>
      </div>

      {store.profiles.length === 0 && (
        <p className="text-sm text-text-dim">{t("profiles.noProfiles")}</p>
      )}

      {editing === "__new__" && draft && (
        <ProfileForm
          draft={draft}
          setDraft={setDraft}
          onSave={() => save(draft)}
          onCancel={() => {
            setEditing(null);
            setDraft(null);
          }}
        />
      )}

      {store.profiles.map((profile) =>
        editing === profile.id && draft ? (
          <ProfileForm
            key={profile.id}
            draft={draft}
            setDraft={setDraft}
            onSave={() => save(draft)}
            onCancel={() => {
              setEditing(null);
              setDraft(null);
            }}
          />
        ) : (
          <div
            key={profile.id}
            className={`bg-surface-2 border rounded-lg p-4 ${
              store.activeProfileId === profile.id
                ? "border-accent"
                : "border-border"
            }`}
          >
            <div className="flex items-center justify-between">
              <div>
                <div className="flex items-center gap-2">
                  <p className="text-sm font-medium text-text">
                    {profile.name || t("profiles.unnamed")}
                  </p>
                  {store.activeProfileId === profile.id && (
                    <span className="text-[10px] bg-accent/20 text-accent px-1.5 py-0.5 rounded">
                      {t("profiles.active")}
                    </span>
                  )}
                </div>
                <p className="text-xs text-text-dim mt-0.5">
                  {roleLabel(profile.preferredRole)} ·{" "}
                  {t("profiles.bans", { n: profile.banPriority.length })} ·{" "}
                  {t("profiles.picks", { n: profile.pickPriority.length })}
                </p>
              </div>
              <div className="flex items-center gap-2">
                <button
                  onClick={() => setActive(profile.id)}
                  className="text-xs text-accent hover:text-accent/80"
                >
                  {t("profiles.setActive")}
                </button>
                <button
                  onClick={() => startEdit(profile)}
                  className="text-xs text-text-dim hover:text-text"
                >
                  {t("profiles.edit")}
                </button>
                <button
                  onClick={() => remove(profile.id)}
                  className="text-xs text-red hover:text-red/80"
                >
                  {t("profiles.delete")}
                </button>
              </div>
            </div>
          </div>
        )
      )}
    </div>
  );
}

const PICK_POSITIONS = [
  { value: 0, key: "pickPos.0" as const },
  { value: 1, key: "pickPos.1" as const },
  { value: 2, key: "pickPos.2" as const },
  { value: 3, key: "pickPos.3" as const },
  { value: 4, key: "pickPos.4" as const },
  { value: 5, key: "pickPos.5" as const },
];

function ProfileForm({
  draft,
  setDraft,
  onSave,
  onCancel,
}: {
  draft: ChampionProfile;
  setDraft: (p: ChampionProfile) => void;
  onSave: () => void;
  onCancel: () => void;
}) {
  const { t } = useTranslation();

  const roleLabel = (r: string) => {
    const key = `role.${r}` as any;
    const val = t(key);
    return val !== key ? val : r;
  };

  return (
    <div className="bg-surface-2 border border-border rounded-lg p-4 space-y-4">
      <div className="flex gap-4">
        <div className="flex-1">
          <label className="text-xs text-text-dim block mb-1">{t("profiles.name")}</label>
          <input
            type="text"
            value={draft.name}
            onChange={(e) => setDraft({ ...draft, name: e.target.value })}
            placeholder={t("profiles.unnamed")}
            className="w-full bg-surface-3 border border-border rounded px-3 py-2 text-sm text-text placeholder:text-text-dim/40 outline-none focus:border-accent"
          />
        </div>
        <div>
          <label className="text-xs text-text-dim block mb-1">
            {t("profiles.preferredRole")}
          </label>
          <select
            value={draft.preferredRole}
            onChange={(e) =>
              setDraft({ ...draft, preferredRole: e.target.value })
            }
            className="bg-surface-3 border border-border rounded px-3 py-2 text-sm text-text outline-none focus:border-accent"
          >
            {ROLES.map((r) => (
              <option key={r} value={r}>
                {roleLabel(r)}
              </option>
            ))}
          </select>
        </div>
      </div>

      <ChampionPicker
        label={t("profiles.banPriority")}
        selected={draft.banPriority}
        onChange={(ids) => setDraft({ ...draft, banPriority: ids })}
        showLockToggle={false}
        showPinToggle={false}
      />

      <ChampionPicker
        label={t("profiles.pickPriority")}
        selected={draft.pickPriority}
        onChange={(ids) => setDraft({ ...draft, pickPriority: ids })}
      />

      <div className="flex items-center gap-6">
        <label className="flex items-center gap-2 text-sm text-text">
          <input
            type="checkbox"
            checked={draft.requestRoleSwapWhenAutofilled}
            onChange={(e) =>
              setDraft({
                ...draft,
                requestRoleSwapWhenAutofilled: e.target.checked,
              })
            }
            className="accent-accent"
          />
          {t("profiles.requestRoleSwap")}
        </label>
      </div>

      <div className="flex items-center gap-6">
        <div>
          <label className="text-xs text-text-dim block mb-1">
            {t("profiles.requestPickPosition")}
          </label>
          <select
            value={draft.requestPickPosition}
            onChange={(e) =>
              setDraft({
                ...draft,
                requestPickPosition: parseInt(e.target.value) || 0,
              })
            }
            className="bg-surface-3 border border-border rounded px-3 py-2 text-sm text-text outline-none focus:border-accent"
          >
            {PICK_POSITIONS.map((pos) => (
              <option key={pos.value} value={pos.value}>
                {t(pos.key)}
              </option>
            ))}
          </select>
        </div>
      </div>

      <div className="flex gap-2">
        <button
          onClick={onSave}
          className="px-4 py-2 bg-accent text-white rounded-lg text-sm font-medium hover:bg-accent/90 transition-colors"
        >
          {t("profiles.save")}
        </button>
        <button
          onClick={onCancel}
          className="px-4 py-2 bg-surface-3 text-text rounded-lg text-sm hover:bg-surface-3/80 transition-colors"
        >
          {t("profiles.cancel")}
        </button>
      </div>
    </div>
  );
}

export default Profiles;
