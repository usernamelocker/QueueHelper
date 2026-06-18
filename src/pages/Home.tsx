import { useApp } from "../hooks/AppContext";
import { useTranslation } from "../i18n";
import StatusBadge from "../components/StatusBadge";
import ToggleCard from "../components/ToggleCard";

function Home() {
  const { t } = useTranslation();
  const {
    snapshot,
    error,
    togglePaused,
    toggleAutoAccept,
    toggleAutoBan,
    toggleAutoPick,
    toggleAutoHover,
  } = useApp();

  if (!snapshot) {
    return (
      <div className="flex items-center justify-center h-64">
        <p className="text-text-dim text-sm">{t("home.connecting")}</p>
      </div>
    );
  }

  return (
    <div className="max-w-2xl space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold text-text">{t("home.title")}</h2>
          <p className="text-xs text-text-dim mt-0.5">
            {t("home.activeProfile", {
              name:
                snapshot.activeProfileName ??
                snapshot.activeProfileId ??
                t("home.none"),
            })}
          </p>
        </div>
        <StatusBadge
          connected={snapshot.connected}
          phase={snapshot.gamePhase}
        />
      </div>

      {error && (
        <div className="bg-red/10 border border-red/30 rounded-lg px-4 py-2 text-sm text-red">
          {error}
        </div>
      )}

      <div className="space-y-3">
        <ToggleCard
          label={t("home.autoAccept")}
          description={t("home.autoAcceptDesc")}
          enabled={snapshot.autoAcceptEnabled}
          onToggle={toggleAutoAccept}
        />
        <ToggleCard
          label={t("home.autoBan")}
          description={t("home.autoBanDesc")}
          enabled={snapshot.autoBanEnabled}
          onToggle={toggleAutoBan}
        />
        <ToggleCard
          label={t("home.autoPick")}
          description={t("home.autoPickDesc")}
          enabled={snapshot.autoPickEnabled}
          onToggle={toggleAutoPick}
        />
        <ToggleCard
          label={t("home.autoHover")}
          description={t("home.autoHoverDesc")}
          enabled={snapshot.autoHoverEnabled}
          onToggle={toggleAutoHover}
        />
      </div>

      <div className="flex items-center gap-4">
        <button
          onClick={togglePaused}
          className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
            snapshot.automationPaused
              ? "bg-yellow/20 text-yellow border border-yellow/40 hover:bg-yellow/30"
              : "bg-surface-3 text-text border border-border hover:bg-surface-3/80"
          }`}
        >
          {snapshot.automationPaused ? t("home.resume") : t("home.pauseAll")}
        </button>
      </div>

      {snapshot.lastAction && (
        <div className="bg-surface-2 border border-border rounded-lg px-4 py-3">
          <p className="text-xs text-text-dim mb-1">{t("home.lastAction")}</p>
          <p className="text-sm text-text">{snapshot.lastAction}</p>
        </div>
      )}
    </div>
  );
}

export default Home;
