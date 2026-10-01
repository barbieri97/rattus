/** Program time as h:mm:ss. */
export function formatClock(seconds: number): string {
  const s = Math.max(0, Math.floor(seconds));
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = s % 60;
  return `${h}:${String(m).padStart(2, "0")}:${String(sec).padStart(2, "0")}`;
}

/** Program time in minutes with one decimal. */
export function formatMinutes(seconds: number): string {
  return `${(seconds / 60).toFixed(1)} min`;
}

export function formatNumber(value: number | null | undefined, digits = 2): string {
  if (value === null || value === undefined || !Number.isFinite(value)) return "–";
  return value.toFixed(digits);
}
