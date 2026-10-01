<script setup lang="ts">
// Animates the chamber: every display frame the rat is drawn between the two latest
// simulation frames, with smooth transitions from one behaviour to the next.
import { computed, onBeforeUnmount, onMounted, reactive, watch } from "vue";
import * as actions from "../actions";
import { blend, NEUTRAL, poseFor, type Pose } from "../animation/pose";
import type { Behavior } from "../bindings/Behavior";
import type { Snapshot } from "../bindings/Snapshot";
import { usePrefsStore } from "../stores/prefs";
import { useSimStore } from "../stores/sim";
import { BEHAVIOR_LABELS } from "../utils/labels";
import ChamberScene, { type RatDrawing } from "./ChamberScene.vue";

const sim = useSimStore();
const prefs = usePrefsStore();

const BLEND_MS = 180;
const DROP_MS = 380;
/** Path of a pellet from the dispenser, down the tube, into the cup. */
const TUBE: [number, number][] = [
  [981, 214],
  [981, 470],
  [944, 500],
];

const rat = reactive<RatDrawing>({ x: 0.5, z: 0.5, facing: 1, pose: { ...NEUTRAL }, time: 0, jitterX: 0, jitterY: 0 });
const state = reactive({ behavior: "lookAround" as Behavior, drop: -1 });

let raf = 0;
let stride = 0;
let lastX: number | null = null;
let shownPose: Pose = { ...NEUTRAL };
let blendFrom: Pose = { ...NEUTRAL };
let blendStart = 0;
let boutKey = "";
let dropStart = -1;

interface Bout {
  behavior: Behavior;
  elapsed: number;
  duration: number;
  start: number;
}

/** The bout under way at program time `t`, from the two latest snapshots. */
function boutAt(prev: Snapshot | null, snap: Snapshot, t: number): Bout {
  const latest = snap.rat;
  const elapsed = latest.elapsed - (snap.t - t);
  if (elapsed >= 0 || !prev) {
    return {
      behavior: latest.behavior,
      elapsed: Math.max(0, elapsed),
      duration: latest.duration,
      start: snap.t - latest.elapsed,
    };
  }
  const p = prev.rat;
  return { behavior: p.behavior, elapsed: p.elapsed + (t - prev.t), duration: p.duration, start: prev.t - p.elapsed };
}

function frame(now: number) {
  raf = requestAnimationFrame(frame);
  const snap = sim.snapshot;
  if (!snap) return;
  const prev = sim.previous;
  const { previousAt, latestAt } = sim.timing;
  const interval = Math.max(16, latestAt - previousAt);
  // Draw one simulation frame behind, interpolating from the previous snapshot to the latest.
  const k = prev ? Math.min(1, Math.max(0, (now - latestAt) / interval)) : 1;
  const teleported = prev ? Math.abs(prev.rat.x - snap.rat.x) > 0.15 : true;
  const from = prev && !teleported ? prev : snap;
  const t = from.t + (snap.t - from.t) * k;
  rat.x = from.rat.x + (snap.rat.x - from.rat.x) * k;
  rat.z = from.rat.z + (snap.rat.z - from.rat.z) * k;
  rat.facing = (k < 0.5 ? from.rat.facing : snap.rat.facing) === "right" ? 1 : -1;
  rat.time = t;

  const bout = boutAt(from === snap ? null : from, snap, t);
  if (lastX !== null) stride += Math.abs(rat.x - lastX) * 100;
  lastX = rat.x;
  const progress = bout.duration > 0 ? bout.elapsed / bout.duration : 1;
  const target = poseFor(bout.behavior, bout.elapsed, progress, stride);
  const key = `${bout.behavior}@${bout.start.toFixed(2)}`;
  if (key !== boutKey) {
    boutKey = key;
    blendFrom = shownPose;
    blendStart = now;
  }
  shownPose = blend(blendFrom, target, (now - blendStart) / BLEND_MS);
  rat.pose = shownPose;
  state.behavior = bout.behavior;
  const j = shownPose.jitter;
  rat.jitterX = j * Math.sin(now * 0.09);
  rat.jitterY = j * Math.sin(now * 0.13 + 1);
  state.drop = dropStart >= 0 && now - dropStart < DROP_MS ? (now - dropStart) / DROP_MS : -1;
}

watch(
  () => sim.snapshot?.chamber.dispenseCount,
  (count, old) => {
    if (count !== undefined && old !== undefined && count > old && !sim.host.isolated) {
      dropStart = performance.now();
    }
  },
);

onMounted(() => {
  raf = requestAnimationFrame(frame);
});
onBeforeUnmount(() => cancelAnimationFrame(raf));

const drop = computed(() => {
  const p = state.drop;
  if (p < 0) return null;
  const segs = TUBE.length - 1;
  const f = Math.min(segs - 1e-6, p * segs);
  const i = Math.floor(f);
  const u = f - i;
  const [x0, y0] = TUBE[i];
  const [x1, y1] = TUBE[i + 1];
  return { x: x0 + (x1 - x0) * u, y: y0 + (y1 - y0) * u };
});

const statusText = computed(() => {
  if (!sim.snapshot) return "Connecting…";
  if (sim.host.isolated) return "Rat isolated: time accelerated";
  const p = sim.snapshot.chamber.pellets;
  return `${BEHAVIOR_LABELS[state.behavior]}${p > 0 ? ` · ${p} pellet${p > 1 ? "s" : ""} in the cup` : ""}`;
});
</script>

<template>
  <div class="chamber" :class="{ scroll: !prefs.prefs.scaleChamber }">
    <ChamberScene
      :rat="sim.snapshot ? rat : null"
      :chamber="sim.snapshot?.chamber ?? null"
      :isolated-speed="sim.host.isolated ? sim.host.isolatedSpeed : null"
      :drop="drop"
      :scale="prefs.prefs.scaleChamber"
      @dispense="actions.givePellet"
    />
    <div class="status">{{ statusText }}</div>
  </div>
</template>

<style scoped>
.chamber {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  background: #c7d0da;
}
.chamber > :deep(svg) {
  flex: 1;
  min-height: 0;
}
.chamber.scroll {
  overflow: auto;
}
.chamber.scroll > :deep(svg) {
  flex: none;
}
.status {
  padding: 3px 10px;
  background: #eef1f5;
  border-top: 1px solid #c4ccd6;
  color: var(--muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
