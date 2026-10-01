<script setup lang="ts">
// The cumulative record: the pen steps up at each bar press and moves right with time,
// resetting to the bottom when the paper is full. Ticks mark reinforcements; the strip at
// the bottom shows CS presentations and shocks.
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { registerCopy } from "../../actions";
import { useElementSize } from "../../composables/useElementSize";
import { lowerBound } from "../../stores/session";
import { usePrefsStore } from "../../stores/prefs";
import { useSimStore } from "../../stores/sim";
import { niceTickMinutes, penHeightAt, penSegments } from "../../utils/cumulative";
import { toTsv } from "../../utils/tsv";

const sim = useSimStore();
const prefs = usePrefsStore();
const box = ref<HTMLElement | null>(null);
const canvas = ref<HTMLCanvasElement | null>(null);
const { width, height } = useElementSize(box);

const ZOOMS = [2, 5, 10, 20, 40, 80, 160];
const zoom = ref(3); // index into ZOOMS: pixels per minute
const capacity = ref(100);
const follow = ref(true);
const viewEnd = ref(0); // program time at the right edge when not following

const M = { left: 38, right: 12, top: 20, bottom: 40 };
const STRIP = 14;

const pxPerMin = computed(() => ZOOMS[zoom.value]);
const plotW = computed(() => Math.max(20, width.value - M.left - M.right));
const plotH = computed(() => Math.max(20, height.value - M.top - M.bottom));
const span = computed(() => (plotW.value / pxPerMin.value) * 60);

function visibleRange(): [number, number] {
  const now = sim.snapshot?.t ?? 0;
  const end = follow.value ? Math.max(now, span.value) : Math.min(viewEnd.value, Math.max(now, span.value));
  return [end - span.value, end];
}

let frame = 0;
function requestDraw() {
  if (!frame) frame = requestAnimationFrame(draw);
}

function draw() {
  frame = 0;
  const c = canvas.value;
  if (!c || width.value === 0) return;
  const dpr = window.devicePixelRatio || 1;
  const w = width.value;
  const h = height.value;
  if (c.width !== Math.round(w * dpr) || c.height !== Math.round(h * dpr)) {
    c.width = Math.round(w * dpr);
    c.height = Math.round(h * dpr);
  }
  const g = c.getContext("2d");
  if (!g) return;
  g.setTransform(dpr, 0, 0, dpr, 0, 0);
  g.clearRect(0, 0, w, h);

  const few = prefs.prefs.fewerColours;
  const inkPen = few ? "#000000" : "#1d4f91";
  const inkTick = few ? "#000000" : "#c0392b";
  const [t0, t1] = visibleRange();
  const x = (t: number) => M.left + ((t - t0) / (t1 - t0)) * plotW.value;
  const top = M.top;
  const bottom = M.top + plotH.value;
  const y = (v: number) => bottom - (v / capacity.value) * plotH.value;

  // paper
  g.fillStyle = "#fffdf5";
  g.fillRect(M.left, top, plotW.value, plotH.value);
  g.strokeStyle = "#c9c2a8";
  g.lineWidth = 1;
  g.strokeRect(M.left + 0.5, top + 0.5, plotW.value, plotH.value);

  // grid and axis labels
  g.font = "10.5px system-ui, sans-serif";
  g.fillStyle = "#5d6b7a";
  g.textAlign = "right";
  for (const v of [0, 0.25, 0.5, 0.75, 1]) {
    const yy = Math.round(y(v * capacity.value)) + 0.5;
    g.strokeStyle = "#efe9d6";
    g.beginPath();
    g.moveTo(M.left, yy);
    g.lineTo(M.left + plotW.value, yy);
    g.stroke();
    g.fillText(String(Math.round(v * capacity.value)), M.left - 5, yy + 3.5);
  }
  const tickMin = niceTickMinutes(pxPerMin.value);
  g.textAlign = "center";
  for (let m = Math.ceil(t0 / 60 / tickMin) * tickMin; m * 60 <= t1; m += tickMin) {
    const xx = Math.round(x(m * 60)) + 0.5;
    g.strokeStyle = "#efe9d6";
    g.beginPath();
    g.moveTo(xx, top);
    g.lineTo(xx, bottom);
    g.stroke();
    g.fillText(`${+m.toFixed(1)}`, xx, h - 6);
  }
  g.textAlign = "right";
  g.fillText("min", M.left - 5, h - 6);

  const s = sim.session;

  // event strip: CS presentations and shocks
  const stripY = bottom + 4;
  g.strokeStyle = "#7d8996";
  g.beginPath();
  g.moveTo(M.left, stripY + STRIP - 2);
  g.lineTo(M.left + plotW.value, stripY + STRIP - 2);
  g.stroke();
  for (const cs of s.cs) {
    const off = cs.off ?? sim.snapshot?.t ?? t1;
    if (off < t0 || cs.on > t1) continue;
    g.fillStyle = few ? "#555555" : "#d9822b";
    g.fillRect(x(cs.on), stripY + 2, Math.max(1.5, x(off) - x(cs.on)), STRIP - 5);
  }
  g.strokeStyle = few ? "#000000" : "#b8213a";
  g.lineWidth = 2;
  for (const sh of s.shocks) {
    if (sh.t < t0 || sh.t > t1) continue;
    const xx = x(sh.t);
    g.beginPath();
    g.moveTo(xx, stripY);
    g.lineTo(xx, stripY + STRIP);
    g.stroke();
  }

  // notes: marks, design changes, time off
  g.save();
  g.textAlign = "left";
  g.beginPath();
  g.rect(M.left, 0, plotW.value, h);
  g.clip();
  g.lineWidth = 1;
  for (const n of s.notes) {
    if (n.t < t0 || n.t > t1) continue;
    const xx = Math.round(x(n.t)) + 0.5;
    if (n.kind === "timeOff" || n.kind === "design" || n.kind === "classical") {
      g.strokeStyle = n.kind === "timeOff" ? "#7d5fb2" : "#8a96a5";
      g.setLineDash([3, 3]);
      g.beginPath();
      g.moveTo(xx, top);
      g.lineTo(xx, bottom);
      g.stroke();
      g.setLineDash([]);
    } else {
      g.fillStyle = "#3a4a5d";
      g.beginPath();
      g.moveTo(xx - 4, top - 9);
      g.lineTo(xx + 4, top - 9);
      g.lineTo(xx, top - 1);
      g.fill();
    }
    g.fillStyle = n.kind === "timeOff" ? "#7d5fb2" : "#3a4a5d";
    g.fillText(n.label, xx + 5, top - 4);
  }
  g.restore();

  // pen trace
  g.save();
  g.beginPath();
  g.rect(M.left, top - 1, plotW.value, plotH.value + 2);
  g.clip();
  g.strokeStyle = inkPen;
  g.lineWidth = 1.4;
  g.lineJoin = "round";
  const segments = penSegments(s.presses, t0, Math.min(t1, sim.snapshot?.t ?? t1), capacity.value);
  for (let i = 0; i < segments.length; i++) {
    const seg = segments[i];
    g.beginPath();
    g.moveTo(x(seg[0].t), y(seg[0].y));
    for (const p of seg) g.lineTo(x(p.t), y(p.y));
    g.stroke();
    if (i < segments.length - 1) {
      // the reset stroke
      const last = seg[seg.length - 1];
      g.globalAlpha = 0.35;
      g.beginPath();
      g.moveTo(x(last.t), y(capacity.value));
      g.lineTo(x(last.t), y(0));
      g.stroke();
      g.globalAlpha = 1;
    }
  }
  // reinforcement ticks
  g.lineWidth = 1.6;
  const first = lowerBound(
    s.reinforcers.map((r) => r.t),
    t0,
  );
  for (let i = first; i < s.reinforcers.length; i++) {
    const r = s.reinforcers[i];
    if (r.t > t1) break;
    const xx = x(r.t);
    const yy = y(penHeightAt(s.presses, r.t, capacity.value));
    g.strokeStyle = r.kind === "manual" ? "#7d8996" : inkTick;
    g.beginPath();
    g.moveTo(xx, yy);
    g.lineTo(xx + 5, yy + 5);
    g.stroke();
  }
  g.restore();
}

watch(
  [
    () => sim.dataVersion,
    () => sim.snapshot?.t,
    width,
    height,
    zoom,
    capacity,
    follow,
    viewEnd,
    () => prefs.prefs.fewerColours,
  ],
  requestDraw,
);
watch(
  () => sim.resetVersion,
  () => {
    follow.value = true;
  },
);

function onWheel(e: WheelEvent) {
  if (e.ctrlKey || e.metaKey) {
    zoom.value = Math.min(ZOOMS.length - 1, Math.max(0, zoom.value + (e.deltaY < 0 ? 1 : -1)));
    return;
  }
  const delta = ((e.deltaX || e.deltaY) / pxPerMin.value) * 60;
  pan(delta);
}

function pan(seconds: number) {
  const now = sim.snapshot?.t ?? 0;
  const [, end] = visibleRange();
  const next = Math.min(Math.max(now, span.value), Math.max(span.value, end + seconds));
  viewEnd.value = next;
  follow.value = next >= Math.max(now, span.value) - 0.5;
}

let dragX: number | null = null;
function onDown(e: PointerEvent) {
  dragX = e.clientX;
  (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
}
function onMove(e: PointerEvent) {
  if (dragX === null) return;
  const dx = e.clientX - dragX;
  dragX = e.clientX;
  pan((-dx / pxPerMin.value) * 60);
}
function onUp() {
  dragX = null;
}

function toggleFollow() {
  follow.value = !follow.value;
  if (!follow.value) viewEnd.value = visibleRange()[1];
}

function minuteTable(): string {
  const s = sim.session;
  const end = sim.snapshot?.t ?? 0;
  const rows: (number | string)[][] = [];
  let cumulative = 0;
  for (let m = 0; m * 60 < end; m++) {
    const a = m * 60;
    const b = a + 60;
    const presses = lowerBound(s.presses, b) - lowerBound(s.presses, a);
    const rf = s.reinforcers.filter((r) => r.t >= a && r.t < b).length;
    cumulative += presses;
    rows.push([m + 1, presses, rf, cumulative]);
  }
  return toTsv(["minute", "presses", "reinforcers", "cumulative presses"], rows);
}

let unregister = () => {};
onMounted(() => {
  unregister = registerCopy("cumulative", minuteTable);
  requestDraw();
});
onBeforeUnmount(() => {
  unregister();
  cancelAnimationFrame(frame);
});
</script>

<template>
  <div class="record">
    <div class="controls">
      <button title="Zoom out" :disabled="zoom === 0" @click="zoom--">−</button>
      <span class="muted">{{ pxPerMin }} px/min</span>
      <button title="Zoom in" :disabled="zoom === ZOOMS.length - 1" @click="zoom++">+</button>
      <label class="row" title="Responses per sweep of the pen">
        Sweep
        <select v-model.number="capacity">
          <option :value="50">50</option>
          <option :value="100">100</option>
          <option :value="200">200</option>
          <option :value="500">500</option>
        </select>
      </label>
      <button :class="{ primary: follow }" title="Keep the newest data in view" @click="toggleFollow">Follow</button>
      <span class="spacer" />
      <span class="legend"><i class="tick" /> reinforcement</span>
      <span class="legend"><i class="cs" /> CS</span>
      <span class="legend"><i class="shock" /> shock</span>
    </div>
    <div
      ref="box"
      class="canvas-box"
      @wheel.prevent="onWheel"
      @pointerdown="onDown"
      @pointermove="onMove"
      @pointerup="onUp"
    >
      <canvas ref="canvas" :style="{ width: `${width}px`, height: `${height}px` }" />
    </div>
  </div>
</template>

<style scoped>
.record {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
}
.controls {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  border-bottom: 1px solid #dde3ea;
  white-space: nowrap;
  overflow: hidden;
}
.controls button {
  padding: 1px 8px;
}
.spacer {
  flex: 1;
}
.legend {
  color: var(--muted);
  font-size: 11.5px;
}
.legend i {
  display: inline-block;
  width: 12px;
  height: 8px;
  margin-right: 3px;
  vertical-align: 0;
}
.tick {
  border-bottom: 2px solid #c0392b;
  transform: rotate(45deg);
}
.cs {
  background: #d9822b;
}
.shock {
  width: 3px !important;
  background: #b8213a;
}
.canvas-box {
  flex: 1;
  min-height: 0;
  position: relative;
  cursor: grab;
}
canvas {
  position: absolute;
  inset: 0;
}
</style>
