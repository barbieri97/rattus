<script setup lang="ts">
// Mind window: the CS Response Strength of each conditioned stimulus, now and after each trial.
import { computed, onBeforeUnmount, onMounted } from "vue";
import { registerCopy } from "../../actions";
import { useSimStore } from "../../stores/sim";
import { csLabel } from "../../utils/labels";
import { usePalette } from "../../utils/palette";
import { toTsv } from "../../utils/tsv";
import BarMeter from "../charts/BarMeter.vue";
import LineChart, { type ChartSeries } from "../charts/LineChart.vue";

const sim = useSimStore();
const palette = usePalette();
const mind = computed(() => sim.snapshot?.mind ?? null);

const used = computed(() => {
  void sim.dataVersion;
  const t = sim.trials;
  return {
    light: t.some((r) => r.cs.light),
    tone: t.some((r) => r.cs.toneDb !== null),
    bell: t.some((r) => r.cs.bell),
  };
});

const series = computed<ChartSeries[]>(() => {
  void sim.dataVersion;
  const t = sim.trials;
  const out: ChartSeries[] = [];
  if (used.value.light)
    out.push({ name: "Light", style: palette.value[0], points: t.map((r) => ({ x: r.trial, y: r.vLight })) });
  if (used.value.tone)
    out.push({ name: "Tone", style: palette.value[1], points: t.map((r) => ({ x: r.trial, y: r.vTone })) });
  if (used.value.bell)
    out.push({ name: "Bell", style: palette.value[2], points: t.map((r) => ({ x: r.trial, y: r.vBell })) });
  return out;
});

const dividers = computed(() => {
  void sim.dataVersion;
  return sim.trials.filter((r, i) => i > 0 && r.stage !== sim.trials[i - 1].stage).map((r) => r.trial - 0.5);
});

let unregister = () => {};
onMounted(() => {
  unregister = registerCopy("csStrength", () =>
    toTsv(
      ["trial", "stage", "CS", "light", "tone", "bell"],
      sim.trials.map((r) => [r.trial, r.stage, csLabel(r.cs), r.vLight, r.vTone, r.vBell]),
    ),
  );
});
onBeforeUnmount(() => unregister());
</script>

<template>
  <div class="mind">
    <template v-if="mind">
      <BarMeter label="Light" :value="mind.vLight" :min="-0.5" :max="1" :color="palette[0].color" />
      <BarMeter label="Tone" :value="mind.vTone" :min="-0.5" :max="1" :color="palette[1].color" />
      <BarMeter label="Bell" :value="mind.vBell" :min="-0.5" :max="1" :color="palette[2].color" />
    </template>
    <div class="chart">
      <LineChart
        v-if="series.length"
        :series="series"
        x-label="trial"
        y-label="strength"
        :y-min="-0.5"
        :y-max="1"
        :ref-lines="[0]"
        :dividers="dividers"
        integer-x
      />
      <p v-else class="muted hint">Strength after each classical conditioning trial will be plotted here.</p>
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
