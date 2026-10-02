<script setup lang="ts">
// Mind window for operant conditioning: what the rat has learned about the dispenser click,
// the bar, and how strongly each action has been reinforced.
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { registerCopy } from "../../actions";
import { useSimStore } from "../../stores/sim";
import { BEHAVIOR_LABELS } from "../../utils/labels";
import { usePalette } from "../../utils/palette";
import { toTsv } from "../../utils/tsv";
import BarMeter from "../charts/BarMeter.vue";
import LineChart, { type ChartSeries } from "../charts/LineChart.vue";

const sim = useSimStore();
const palette = usePalette();
const showActions = ref(false);
const mind = computed(() => sim.snapshot?.mind ?? null);

const series = computed<ChartSeries[]>(() => {
  void sim.dataVersion;
  const samples = sim.samples;
  const step = Math.max(1, Math.floor(samples.length / 400));
  const pick = samples.filter((_, i) => i % step === 0 || i === samples.length - 1);
  return [
    { name: "Sound–Food", style: palette.value[0], points: pick.map((s) => ({ x: s.t / 60, y: s.soundFood })) },
    { name: "Bar–Sound", style: palette.value[1], points: pick.map((s) => ({ x: s.t / 60, y: s.barSound })) },
    {
      name: "Bar press strength",
      style: palette.value[2],
      points: pick.map((s) => ({ x: s.t / 60, y: s.barStrength })),
    },
  ];
});

const actions = computed(() =>
  (mind.value?.actions ?? []).map((a) => ({ label: BEHAVIOR_LABELS[a.behavior], value: a.value })),
);

let unregister = () => {};
onMounted(() => {
  unregister = registerCopy("operant", () =>
    toTsv(
      ["time (min)", "sound-food", "bar-sound", "bar press strength"],
      sim.samples.map((s) => [s.t / 60, s.soundFood, s.barSound, s.barStrength]),
    ),
  );
});
onBeforeUnmount(() => unregister());
</script>

<template>
  <div class="mind">
    <template v-if="mind">
      <BarMeter
        label="Sound–Food"
        :value="mind.soundFood"
        :color="palette[0].color"
        hint="How strongly the dispenser click predicts food (built by magazine training)"
      />
      <BarMeter
        label="Bar–Sound"
        :value="mind.barSound"
        :color="palette[1].color"
        hint="How strongly pressing the bar predicts the click"
      />
      <BarMeter
        label="Action strength"
        :value="mind.barStrength"
        :color="palette[2].color"
        hint="Strength of bar pressing, increased by reinforcing it"
      />
      <label class="toggle"
        ><input v-model="showActions" type="checkbox" /> Show the strength of every action (for shaping)</label
      >
      <div v-if="showActions" class="actions">
        <BarMeter v-for="a in actions" :key="a.label" :label="a.label" :value="a.value" :color="palette[3].color" />
      </div>
      <div v-else class="history">
        <LineChart :series="series" x-label="minutes" :y-min="0" :y-max="1" :markers="false" />
      </div>
    </template>
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
.toggle {
  margin: 4px 0;
  color: var(--muted);
}
.history {
  flex: 1;
  min-height: 0;
}
.actions {
  flex: 1;
  overflow: auto;
}
</style>
