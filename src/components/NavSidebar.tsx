import { NavLink } from "react-router-dom";
import { useTranslation } from "../i18n";

const linkDefs = [
  { to: "/", key: "nav.dashboard" as const, icon: "◈" },
  { to: "/settings", key: "nav.settings" as const, icon: "⚙" },
  { to: "/profiles", key: "nav.profiles" as const, icon: "♛" },
  { to: "/draft-rules", key: "nav.rules" as const, icon: "▣" },
  { to: "/monitor", key: "nav.monitor" as const, icon: "≡" },
];

interface Props {
  onHelp: () => void;
  showGlow: boolean;
}

function NavSidebar({ onHelp, showGlow }: Props) {
  const { t } = useTranslation();

  return (
    <nav className="w-48 min-h-screen bg-surface-2 border-r border-border flex flex-col py-4 shrink-0">
      <div className="px-4 pb-4 mb-2 border-b border-border">
        <h1 className="text-sm font-semibold text-accent tracking-wide">
          {t("app.title")}
        </h1>
      </div>
      {linkDefs.map((link) => (
        <NavLink
          key={link.to}
          to={link.to}
          className={({ isActive }) =>
            `flex items-center gap-3 px-4 py-2.5 text-sm transition-colors ${
              isActive
                ? "bg-accent/10 text-accent border-r-2 border-accent"
                : "text-text-dim hover:text-text hover:bg-surface-3"
            }`
          }
        >
          <span className="text-base w-5 text-center">{link.icon}</span>
          {t(link.key)}
        </NavLink>
      ))}
      <div className="mt-auto px-4 pt-4 border-t border-border">
        <button
          onClick={onHelp}
          className={`flex items-center gap-3 w-full px-4 py-2.5 text-sm rounded-lg transition-all ${
            showGlow
              ? "text-accent bg-accent/10 animate-pulse shadow-[0_0_12px_2px_rgba(64,169,255,0.3)]"
              : "text-text-dim hover:text-text hover:bg-surface-3"
          }`}
        >
          <span className="text-base w-5 text-center">?</span>
          {t("nav.howToUse")}
        </button>
      </div>
    </nav>
  );
}

export default NavSidebar;
