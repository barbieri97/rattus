<script setup lang="ts">
// What the rat is doing now and has been doing, with session counters.
import { computed, onBeforeUnmount, onMounted } from "vue";
import { registerCopy } from "../../actions";
import { useSimStore } from "../../stores/sim";
import { formatClock } from "../../utils/format";
import { BEHAVIOR_LABELS, designLabel } from "../../utils/labels";
import { toTsv } from "../../utils/tsv";

const sim = useSimStore();
const snap = computed(() => sim.snapshot);
const recent = computed(() => [...sim.behaviorLog].reverse());

let unregister = () => {};
onMounted(() => {
  unregister = registerCopy("behavior", () =>
    toTsv(
      ["time (s)", "behavior"],
      sim.behaviorLog.map((b) => [b.t, BEHAVIOR_LABELS[b.behavior]]),
    ),
  );
});
onBeforeUnmount(() => unregister());
</script>

<template>
  <div v-if="snap" class="log">
    <div class="now">
      <span class="muted">Now: </span>
      <strong>{{ sim.host.isolated ? "(isolated)" : BEHAVIOR_LABELS[snap.rat.behavior] }}</strong>
    </div>
    <div class="counters">
      <div>
        <span>{{ snap.counters.presses }}</span
        >bar presses
      </div>
      <div>
        <span>{{ snap.counters.reinforcers }}</span
        >reinforcements
      </div>
      <div>
        <span>{{ snap.counters.pelletsEaten }}</span
        >pellets eaten
      </div>
    </div>
    <div class="muted design">Schedule: {{ designLabel(snap.operant) }}</div>
    <ol class="entries">
      <li v-for="(b, i) in recent" :key="`${b.t}-${i}`">
        <span class="time">{{ formatClock(b.t) }}</span>
        {{ BEHAVIOR_LABELS[b.behavior] }}
      </li>
    </ol>
  </div>
</template>

<style scoped>
.log {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  padding: 6px 10px;
  gap: 4px;
}
.now {
  font-size: 15px;
}
.counters {
  display: flex;
  gap: 14px;
  color: var(--muted);
}
.counters span {
  display: block;
  font-size: 17px;
  font-weight: 600;
  color: var(--text);
  font-variant-numeric: tabular-nums;
}
.design {
  font-size: 12px;
}
.entries {
  flex: 1;
  min-height: 0;
  overflow: auto;
  margin: 2px 0 0;
  padding: 0;
  list-style: none;
  border-top: 1px solid #dde3ea;
}
.entries li {
  padding: 2px 0;
  border-bottom: 1px solid #f0f2f5;
}
.time {
  display: inline-block;
  width: 64px;
  color: var(--muted);
  font-variant-numeric: tabular-nums;
}
</style>
