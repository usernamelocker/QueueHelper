import { useState, useEffect, useRef } from "react";
import { getMonitorEntries, setMonitorAutoScroll, setMonitorScrollTop, getMonitorScrollTop } from "../api/tauri";
import { useApp } from "../hooks/AppContext";
import { useTranslation } from "../i18n";
import type { MonitorEntry, MonitorLevel } from "../types/models";

const LEVELS: MonitorLevel[] = ["INFO", "WARN", "ERROR"];

const levelColors: Record<MonitorLevel, string> = {
  INFO: "text-text-dim",
  WARN: "text-yellow",
  ERROR: "text-red",
};

function Monitor() {
  const { t } = useTranslation();
  const app = useApp();
  const [entries, setEntries] = useState<MonitorEntry[]>([]);
  const [filter, setFilter] = useState<MonitorLevel | "ALL">("ALL");
  const [autoScroll, setAutoScroll] = useState(app.snapshot?.monitorAutoScroll ?? true);
  const containerRef = useRef<HTMLDivElement>(null);
  const bottomRef = useRef<HTMLDivElement>(null);
  const restoredRef = useRef(false);
  const firstEntryIdRef = useRef<number | null>(null);

  const toggleAutoScroll = async () => {
    const next = !autoScroll;
    setAutoScroll(next);
    try {
      const snapshot = await setMonitorAutoScroll(next);
      setAutoScroll(snapshot.monitorAutoScroll);
      if (next && containerRef.current) {
        containerRef.current.scrollTop = containerRef.current.scrollHeight;
      }
    } catch {
      // ignore
    }
  };

  const fetchEntries = async () => {
    try {
      const items = await getMonitorEntries(200);
      setEntries(items);
    } catch {
      // ignore
    }
  };

  useEffect(() => {
    fetchEntries();
    const interval = setInterval(fetchEntries, 2000);
    return () => clearInterval(interval);
  }, []);

  // Save scroll position periodically (every 1s) + on page leave
  useEffect(() => {
    const save = () => {
      if (containerRef.current && !autoScroll) {
        setMonitorScrollTop(containerRef.current.scrollTop).catch(() => {});
      }
    };
    const interval = setInterval(save, 1000);
    return () => {
      clearInterval(interval);
      save();
    };
  }, [autoScroll]);

  // Restore saved scroll after first meaningful render
  useEffect(() => {
    if (entries.length === 0 || restoredRef.current) return;
    const container = containerRef.current;
    if (!container) return;

    if (autoScroll) {
      container.scrollTop = container.scrollHeight;
      restoredRef.current = true;
      return;
    }

    getMonitorScrollTop().then((top) => {
      if (containerRef.current) {
        containerRef.current.scrollTop = top;
      }
      restoredRef.current = true;
    }).catch(() => {
      restoredRef.current = true;
    });
  }, [entries, autoScroll]);

  // Auto-scroll: scroll to bottom when new entries arrive
  useEffect(() => {
    if (!autoScroll || entries.length === 0) return;
    const newest = entries[entries.length - 1];
    if (firstEntryIdRef.current !== null && newest.id === firstEntryIdRef.current) {
      return;
    }
    firstEntryIdRef.current = newest.id;
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [entries, autoScroll]);

  const sorted = [...entries].reverse();
  const filtered =
    filter === "ALL"
      ? sorted
      : sorted.filter((e) => e.level === filter);

  return (
    <div className="max-w-3xl space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold text-text">{t("monitor.title")}</h2>
        <div className="flex items-center gap-2">
          {(["ALL", ...LEVELS] as const).map((level) => (
            <button
              key={level}
              onClick={() => setFilter(level)}
              className={`px-2.5 py-1 rounded text-xs font-medium transition-colors ${
                filter === level
                  ? level === "ALL"
                    ? "bg-surface-3 text-text"
                    : level === "ERROR"
                    ? "bg-red/20 text-red"
                    : level === "WARN"
                    ? "bg-yellow/20 text-yellow"
                    : "bg-accent/20 text-accent"
                  : "bg-surface-2 text-text-dim border border-border hover:bg-surface-3"
              }`}
            >
              {level === "ALL" ? t("monitor.all") : level}
            </button>
          ))}
          <button
            onClick={toggleAutoScroll}
            className={`px-2.5 py-1 rounded text-xs font-medium transition-colors ${
              autoScroll
                ? "bg-accent/20 text-accent"
                : "bg-surface-2 text-text-dim border border-border"
            }`}
          >
            {autoScroll ? t("monitor.autoScrollOn") : t("monitor.autoScrollOff")}
          </button>
        </div>
      </div>

      <div
        ref={containerRef}
        className="bg-surface-2 border border-border rounded-lg h-[60vh] overflow-y-auto font-mono text-xs"
      >
        {filtered.length === 0 ? (
          <p className="p-4 text-text-dim">{t("monitor.noEntries")}</p>
        ) : (
          filtered.map((e) => (
            <div
              key={e.id}
              className="px-4 py-1.5 border-b border-border/50 last:border-0 hover:bg-surface-3/50"
            >
              <span className="text-text-dim mr-3">{e.timestamp}</span>
              <span className={`mr-3 ${levelColors[e.level]}`}>
                [{e.level}]
              </span>
              <span className="text-text-dim mr-2">{e.category}</span>
              <span className="text-text">{e.message}</span>
            </div>
          ))
        )}
        <div ref={bottomRef} />
      </div>
    </div>
  );
}

export default Monitor;
