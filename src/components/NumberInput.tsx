import { useState, useCallback } from "react";

interface Props {
  value: number;
  onChange: (v: number) => void;
  step?: number;
  min?: number;
  className?: string;
}

function NumberInput({ value, onChange, step = 0.5, min = 0, className = "" }: Props) {
  const [display, setDisplay] = useState<string>(String(value));

  const commit = useCallback((raw: string) => {
    const trimmed = raw.trim();
    if (trimmed === "" || trimmed === ".") {
      onChange(min);
      setDisplay(String(min));
      return;
    }
    const parsed = parseFloat(trimmed);
    if (isNaN(parsed)) {
      setDisplay(String(value));
      return;
    }
    const clamped = Math.max(min, parsed);
    onChange(clamped);
    setDisplay(String(clamped));
  }, [value, onChange, min]);

  return (
    <input
      type="text"
      inputMode="decimal"
      value={display}
      onChange={(e) => {
        const raw = e.target.value;
        if (raw === "" || raw === "-" || raw === "." || raw === "-.") {
          setDisplay(raw);
          return;
        }
        const sanitized = raw.replace(/[^0-9.]/g, "").replace(/(\..*)\./g, "$1");
        if (sanitized !== raw) {
          setDisplay(sanitized);
          return;
        }
        setDisplay(sanitized);
        const parsed = parseFloat(sanitized);
        if (!isNaN(parsed)) {
          onChange(Math.max(min, parsed));
        }
      }}
      onBlur={() => commit(display)}
      onKeyDown={(e) => {
        if (e.key === "Enter") commit(display);
      }}
      step={step}
      min={min}
      className={`w-full bg-surface-3 border border-border rounded px-3 py-2 text-sm text-text outline-none focus:border-accent transition-colors ${className}`}
    />
  );
}

export default NumberInput;
