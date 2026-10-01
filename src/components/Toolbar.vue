<script setup lang="ts">
// Quick controls: run/pause, speed, isolation, pellet, mark, and the program clock.
import { computed } from "vue";
import * as actions from "../actions";
import { useSimStore } from "../stores/sim";
import { useUiStore } from "../stores/ui";
import { formatClock } from "../utils/format";
import { designLabel } from "../utils/labels";

const sim = useSimStore();
const ui = useUiStore();
const SPEEDS = [0.5, 1, 2, 4, 8];

const clock = computed(() => formatClock(sim.snapshot?.t ?? 0));
const schedule = computed(() => (sim.snapshot ? designLabel(sim.snapshot.operant) : ""));
const classical = computed(() => {
  const c = sim.classical;
  if (!c) return "";
  const phase = c.inCs ? `CS on, ${Math.ceil(c.remaining)} s left` : `next CS in ${Math.ceil(c.remaining)} s`;
  return `Stage ${c.stage}/${c.stageCount} · Trial ${c.trialInStage}/${c.trialsInStage} · ${phase}`;
});

function onSpeed(e: Event) {
  void actions.setSpeed(Number((e.target as HTMLSelectElement).value));
}
</script>

<template>
  <div class="toolbar">
    <button class="run" :title="sim.host.paused ? 'Resume (P)' : 'Pause (P)'" @click="actions.togglePause">
      {{ sim.host.paused ? "▶ Run" : "❚❚ Pause" }}
    </button>
    <label class="row speed" title="Animation speed while the rat is visible">
      Speed
      <select :value="sim.host.speed" :disabled="sim.host.isolated" @change="onSpeed">
        <option v-for="s in SPEEDS" :key="s" :value="s">{{ s }}×</option>
      </select>
    </label>
    <button
      :class="{ primary: sim.host.isolated }"
      title="Isolate the rat to accelerate time"
      @click="actions.toggleIsolation"
    >
      {{ sim.host.isolated ? "⏩ Accelerated" : "⏩ Isolate" }}
    </button>
    <span class="divider" />
    <button title="Operate the food dispenser (Space)" @click="actions.givePellet">🍽 Give pellet</button>
    <button title="Mark the cumulative record" @click="actions.markRecord">⚑ Mark</button>
    <span class="divider" />
    <button @click="ui.openDialog('operantDesign')">Operant…</button>
    <button @click="ui.openDialog('classicalDesign')">Classical…</button>
    <span class="spacer" />
    <span class="status" :title="schedule">{{ schedule }}</span>
    <span v-if="classical" class="status classical">{{ classical }}</span>
    <span class="clock" title="Program time">{{ clock }}</span>
  </div>
</template>

<style scoped>
.toolbar {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 8px;
  background: #f3f5f8;
  border-bottom: 1px solid var(--panel-border);
  white-space: nowrap;
  overflow: hidden;
}
.run {
  min-width: 82px;
}
.divider {
  width: 1px;
  height: 20px;
  background: #c5cdd7;
  margin: 0 2px;
}
.spacer {
  flex: 1;
}
.status {
  color: var(--muted);
  overflow: hidden;
  text-overflow: ellipsis;
}
.classical {
  color: #7a4b12;
}
.clock {
  font-variant-numeric: tabular-nums;
  font-weight: 600;
  background: #1f2a36;
  color: #e9f1fa;
  padding: 2px 8px;
  border-radius: 4px;
  min-width: 72px;
  text-align: center;
}
.speed select {
  padding: 1px 2px;
}
</style>
