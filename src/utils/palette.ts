// Colours for graphs. "Fewer colours" uses a short, high-contrast set and relies on dash
// patterns and marker shapes to tell series apart.
import { computed } from "vue";
import { usePrefsStore } from "../stores/prefs";

export interface SeriesStyle {
  color: string;
  dash: string;
  marker: "circle" | "square" | "triangle";
}

const FULL: SeriesStyle[] = [
  { color: "#2f6db3", dash: "", marker: "circle" },
  { color: "#d9822b", dash: "", marker: "square" },
  { color: "#3a9d5d", dash: "", marker: "triangle" },
  { color: "#b8404f", dash: "", marker: "circle" },
  { color: "#7d5fb2", dash: "", marker: "square" },
];

const FEW: SeriesStyle[] = [
  { color: "#111111", dash: "", marker: "circle" },
  { color: "#0060a8", dash: "6 3", marker: "square" },
  { color: "#c24f00", dash: "2 3", marker: "triangle" },
  { color: "#111111", dash: "8 3 2 3", marker: "square" },
  { color: "#0060a8", dash: "1 2", marker: "triangle" },
];

export function usePalette() {
  const prefs = usePrefsStore();
  return computed(() => (prefs.prefs.fewerColours ? FEW : FULL));
}
