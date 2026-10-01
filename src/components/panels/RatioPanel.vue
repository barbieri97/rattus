<script setup lang="ts">
// Suppression ratio (bar presses) or movement ratio (time moving) for every trial:
// during the CS divided by (during the CS + the same time just before it).
// 0.5 means the CS changed nothing; 0 means complete suppression.
import { computed, onBeforeUnmount, onMounted } from "vue";
import { registerCopy } from "../../actions";
import { useSimStore } from "../../stores/sim";
import { csLabel, US_LABELS } from "../../utils/labels";
import { usePalette } from "../../utils/palette";
import { toTsv } from "../../utils/tsv";
import LineChart, { type ChartSeries } from "../charts/LineChart.vue";
import EmptyNote from "./EmptyNote.vue";

const props = defineProps<{ kind: "suppression" | "movement" }>();
const sim = useSimStore();
const palette = usePalette();

const value = (r: (typeof sim.trials)[number]) => (props.kind === "suppression" ? r.suppressionRatio : r.movementRatio);

const series = computed<ChartSeries[]>(() => {
  void sim.dataVersion;
  return [
    {
      name: props.kind === "suppression" ? "Suppression ratio" : "Movement ratio",
      style: palette.value[0],
      points: sim.trials.map((r) => ({ x: r.trial, y: value(r) })),
    },
  ];
});

const dividers = computed(() => {
  void sim.dataVersion;
  return sim.trials.filter((r, i) => i > 0 && r.stage !== sim.trials[i - 1].stage).map((r) => r.trial - 0.5);
});

const last = computed(() => {
  void sim.dataVersion;
  const r = sim.trials[sim.trials.length - 1];
  if (!r) return "";
  const v = value(r);
  const detail =
    props.kind === "suppression"
      ? `${r.csPresses} presses during the CS, ${r.prePresses} before`
      : `${r.csMove.toFixed(1)} s moving during the CS, ${r.preMove.toFixed(1)} s before`;
  return `Trial ${r.trial} (stage ${r.stage}, ${csLabel(r.cs)}, ${US_LABELS[r.us].toLowerCase()}): ${v === null ? "–" : v.toFixed(2)} · ${detail}`;
});

let unregister = () => {};
onMounted(() => {
  unregister = registerCopy(props.kind, () =>
    props.kind === "suppression"
      ? toTsv(
          ["trial", "stage", "CS", "US", "pre-CS presses", "CS presses", "suppression ratio"],
          sim.trials.map((r) => [
            r.trial,
            r.stage,
            csLabel(r.cs),
            US_LABELS[r.us],
            r.prePresses,
            r.csPresses,
            r.suppressionRatio,
          ]),
        )
      : toTsv(
          ["trial", "stage", "CS", "US", "pre-CS moving (s)", "CS moving (s)", "movement ratio"],
          sim.trials.map((r) => [
            r.trial,
            r.stage,
            csLabel(r.cs),
            US_LABELS[r.us],
            r.preMove,
            r.csMove,
            r.movementRatio,
          ]),
        ),
  );
});
onBeforeUnmount(() => unregister());
</script>

<template>
  <div class="ratio">
    <template v-if="sim.trials.length">
      <div class="chart">
        <LineChart
          :series="series"
          x-label="trial"
          :y-label="kind === 'suppression' ? 'suppression ratio' : 'movement ratio'"
          :y-min="0"
          :y-max="1"
          :ref-lines="[0.5]"
          :dividers="dividers"
          integer-x
        />
      </div>
      <div class="last">{{ last }}</div>
    </template>
    <EmptyNote v-else />
  </div>
</template>

<style scoped>
.ratio {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  padding: 6px 10px 6px;
}
.chart {
  flex: 1;
  min-height: 0;
}
.last {
  color: var(--muted);
  font-size: 12px;
  padding-top: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
