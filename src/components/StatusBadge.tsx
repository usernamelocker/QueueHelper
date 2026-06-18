import { useTranslation } from "../i18n";

interface Props {
  connected: boolean;
  phase?: string | null;
}

function StatusBadge({ connected, phase }: Props) {
  const { t } = useTranslation();
  const color = connected ? "bg-green" : "bg-red";

  return (
    <div className="flex items-center gap-3">
      <div className="flex items-center gap-2">
        <span className={`w-2.5 h-2.5 rounded-full ${color}`} />
        <span className="text-sm text-text">
          {connected ? t("status.connected") : t("status.disconnected")}
        </span>
      </div>
      {phase && phase !== "None" && (
        <span className="text-xs text-text-dim bg-surface-3 px-2 py-0.5 rounded">
          {phase}
        </span>
      )}
    </div>
  );
}

export default StatusBadge;
