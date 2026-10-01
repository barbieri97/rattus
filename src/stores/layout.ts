// Position, size and visibility of the floating windows.
import { defineStore } from "pinia";
import { ref, watch } from "vue";
import { loadJson, saveJson } from "../utils/storage";

export type PanelId =
  "chamber" | "cumulative" | "operant" | "behavior" | "csStrength" | "sensitivityFear" | "suppression" | "movement";

export interface PanelState {
  open: boolean;
  x: number;
  y: number;
  w: number;
  h: number;
  z: number;
}

export const PANEL_TITLES: Record<PanelId, string> = {
  chamber: "Operant Chamber",
  cumulative: "Cumulative Record",
  operant: "Operant Associations",
  behavior: "Behavior Log",
  csStrength: "CS Response Strength",
  sensitivityFear: "Sensitivity & Fear",
  suppression: "Suppression Ratio",
  movement: "Movement Ratio",
};

export const PANEL_ORDER: PanelId[] = [
  "chamber",
  "cumulative",
  "operant",
  "behavior",
  "csStrength",
  "sensitivityFear",
  "suppression",
  "movement",
];

const KEY = "rattus.layout.v1";
export const MIN_W = 220;
export const MIN_H = 140;

/** A sensible arrangement for a workspace of the given size. */
export function defaultLayout(width: number, height: number): Record<PanelId, PanelState> {
  const gap = 8;
  const leftW = Math.max(520, Math.round(width * 0.6));
  const rightW = Math.max(MIN_W, width - leftW - gap * 3);
  const chamberH = Math.max(320, Math.round(height * 0.6));
  const bottomH = Math.max(MIN_H, height - chamberH - gap * 3);
  const rightX = leftW + gap * 2;
  const third = Math.max(MIN_H, Math.round((height - gap * 4) / 3));
  const half = Math.max(MIN_H, Math.round((height - gap * 3) / 2));
  const panel = (open: boolean, x: number, y: number, w: number, h: number, z: number): PanelState => ({
    open,
    x,
    y,
    w,
    h,
    z,
  });
  return {
    chamber: panel(true, gap, gap, leftW, chamberH, 1),
    cumulative: panel(true, gap, chamberH + gap * 2, leftW, bottomH, 2),
    operant: panel(true, rightX, gap, rightW, third, 3),
    behavior: panel(true, rightX, third + gap * 2, rightW, third, 4),
    csStrength: panel(false, rightX, third * 2 + gap * 3, rightW, third, 5),
    sensitivityFear: panel(false, rightX, gap, rightW, third, 6),
    suppression: panel(false, rightX, gap, rightW, half, 7),
    movement: panel(false, rightX, half + gap * 2, rightW, half, 8),
  };
}

function isPanelState(v: unknown): v is PanelState {
  if (!v || typeof v !== "object") return false;
  const p = v as Record<string, unknown>;
  return (
    ["x", "y", "w", "h", "z"].every((k) => typeof p[k] === "number" && Number.isFinite(p[k])) &&
    typeof p.open === "boolean"
  );
}

export const useLayoutStore = defineStore("layout", () => {
  const panels = ref<Record<PanelId, PanelState>>(defaultLayout(1260, 720));
  const active = ref<PanelId>("chamber");
  const workspace = ref({ width: 1260, height: 720 });
  let topZ = 10;

  function init(width: number, height: number) {
    workspace.value = { width, height };
    const saved = loadJson<Record<PanelId, PanelState>>(KEY);
    const base = defaultLayout(width, height);
    if (saved) {
      for (const id of PANEL_ORDER) {
        const p = saved[id];
        if (isPanelState(p)) base[id] = { ...p };
      }
    }
    panels.value = base;
    topZ = Math.max(...PANEL_ORDER.map((id) => base[id].z)) + 1;
    keepInside();
  }

  /** Moves windows back inside the workspace after it shrank. */
  function keepInside() {
    const { width, height } = workspace.value;
    for (const id of PANEL_ORDER) {
      const p = panels.value[id];
      p.w = Math.max(MIN_W, Math.min(p.w, width));
      p.h = Math.max(MIN_H, Math.min(p.h, height));
      p.x = Math.max(0, Math.min(p.x, width - 80));
      p.y = Math.max(0, Math.min(p.y, height - 32));
    }
  }

  function resize(width: number, height: number) {
    workspace.value = { width, height };
    keepInside();
  }

  function focus(id: PanelId) {
    active.value = id;
    if (panels.value[id].z < topZ - 1) panels.value[id].z = topZ++;
  }

  function open(id: PanelId) {
    panels.value[id].open = true;
    focus(id);
  }

  function close(id: PanelId) {
    panels.value[id].open = false;
  }

  function toggle(id: PanelId) {
    if (panels.value[id].open) close(id);
    else open(id);
  }

  function reset() {
    panels.value = defaultLayout(workspace.value.width, workspace.value.height);
    topZ = 20;
  }

  watch(panels, (value) => saveJson(KEY, value), { deep: true });

  return { panels, active, workspace, init, resize, focus, open, close, toggle, reset };
});
