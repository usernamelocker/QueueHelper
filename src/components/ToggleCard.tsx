interface Props {
  label: string;
  description: string;
  enabled: boolean;
  loading?: boolean;
  onToggle: () => void;
}

function ToggleCard({ label, description, enabled, loading, onToggle }: Props) {
  return (
    <div className="bg-surface-2 border border-border rounded-lg p-4 flex items-center justify-between gap-4">
      <div className="min-w-0">
        <p className="text-sm font-medium text-text">{label}</p>
        <p className="text-xs text-text-dim mt-0.5">{description}</p>
      </div>
      <button
        onClick={onToggle}
        disabled={loading}
        className={`relative w-10 h-5 rounded-full transition-colors shrink-0 disabled:opacity-50 ${
          enabled ? "bg-accent" : "bg-surface-3"
        }`}
      >
        <span
          className={`absolute top-0.5 left-0.5 w-4 h-4 bg-white rounded-full transition-transform ${
            enabled ? "translate-x-5" : "translate-x-0"
          }`}
        />
      </button>
    </div>
  );
}

export default ToggleCard;
