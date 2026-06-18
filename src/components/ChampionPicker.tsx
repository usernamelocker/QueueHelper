import { useState, useRef, useEffect } from "react";
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
} from "@dnd-kit/core";
import {
  arrayMove,
  SortableContext,
  sortableKeyboardCoordinates,
  useSortable,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { useChampions } from "../hooks/useChampions";
import { useTranslation } from "../i18n";
import type { Champion } from "../hooks/useChampions";
import type { ChampionPriorityEntry } from "../types/models";

interface Props {
  selected: ChampionPriorityEntry[];
  onChange: (entries: ChampionPriorityEntry[]) => void;
  max?: number;
  label?: string;
  showLockToggle?: boolean;
  showPinToggle?: boolean;
}

function SortableItem({
  champion,
  entry,
  index,
  onRemove,
  onToggleHover,
  onTogglePin,
  showLockToggle,
  showPinToggle,
}: {
  champion: Champion;
  entry: ChampionPriorityEntry;
  index: number;
  onRemove: (key: number) => void;
  onToggleHover: (key: number) => void;
  onTogglePin: (key: number) => void;
  showLockToggle: boolean;
  showPinToggle: boolean;
}) {
  const { t } = useTranslation();
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: entry.championId });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      className="flex items-center justify-between bg-surface-3 border border-border rounded px-3 py-1.5"
    >
      <div className="flex items-center gap-2 min-w-0">
        <button
          {...attributes}
          {...listeners}
          className="p-0.5 text-text-dim hover:text-text cursor-grab active:cursor-grabbing text-sm"
        >
          ⠿
        </button>
        <span className="text-sm text-text truncate">
          {index + 1}. {champion.name}
        </span>
      </div>
      <div className="flex items-center gap-1 shrink-0">
        {showPinToggle && (
          <button
            onClick={() => onTogglePin(entry.championId)}
            className={`p-0.5 text-xs transition-colors ${
              entry.isHoverTarget
                ? "text-accent hover:text-accent/80"
                : "text-text-dim hover:text-text"
            }`}
            title={
              entry.isHoverTarget
                ? t("picker.tooltipPinned")
                : t("picker.tooltipNotPinned")
            }
          >
            {entry.isHoverTarget ? "📌" : "📍"}
          </button>
        )}
        {showLockToggle && (
          <button
            onClick={() => onToggleHover(entry.championId)}
            className={`p-0.5 text-xs transition-colors ${
              entry.ignoreTeammateHovers
                ? "text-yellow hover:text-yellow/80"
                : "text-text-dim hover:text-text"
            }`}
            title={
              entry.ignoreTeammateHovers
                ? t("picker.tooltipLocked")
                : t("picker.tooltipUnlocked")
            }
          >
            {entry.ignoreTeammateHovers ? "🔓" : "🔒"}
          </button>
        )}
        <button
          onClick={() => onRemove(entry.championId)}
          className="p-0.5 text-text-dim hover:text-red text-xs"
        >
          ✕
        </button>
      </div>
    </div>
  );
}

function ChampionPicker({ selected, onChange, max, label, showLockToggle = true, showPinToggle = true }: Props) {
  const { champions, loading } = useChampions();
  const { t } = useTranslation();
  const [query, setQuery] = useState("");
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 4 } }),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  );

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, []);

  const championMap = new Map(champions.map((c) => [Number(c.key), c]));

  const filtered = query
    ? champions.filter(
        (c) =>
          c.name.toLowerCase().includes(query.toLowerCase()) ||
          c.key.includes(query)
      )
    : champions;

  const selectedChampions = selected
    .map((e) => championMap.get(e.championId))
    .filter((c): c is Champion => c != null);

  const addChampion = (c: Champion) => {
    if (max && selected.length >= max) return;
    if (selected.some((e) => e.championId === Number(c.key))) return;
    onChange([
      ...selected,
      { championId: Number(c.key), ignoreTeammateHovers: false, isHoverTarget: false },
    ]);
    setQuery("");
    setOpen(false);
  };

  const removeChampion = (key: number) => {
    onChange(selected.filter((e) => e.championId !== key));
  };

  const toggleHover = (key: number) => {
    onChange(
      selected.map((e) =>
        e.championId === key
          ? { ...e, ignoreTeammateHovers: !e.ignoreTeammateHovers }
          : e
      )
    );
  };

  const togglePin = (key: number) => {
    onChange(
      selected.map((e) =>
        e.championId === key
          ? { ...e, isHoverTarget: !e.isHoverTarget }
          : { ...e, isHoverTarget: false }
      )
    );
  };

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    if (!over || active.id === over.id) return;
    const oldIndex = selected.findIndex(
      (e) => e.championId === Number(active.id)
    );
    const newIndex = selected.findIndex(
      (e) => e.championId === Number(over.id)
    );
    if (oldIndex === -1 || newIndex === -1) return;
    onChange(arrayMove(selected, oldIndex, newIndex));
  };

  return (
    <div ref={ref} className="space-y-2">
      {label && (
        <p className="text-xs text-text-dim font-medium">{label}</p>
      )}

      <div className="relative">
        <input
          type="text"
          value={query}
          onChange={(e) => {
            setQuery(e.target.value);
            setOpen(true);
          }}
          onFocus={() => setOpen(true)}
          placeholder={loading ? t("picker.loadingChampions") : t("picker.searchChampions")}
          disabled={loading}
          className="w-full bg-surface-3 border border-border rounded px-3 py-2 text-sm text-text placeholder:text-text-dim/40 outline-none focus:border-accent transition-colors disabled:opacity-50"
        />
        {open && query && (
          <div className="absolute top-full left-0 right-0 mt-1 bg-surface-2 border border-border rounded max-h-48 overflow-y-auto z-10">
            {filtered.length === 0 ? (
              <p className="px-3 py-2 text-xs text-text-dim">{t("picker.noResults")}</p>
            ) : (
              filtered.map((c) => (
                <button
                  key={c.key}
                  onClick={() => addChampion(c)}
                  className="w-full text-left px-3 py-1.5 text-sm text-text hover:bg-surface-3 transition-colors"
                >
                  {c.name}
                  <span className="text-text-dim ml-2 text-xs">
                    #{c.key}
                  </span>
                </button>
              ))
            )}
          </div>
        )}
      </div>

      {selectedChampions.length > 0 && (
        <div className="text-[10px] text-text-dim mb-1 flex items-center gap-3">
          {showPinToggle && <span>{t("picker.notPinned")}</span>}
          {showPinToggle && <span>{t("picker.pinned")}</span>}
          {showLockToggle && <span>{t("picker.respectHovers")}</span>}
          {showLockToggle && <span>{t("picker.ignoreHovers")}</span>}
        </div>
      )}

      {selectedChampions.length > 0 && (
        <DndContext
          sensors={sensors}
          collisionDetection={closestCenter}
          onDragEnd={handleDragEnd}
        >
          <SortableContext
            items={selected.map((e) => e.championId)}
            strategy={verticalListSortingStrategy}
          >
            <div className="space-y-1">
              {selectedChampions.map((c) => {
                const entry = selected.find(
                  (e) => e.championId === Number(c.key)
                );
                if (!entry) return null;
                const idx = selected.indexOf(entry);
                return (
                  <SortableItem
                    key={c.key}
                    champion={c}
                    entry={entry}
                    index={idx}
                    onRemove={removeChampion}
                    onToggleHover={toggleHover}
                    onTogglePin={togglePin}
                    showLockToggle={showLockToggle}
                    showPinToggle={showPinToggle}
                  />
                );
              })}
            </div>
          </SortableContext>
        </DndContext>
      )}
    </div>
  );
}

export default ChampionPicker;
