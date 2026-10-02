<script setup lang="ts">
// Design Operant Conditioning Experiment: reinforcement schedule and reinforcer for bar presses.
import { computed, ref } from "vue";
import type { Reinforcer } from "../../bindings/Reinforcer";
import type { Schedule } from "../../bindings/Schedule";
import { useSimStore } from "../../stores/sim";
import { useUiStore } from "../../stores/ui";
import ModalDialog from "./ModalDialog.vue";

type Kind = Schedule["kind"];
const sim = useSimStore();
const ui = useUiStore();

const current = sim.snapshot?.operant ?? { schedule: { kind: "continuous" }, reinforcer: "food" };
const kind = ref<Kind>(current.schedule.kind);
const ratio = ref(
  current.schedule.kind === "fixedRatio" || current.schedule.kind === "variableRatio" ? current.schedule.n : 10,
);
const interval = ref(
  current.schedule.kind === "fixedInterval" || current.schedule.kind === "variableInterval"
    ? current.schedule.seconds
    : 30,
);
const reinforcer = ref<Reinforcer>(current.reinforcer);

const KINDS: { kind: Kind; label: string; help: string }[] = [
  { kind: "continuous", label: "Continuous (CRF)", help: "Every press is reinforced." },
  { kind: "fixedRatio", label: "Fixed ratio (FR)", help: "Every n-th press is reinforced." },
  { kind: "variableRatio", label: "Variable ratio (VR)", help: "On average every n-th press is reinforced." },
  { kind: "fixedInterval", label: "Fixed interval (FI)", help: "The first press after t seconds is reinforced." },
  {
    kind: "variableInterval",
    label: "Variable interval (VI)",
    help: "The first press after an interval averaging t seconds is reinforced.",
  },
];

const isRatio = computed(() => kind.value === "fixedRatio" || kind.value === "variableRatio");
const isInterval = computed(() => kind.value === "fixedInterval" || kind.value === "variableInterval");
const valid = computed(() => {
  if (isRatio.value) return Number.isInteger(ratio.value) && ratio.value >= 1 && ratio.value <= 500;
  if (isInterval.value) return interval.value >= 1 && interval.value <= 3600;
  return true;
});

function schedule(): Schedule {
  switch (kind.value) {
    case "fixedRatio":
      return { kind: "fixedRatio", n: ratio.value };
    case "variableRatio":
      return { kind: "variableRatio", n: ratio.value };
    case "fixedInterval":
      return { kind: "fixedInterval", seconds: interval.value };
    case "variableInterval":
      return { kind: "variableInterval", seconds: interval.value };
    default:
      return { kind: "continuous" };
  }
}

async function apply() {
  if (!valid.value) return;
  const ok = await sim.command({
    type: "setOperantDesign",
    design: { schedule: schedule(), reinforcer: reinforcer.value },
  });
  if (ok) {
    await ui.closeDialog();
    ui.notify("New operant design in effect.");
  }
}
</script>

<template>
  <ModalDialog title="Design Operant Conditioning Experiment" :width="540" @close="ui.closeDialog()">
    <fieldset>
      <legend>Reinforcement schedule for bar pressing</legend>
      <label v-for="k in KINDS" :key="k.kind" class="choice">
        <input v-model="kind" type="radio" :value="k.kind" />
        <strong>{{ k.label }}</strong> <span class="muted">{{ k.help }}</span>
      </label>
      <div class="row value">
        <template v-if="isRatio">
          <span>n =</span>
          <input v-model.number="ratio" type="number" min="1" max="500" step="1" />
          <span class="muted">responses (1–500)</span>
        </template>
        <template v-else-if="isInterval">
          <span>t =</span>
          <input v-model.number="interval" type="number" min="1" max="3600" step="1" />
          <span class="muted">seconds (1–3600)</span>
        </template>
      </div>
    </fieldset>
    <fieldset>
      <legend>Reinforcer</legend>
      <label class="choice"
        ><input v-model="reinforcer" type="radio" value="food" /><strong>Food</strong>
        <span class="muted">The dispenser clicks and drops a pellet.</span></label
      >
      <label class="choice"
        ><input v-model="reinforcer" type="radio" value="soundOnly" /><strong>Sound only</strong>
        <span class="muted">The dispenser clicks but drops nothing (secondary reinforcement).</span></label
      >
      <label class="choice"
        ><input v-model="reinforcer" type="radio" value="none" /><strong>None</strong>
        <span class="muted">Presses have no consequence (extinction).</span></label
      >
    </fieldset>
    <p class="muted note">
      You can always give pellets by hand with the Space bar or by clicking the lever, for magazine training and
      shaping.
    </p>
    <template #footer>
      <button @click="ui.closeDialog()">Cancel</button>
      <button class="primary" :disabled="!valid" @click="apply">Apply</button>
    </template>
  </ModalDialog>
</template>

<style scoped>
fieldset {
  border: 1px solid #dde3ea;
  border-radius: 6px;
  margin: 0 0 10px;
  padding: 6px 12px 8px;
}
legend {
  color: var(--muted);
  padding: 0 4px;
}
.choice {
  display: block;
  margin: 5px 0;
}
.value {
  min-height: 28px;
  margin-top: 6px;
  padding-left: 22px;
}
.note {
  margin: 4px 0 0;
}
</style>
