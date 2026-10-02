<script setup lang="ts">
// Design Classical Conditioning Experiment: stages of trials pairing conditioned stimuli
// (light, tone, bell) with shock, presented while the rat presses the bar.
import { computed, ref } from "vue";
import * as actions from "../../actions";
import type { TrialType } from "../../bindings/TrialType";
import type { UsLevel } from "../../bindings/UsLevel";
import { useSimStore } from "../../stores/sim";
import { useUiStore } from "../../stores/ui";
import {
  cloneDesign,
  defaultDesign,
  defaultStage,
  defaultTrialType,
  estimatedMinutes,
  totalTrials,
  validateDesign,
} from "../../utils/classical";
import { US_LABELS } from "../../utils/labels";
import ModalDialog from "./ModalDialog.vue";

const sim = useSimStore();
const ui = useUiStore();
const draft = ref(cloneDesign(sim.classicalDesign ?? defaultDesign()));
const US_LEVELS: UsLevel[] = ["none", "low", "medium", "high"];

const error = computed(() => validateDesign(draft.value));
const running = computed(() => sim.classical !== null);
const notPressing = computed(() => (sim.snapshot?.mind.barStrength ?? 0) < 0.3);
const summary = computed(() => {
  const minutes = estimatedMinutes(draft.value);
  const time = minutes >= 90 ? `${(minutes / 60).toFixed(1)} h` : `${Math.round(minutes)} min`;
  return `${totalTrials(draft.value)} trials, about ${time} of program time`;
});

function toggleTone(t: TrialType, on: boolean) {
  t.cs.toneDb = on ? 80 : null;
}

function addStage() {
  const last = draft.value.stages[draft.value.stages.length - 1];
  draft.value.stages.push(last ? JSON.parse(JSON.stringify(last)) : defaultStage());
}

async function save(): Promise<boolean> {
  if (error.value) return false;
  return sim.command({ type: "setClassicalDesign", design: cloneDesign(draft.value) });
}

async function saveOnly() {
  if (await save()) {
    await ui.closeDialog();
    ui.notify("Classical conditioning design saved. Run it from the Experiment menu.");
  }
}

async function saveAndRun() {
  if (!(await save())) return;
  await ui.closeDialog();
  await actions.startClassical();
}

async function stop() {
  await actions.stopClassical();
}

async function barTrained() {
  await ui.closeDialog();
  await actions.newRat("barTrained");
}
</script>

<template>
  <ModalDialog title="Design Classical Conditioning Experiment" :width="720" @close="ui.closeDialog()">
    <p class="muted intro">
      Each trial presents the conditioned stimulus (CS) for 30 s; when a shock is chosen, it comes in the last second.
      Trials of a stage are presented in random order, separated by intervals that vary around the mean. The suppression
      ratio compares bar presses during the CS with the 30 s before it.
    </p>
    <div v-if="notPressing" class="warning">
      This rat is not pressing the bar, so suppression ratios cannot be measured (the movement ratio still works).
      <button @click="barTrained">Use a bar-trained rat</button>
    </div>
    <div v-if="running" class="warning">
      An experiment is running. Saving changes the design for the next run.
      <button @click="stop">Stop the experiment</button>
    </div>

    <div v-for="(stage, si) in draft.stages" :key="si" class="stage">
      <div class="stage-head row">
        <strong>Stage {{ si + 1 }}</strong>
        <label class="row">
          Mean interval between trials
          <input v-model.number="stage.meanItiMinutes" type="number" min="1" max="60" step="0.5" />
          min
        </label>
        <span class="spacer" />
        <button :disabled="draft.stages.length <= 1" title="Remove this stage" @click="draft.stages.splice(si, 1)">
          Remove stage
        </button>
      </div>
      <table>
        <thead>
          <tr>
            <th>Light</th>
            <th>Tone</th>
            <th>Bell</th>
            <th>US</th>
            <th>Trials</th>
            <th />
          </tr>
        </thead>
        <tbody>
          <tr v-for="(t, ti) in stage.trialTypes" :key="ti">
            <td><input v-model="t.cs.light" type="checkbox" /></td>
            <td class="tone">
              <input
                type="checkbox"
                :checked="t.cs.toneDb !== null"
                @change="toggleTone(t, ($event.target as HTMLInputElement).checked)"
              />
              <template v-if="t.cs.toneDb !== null">
                <input v-model.number="t.cs.toneDb" type="number" min="60" max="100" step="5" /> dB
              </template>
            </td>
            <td><input v-model="t.cs.bell" type="checkbox" /></td>
            <td>
              <select v-model="t.us">
                <option v-for="u in US_LEVELS" :key="u" :value="u">{{ US_LABELS[u] }}</option>
              </select>
            </td>
            <td><input v-model.number="t.count" type="number" min="1" max="200" step="1" /></td>
            <td>
              <button
                :disabled="stage.trialTypes.length <= 1"
                title="Remove this trial type"
                @click="stage.trialTypes.splice(ti, 1)"
              >
                ✕
              </button>
            </td>
          </tr>
        </tbody>
      </table>
      <button :disabled="stage.trialTypes.length >= 4" @click="stage.trialTypes.push(defaultTrialType())">
        + Trial type
      </button>
    </div>
    <div class="row">
      <button :disabled="draft.stages.length >= 20" @click="addStage">+ Add stage</button>
      <span class="muted">{{ summary }}</span>
    </div>
    <p v-if="error" class="error">{{ error }}</p>

    <template #footer>
      <button class="left" @click="draft = defaultDesign()">Default</button>
      <button @click="ui.closeDialog()">Cancel</button>
      <button :disabled="!!error" @click="saveOnly">Save Design</button>
      <button class="primary" :disabled="!!error || running" @click="saveAndRun">Save and Run</button>
    </template>
  </ModalDialog>
</template>

<style scoped>
.intro {
  margin: 0 0 10px;
}
.warning {
  background: #fff4dc;
  border: 1px solid #e8c779;
  border-radius: 6px;
  padding: 6px 10px;
  margin-bottom: 10px;
  display: flex;
  align-items: center;
  gap: 10px;
  justify-content: space-between;
}
.stage {
  border: 1px solid #dde3ea;
  border-radius: 6px;
  padding: 8px 10px;
  margin-bottom: 10px;
  background: #ffffff;
}
.stage-head {
  margin-bottom: 6px;
}
.spacer {
  flex: 1;
}
table {
  border-collapse: collapse;
  margin-bottom: 6px;
}
th {
  font-weight: normal;
  color: var(--muted);
  text-align: left;
  padding: 2px 10px 2px 0;
}
td {
  padding: 3px 10px 3px 0;
}
.tone input[type="number"] {
  width: 4.5em;
}
.error {
  color: var(--danger);
  margin: 8px 0 0;
}
</style>
