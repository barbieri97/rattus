<script setup lang="ts">
// Mind window: pain sensitivity (how long the next response to shock will last) and fear.
import { computed, onBeforeUnmount, onMounted } from "vue";
import { registerCopy } from "../../actions";
import { useSimStore } from "../../stores/sim";
import { usePalette } from "../../utils/palette";
import { toTsv } from "../../utils/tsv";
import BarMeter from "../charts/BarMeter.vue";
import LineChart, { type ChartSeries } from "../charts/LineChart.vue";

const sim = useSimStore();
const palette = usePalette();
const mind = computed(() => sim.snapshot?.mind ?? null);

const series = computed<ChartSeries[]>(() => {
  void sim.dataVersion;
  const t = sim.trials;
  if (!t.length) return [];
  return [
    { name: "Peak fear during CS", style: palette.value[3], points: t.map((r) => ({ x: r.trial, y: r.fearPeak })) },
    { name: "Pain sensitivity", style: palette.value[4], points: t.map((r) => ({ x: r.trial, y: r.painSensitivity })) },
  ];
});

let unregister = () => {};
onMounted(() => {
  unregister = registerCopy("sensitivityFear", () =>
    toTsv(
      ["trial", "stage", "peak fear", "pain sensitivity"],
      sim.trials.map((r) => [r.trial, r.stage, r.fearPeak, r.painSensitivity]),
    ),
  );
});
onBeforeUnmount(() => unregister());
</script>

<template>
  <div class="mind">
    <template v-if="mind">
      <BarMeter
        label="Pain sensitivity"
        :value="mind.painSensitivity"
        :min="0"
        :max="2"
        :color="palette[4].color"
        hint="Predicts how long the response to the next shock will last"
      />
      <BarMeter
        label="Fear"
        :value="mind.fear"
        :color="palette[3].color"
        hint="Likelihood of freezing and other fear behaviour right now"
      />
    </template>
    <div class="chart">
      <LineChart v-if="series.length" :series="series" x-label="trial" :y-min="0" :y-max="2" integer-x />
      <p v-else class="muted hint">Fear and pain sensitivity after each trial will be plotted here.</p>
    </div>
  </div>
</template>

<style scoped>
.mind {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  padding: 6px 10px 8px;
}
.chart {
  flex: 1;
  min-height: 0;
}
.hint {
  margin: 8px 0;
}
</style>
