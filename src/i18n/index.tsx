import { createContext, useContext, useState, useEffect, useCallback, type ReactNode } from "react";
import { getSettings } from "../api/tauri";
import { en } from "./en";
import { tr } from "./tr";

type Dict = typeof en;
type TranslationKey = keyof Dict;

const dicts: Record<string, Dict> = { en, tr };

function interpolate(template: string, args?: Record<string, string | number>): string {
  if (!args) return template;
  return template.replace(/\{(\w+)\}/g, (_, key) =>
    args[key] !== undefined ? String(args[key]) : `{${key}}`
  );
}

interface TranslationContextValue {
  t: (key: TranslationKey, args?: Record<string, string | number>) => string;
  language: string;
  setLanguage: (lang: string) => void;
}

const TranslationContext = createContext<TranslationContextValue>({
  t: (key) => String(key),
  language: "en",
  setLanguage: () => {},
});

export function TranslationProvider({ children }: { children: ReactNode }) {
  const [language, setLanguageState] = useState("en");

  useEffect(() => {
    getSettings()
      .then((s) => {
        if (s.language) setLanguageState(s.language);
      })
      .catch(() => {});
  }, []);

  const setLanguage = useCallback((lang: string) => {
    setLanguageState(lang);
  }, []);

  const t = useCallback(
    (key: TranslationKey, args?: Record<string, string | number>): string => {
      const dict = dicts[language] ?? en;
      const val = dict[key];
      if (val === undefined) return String(key);
      if (Array.isArray(val)) return val.join("\n");
      return interpolate(val, args);
    },
    [language]
  );

  return (
    <TranslationContext.Provider value={{ t, language, setLanguage }}>
      {children}
    </TranslationContext.Provider>
  );
}

export function useTranslation() {
  return useContext(TranslationContext);
}
