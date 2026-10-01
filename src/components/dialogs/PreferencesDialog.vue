<script setup lang="ts">
import { ref } from "vue";
import { DEFAULT_PREFERENCES, usePrefsStore, type Preferences } from "../../stores/prefs";
import { useUiStore } from "../../stores/ui";
import ModalDialog from "./ModalDialog.vue";

const prefsStore = usePrefsStore();
const ui = useUiStore();
const draft = ref<Preferences>({ ...prefsStore.prefs });
const SPEEDS = [60, 300, 600, 1800, 3600];

function ok() {
  prefsStore.update(draft.value);
  void ui.closeDialog();
}
</script>

<template>
  <ModalDialog title="Preferences" :width="480" @close="ui.closeDialog()">
    <fieldset>
      <legend>Environment</legend>
      <div class="row slider">
        <span>Sound proofing</span>
        <span class="muted">Quiet</span>
        <input v-model.number="draft.soundProofing" type="range" min="0" max="1" step="0.05" />
        <span class="muted">Loud</span>
      </div>
      <label class="line"
        ><input v-model="draft.fewerColours" type="checkbox" />Fewer colours (easier to tell data points apart)</label
      >
      <label class="line"
        ><input v-model="draft.scaleChamber" type="checkbox" />Scale the chamber with its window</label
      >
    </fieldset>
    <fieldset>
      <legend>Opening files</legend>
      <label class="line"
        ><input v-model="draft.randomStartOnOpen" type="checkbox" />Place the rat near the centre of the cage in a
        random starting position</label
      >
    </fieldset>
    <fieldset>
      <legend>Accelerated time</legend>
      <label class="row">
        While the rat is isolated, time runs
        <select v-model.number="draft.acceleratedSpeed">
          <option v-for="s in SPEEDS" :key="s" :value="s">{{ s }}×</option>
        </select>
        faster than real time.
      </label>
    </fieldset>
    <template #footer>
      <button class="left" @click="draft = { ...DEFAULT_PREFERENCES }">Use Defaults</button>
      <button @click="ui.closeDialog()">Cancel</button>
      <button class="primary" @click="ok">OK</button>
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
.line {
  display: block;
  margin: 6px 0;
}
.slider input {
  flex: 1;
}
</style>
