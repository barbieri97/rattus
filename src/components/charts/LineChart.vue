<script setup lang="ts">
// A small line chart in SVG, sized to its container.
import { computed, ref } from "vue";
import { useElementSize } from "../../composables/useElementSize";
import type { SeriesStyle } from "../../utils/palette";

export interface ChartSeries {
  name: string;
  style: SeriesStyle;
  points: { x: number; y: number | null }[];
}

const props = withDefaults(
  defineProps<{
    series: ChartSeries[];
    xLabel?: string;
    yLabel?: string;
    yMin?: number;
    yMax?: number;
    xMin?: number;
    xMax?: number;
    refLines?: number[];
    dividers?: number[];
    markers?: boolean;
    integerX?: boolean;
  }>(),
  { xLabel: "", yLabel: "", yMin: 0, yMax: 1, refLines: () => [], dividers: () => [], markers: true, integerX: false },
);

const root = ref<HTMLElement | null>(null);
const { width, height } = useElementSize(root);
const M = { left: 42, right: 10, top: 10, bottom: 30 };

const xRange = computed(() => {
  const xs = props.series.flatMap((s) => s.points.map((p) => p.x));
  const lo = props.xMin ?? (xs.length ? Math.min(...xs) : 0);
  let hi = props.xMax ?? (xs.length ? Math.max(...xs) : 1);
  if (hi <= lo) hi = lo + 1;
  return { lo, hi };
});

const plotW = computed(() => Math.max(10, width.value - M.left - M.right));
const plotH = computed(() => Math.max(10, height.value - M.top - M.bottom));
const sx = (x: number) => M.left + ((x - xRange.value.lo) / (xRange.value.hi - xRange.value.lo)) * plotW.value;
const sy = (y: number) => {
  const c = Math.min(props.yMax, Math.max(props.yMin, y));
  return M.top + (1 - (c - props.yMin) / (props.yMax - props.yMin)) * plotH.value;
};

const paths = computed(() =>
  props.series.map((s) => {
    let d = "";
    let pen = false;
    for (const p of s.points) {
      if (p.y === null || !Number.isFinite(p.y)) {
        pen = false;
        continue;
      }
      d += `${pen ? "L" : "M"}${sx(p.x).toFixed(1)},${sy(p.y).toFixed(1)}`;
      pen = true;
    }
    return { d, style: s.style, name: s.name };
  }),
);

const dots = computed(() => {
  if (!props.markers) return [];
  const out: { x: number; y: number; style: SeriesStyle }[] = [];
  for (const s of props.series) {
    if (s.points.length > 150) continue;
    for (const p of s.points) {
      if (p.y !== null && Number.isFinite(p.y)) out.push({ x: sx(p.x), y: sy(p.y), style: s.style });
    }
  }
  return out;
});

function niceStep(span: number, target: number): number {
  const raw = span / Math.max(1, target);
  const mag = 10 ** Math.floor(Math.log10(raw));
  for (const m of [1, 2, 5, 10]) if (m * mag >= raw) return m * mag;
  return 10 * mag;
}

const xTicks = computed(() => {
  const { lo, hi } = xRange.value;
  let step = niceStep(hi - lo, Math.max(2, Math.floor(plotW.value / 70)));
  if (props.integerX) step = Math.max(1, Math.round(step));
  const out: number[] = [];
  for (let v = Math.ceil(lo / step) * step; v <= hi + 1e-9; v += step) out.push(+v.toFixed(6));
  return out;
});

const yTicks = computed(() => {
  const step = niceStep(props.yMax - props.yMin, Math.max(2, Math.floor(plotH.value / 36)));
  const out: number[] = [];
  for (let v = Math.ceil(props.yMin / step) * step; v <= props.yMax + 1e-9; v += step) out.push(+v.toFixed(6));
  return out;
});

const markerPath = (m: SeriesStyle["marker"], x: number, y: number) => {
  if (m === "square") return `M${x - 3},${y - 3}h6v6h-6z`;
  if (m === "triangle") return `M${x},${y - 4}L${x + 4},${y + 3}L${x - 4},${y + 3}z`;
  return `M${x - 3.2},${y}a3.2,3.2 0 1,0 6.4,0a3.2,3.2 0 1,0 -6.4,0`;
};
</script>

<template>
  <div class="wrap">
    <div v-if="series.length > 1" class="legend">
      <span v-for="s in series" :key="s.name">
        <svg width="22" height="10">
          <line
            x1="1"
            y1="5"
            x2="21"
            y2="5"
            :stroke="s.style.color"
            :stroke-dasharray="s.style.dash"
            stroke-width="2.5"
          />
        </svg>
        {{ s.name }}
      </span>
    </div>
    <div ref="root" class="chart">
      <svg v-if="width > 0 && height > 0" :width="width" :height="height">
        <rect :x="M.left" :y="M.top" :width="plotW" :height="plotH" fill="#ffffff" stroke="#c3ccd6" />
        <g class="grid">
          <line v-for="t in yTicks" :key="`y${t}`" :x1="M.left" :x2="M.left + plotW" :y1="sy(t)" :y2="sy(t)" />
        </g>
        <line
          v-for="r in refLines"
          :key="`r${r}`"
          class="ref"
          :x1="M.left"
          :x2="M.left + plotW"
          :y1="sy(r)"
          :y2="sy(r)"
        />
        <line
          v-for="d in dividers"
          :key="`d${d}`"
          class="divider"
          :x1="sx(d)"
          :x2="sx(d)"
          :y1="M.top"
          :y2="M.top + plotH"
        />
        <g class="axis">
          <text v-for="t in yTicks" :key="`yt${t}`" :x="M.left - 5" :y="sy(t) + 4" text-anchor="end">{{ t }}</text>
          <text v-for="t in xTicks" :key="`xt${t}`" :x="sx(t)" :y="M.top + plotH + 14" text-anchor="middle">
            {{ t }}
          </text>
          <text :x="M.left + plotW / 2" :y="height - 3" text-anchor="middle" class="axis-label">{{ xLabel }}</text>
          <text :transform="`translate(11, ${M.top + plotH / 2}) rotate(-90)`" text-anchor="middle" class="axis-label">
            {{ yLabel }}
          </text>
        </g>
        <path
          v-for="p in paths"
          :key="p.name"
          :d="p.d"
          fill="none"
          :stroke="p.style.color"
          :stroke-dasharray="p.style.dash"
          stroke-width="2"
          stroke-linejoin="round"
        />
        <path v-for="(d, i) in dots" :key="i" :d="markerPath(d.style.marker, d.x, d.y)" :fill="d.style.color" />
      </svg>
    </div>
  </div>
</template>

<style scoped>
.wrap {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
}
.chart {
  position: relative;
  flex: 1;
  min-height: 80px;
}
svg {
  display: block;
}
.grid line {
  stroke: #edf0f4;
}
.ref {
  stroke: #8a96a5;
  stroke-dasharray: 5 4;
}
.divider {
  stroke: #b0b9c4;
  stroke-dasharray: 2 3;
}
.axis text {
  font-size: 10.5px;
  fill: #5d6b7a;
}
.axis-label {
  font-size: 11px;
  fill: #3a4a5d;
}
.legend {
  display: flex;
  flex-wrap: wrap;
  gap: 2px 12px;
  padding: 2px 0 2px 40px;
  font-size: 11px;
  color: #3a4a5d;
}
.legend svg {
  display: inline-block;
  vertical-align: middle;
}
</style>
