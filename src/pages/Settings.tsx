import { useState, useEffect } from "react";
import { updateSettings } from "../api/tauri";
import { useApp } from "../hooks/AppContext";
import { useTranslation } from "../i18n";
import type { AppSettings } from "../types/models";
import NumberInput from "../components/NumberInput";

const KNOWN_QUEUES = [400, 420, 430, 440, 450, 490, 700, 1700, 1900];

function getQueueOptions(t: (key: any) => string) {
  return KNOWN_QUEUES.map((id) => ({
    id,
    name: t(`queue.${id}` as any),
  }));
}

function Settings() {
  const { t, language, setLanguage } = useTranslation();
  const app = useApp();
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [initialized, setInitialized] = useState(false);
  const [dirty, setDirty] = useState(false);

  useEffect(() => {
    if (app.settings && !initialized) {
      setSettings(app.settings);
      setInitialized(true);
    }
  }, [app.settings, initialized]);
  const [saving, setSaving] = useState(false);

  const update = (patch: Partial<AppSettings>) => {
    if (!settings) return;
    setSettings({ ...settings, ...patch });
    setDirty(true);
  };

  const updateAutomation = (patch: Partial<AppSettings["automation"]>) => {
    if (!settings) return;
    setSettings({
      ...settings,
      automation: { ...settings.automation, ...patch },
    });
    setDirty(true);
  };

  const updateAutoAccept = (
    patch: Partial<AppSettings["automation"]["autoAccept"]>
  ) => {
    if (!settings) return;
    setSettings({
      ...settings,
      automation: {
        ...settings.automation,
        autoAccept: { ...settings.automation.autoAccept, ...patch },
      },
    });
    setDirty(true);
  };

  const save = async () => {
    if (!settings) return;
    setSaving(true);
    try {
      await updateSettings(settings);
      setDirty(false);
    } catch (e) {
      console.error(e);
    } finally {
      setSaving(false);
    }
  };

  if (!settings) {
    return (
      <div className="flex items-center justify-center h-64">
        <p className="text-text-dim text-sm">{t("settings.loading")}</p>
      </div>
    );
  }

  const queueOptions = getQueueOptions(t);
  const customLabel = t("queue.custom");

  return (
    <div className="max-w-lg space-y-6">
      <h2 className="text-lg font-semibold text-text">{t("settings.title")}</h2>

      <section className="space-y-4">
        <h3 className="text-xs text-text-dim font-medium uppercase tracking-wide">
          {t("settings.language")}
        </h3>
        <select
          value={language}
          onChange={(e) => {
            const lang = e.target.value;
            setLanguage(lang);
            update({ language: lang });
          }}
          className="bg-surface-3 border border-border rounded px-3 py-2 text-sm text-text outline-none focus:border-accent"
        >
          <option value="en">English</option>
          <option value="tr">Türkçe</option>
        </select>
      </section>

      <section className="space-y-4">
        <h3 className="text-xs text-text-dim font-medium uppercase tracking-wide">
          {t("settings.leagueClient")}
        </h3>
        <div>
          <label className="text-xs text-text-dim block mb-1">
            {t("settings.installPath")}
          </label>
          <input
            type="text"
            value={settings.leagueInstallPath ?? ""}
            onChange={(e) =>
              update({
                leagueInstallPath: e.target.value || null,
              })
            }
            placeholder={t("settings.installPathPlaceholder")}
            className="w-full bg-surface-3 border border-border rounded px-3 py-2 text-sm text-text placeholder:text-text-dim/40 outline-none focus:border-accent transition-colors font-mono"
          />
        </div>
      </section>

      <section className="space-y-4">
        <h3 className="text-xs text-text-dim font-medium uppercase tracking-wide">
          {t("settings.autoAcceptDelay")}
        </h3>
        <div className="flex gap-4">
          <div className="flex-1">
            <label className="text-xs text-text-dim block mb-1">
              {t("settings.minSeconds")}
            </label>
            <NumberInput
              value={settings.automation.autoAccept.delayMinSeconds}
              onChange={(v) => updateAutoAccept({ delayMinSeconds: v })}
              step={0.1}
              min={0}
            />
          </div>
          <div className="flex-1">
            <label className="text-xs text-text-dim block mb-1">
              {t("settings.maxSeconds")}
            </label>
            <NumberInput
              value={settings.automation.autoAccept.delayMaxSeconds}
              onChange={(v) => updateAutoAccept({ delayMaxSeconds: v })}
              step={0.1}
              min={0}
            />
          </div>
        </div>
      </section>

      <section className="space-y-4">
        <h3 className="text-xs text-text-dim font-medium uppercase tracking-wide">
          {t("settings.autoHoverDelay")}
        </h3>
        <p className="text-xs text-text-dim leading-relaxed">
          {t("settings.autoHoverDelayDesc")}
        </p>
        <div>
          <label className="text-xs text-text-dim block mb-1">
            {t("settings.delaySeconds")}
          </label>
          <NumberInput
            value={settings.automation.autoHoverDelaySeconds}
            onChange={(v) => updateAutomation({ autoHoverDelaySeconds: v })}
            step={0.5}
            min={0}
          />
        </div>
      </section>

      <section className="space-y-4">
        <h3 className="text-xs text-text-dim font-medium uppercase tracking-wide">
          {t("settings.autoBanDelay")}
        </h3>
        <p className="text-xs text-text-dim leading-relaxed">
          {t("settings.autoBanDelayDesc")}
        </p>
        <div>
          <label className="text-xs text-text-dim block mb-1">
            {t("settings.delaySeconds")}
          </label>
          <NumberInput
            value={settings.automation.autoBanDelaySeconds}
            onChange={(v) => updateAutomation({ autoBanDelaySeconds: v })}
            step={0.5}
            min={0}
          />
        </div>
      </section>

      <section className="space-y-4">
        <h3 className="text-xs text-text-dim font-medium uppercase tracking-wide">
          {t("settings.autoPickDelay")}
        </h3>
        <p className="text-xs text-text-dim leading-relaxed">
          {t("settings.autoPickDelayDesc")}
        </p>
        <div>
          <label className="text-xs text-text-dim block mb-1">
            {t("settings.delaySeconds")}
          </label>
          <NumberInput
            value={settings.automation.autoPickDelaySeconds}
            onChange={(v) => updateAutomation({ autoPickDelaySeconds: v })}
            step={0.5}
            min={0}
          />
        </div>
      </section>

      <section className="space-y-4">
        <h3 className="text-xs text-text-dim font-medium uppercase tracking-wide">
          {t("settings.queueOverrides")}
        </h3>
        <p className="text-xs text-text-dim leading-relaxed">
          {t("settings.queueOverridesDesc")}
        </p>
        {settings.automation.queueOverrides.length === 0 ? (
          <p className="text-xs text-text-dim">{t("settings.noOverrides")}</p>
        ) : (
          settings.automation.queueOverrides.map((qo, i) => {
            return (
              <div key={i} className="flex items-center gap-3">
                <div className="flex items-center gap-2">
                  <select
                    value={qo.queueId}
                    onChange={(e) => {
                      const id = parseInt(e.target.value);
                      if (isNaN(id)) return;
                      const next = [...settings.automation.queueOverrides];
                      next[i] = { ...next[i], queueId: id };
                      updateAutomation({ queueOverrides: next });
                    }}
                    className="bg-surface-3 border border-border rounded px-2 py-1.5 text-sm text-text outline-none focus:border-accent"
                  >
                    {queueOptions.map((q) => (
                      <option key={q.id} value={q.id}>
                        {q.name}
                      </option>
                    ))}
                    <option value={-1}>{customLabel}</option>
                  </select>
                </div>
                <span className="text-xs text-text-dim">
                  {t("settings.autoAccept")}
                </span>
                <input
                  type="checkbox"
                  checked={qo.autoAcceptEnabled}
                  onChange={() => {
                    const next = [...settings.automation.queueOverrides];
                    next[i] = {
                      ...next[i],
                      autoAcceptEnabled: !qo.autoAcceptEnabled,
                    };
                    updateAutomation({ queueOverrides: next });
                  }}
                  className="accent-accent"
                />
                <button
                  onClick={() => {
                    const next = settings.automation.queueOverrides.filter(
                      (_, j) => j !== i
                    );
                    updateAutomation({ queueOverrides: next });
                  }}
                  className="text-xs text-red hover:text-red/80"
                >
                  {t("settings.remove")}
                </button>
              </div>
            );
          })
        )}
        <button
          onClick={() => {
            const next = [
              ...settings.automation.queueOverrides,
              { queueId: 400, autoAcceptEnabled: true },
            ];
            updateAutomation({ queueOverrides: next });
          }}
          className="text-xs text-accent hover:text-accent/80"
        >
          {t("settings.addOverride")}
        </button>
      </section>

      <div className="flex items-center gap-3 pt-2">
        <button
          onClick={save}
          disabled={!dirty || saving}
          className="px-4 py-2 bg-accent text-white rounded-lg text-sm font-medium hover:bg-accent/90 disabled:opacity-40 transition-colors"
        >
          {saving ? t("settings.saving") : t("settings.saveSettings")}
        </button>
        {dirty && (
          <span className="text-xs text-yellow">{t("settings.unsavedChanges")}</span>
        )}
      </div>
    </div>
  );
}

export default Settings;
