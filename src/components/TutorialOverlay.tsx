import { useTranslation } from "../i18n";

interface Props {
  onClose: () => void;
}

function TutorialOverlay({ onClose }: Props) {
  const { t } = useTranslation();

  const sections = [
    {
      titleKey: "tutorial.dashboardTitle" as const,
      key: "dashboard" as const,
      icon: "◈",
      linesKey: "tutorial.dashboardLines" as const,
    },
    {
      titleKey: "tutorial.settingsTitle" as const,
      key: "settings" as const,
      icon: "⚙",
      linesKey: "tutorial.settingsLines" as const,
    },
    {
      titleKey: "tutorial.profilesTitle" as const,
      key: "profiles" as const,
      icon: "♛",
      linesKey: "tutorial.profilesLines" as const,
    },
    {
      titleKey: "tutorial.draftRulesTitle" as const,
      key: "draftRules" as const,
      icon: "▣",
      linesKey: "tutorial.draftRulesLines" as const,
    },
    {
      titleKey: "tutorial.monitorTitle" as const,
      key: "monitor" as const,
      icon: "≡",
      linesKey: "tutorial.monitorLines" as const,
    },
  ];

  const dashboardLines = t("tutorial.dashboardLines");
  const settingsLines = t("tutorial.settingsLines");
  const profilesLines = t("tutorial.profilesLines");
  const draftRulesLines = t("tutorial.draftRulesLines");
  const monitorLines = t("tutorial.monitorLines");

  const sectionLines: Record<string, string> = {
    dashboard: dashboardLines,
    settings: settingsLines,
    profiles: profilesLines,
    draftRules: draftRulesLines,
    monitor: monitorLines,
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
      onClick={onClose}
    >
      <div
        className="bg-surface-2 border border-border rounded-xl shadow-2xl max-w-lg w-full mx-4 max-h-[80vh] overflow-y-auto"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="sticky top-0 bg-surface-2 border-b border-border px-6 py-4 flex items-center justify-between">
          <h2 className="text-base font-semibold text-accent tracking-wide">
            {t("tutorial.title")}
          </h2>
          <button
            onClick={onClose}
            className="text-text-dim hover:text-text text-lg leading-none transition-colors"
          >
            ✕
          </button>
        </div>
        <div className="px-6 py-5 space-y-6">
          <p className="text-sm text-text-dim leading-relaxed">
            {t("tutorial.intro")}
          </p>
          {sections.map((section) => (
            <div key={section.key}>
              <h3 className="text-sm font-medium text-text mb-2">
                <span className="text-accent mr-2">{section.icon}</span>
                {t(section.titleKey)}
              </h3>
              <ul className="space-y-1">
                {sectionLines[section.key].split("\n").map((line, i) => (
                  <li
                    key={i}
                    className="text-xs text-text-dim leading-relaxed pl-5"
                  >
                    {line}
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

export default TutorialOverlay;
