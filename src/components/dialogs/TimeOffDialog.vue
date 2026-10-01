<script setup lang="ts">
// Remove the rat from the chamber for a while (to its home cage), e.g. for spontaneous recovery.
import { ref } from "vue";
import { useSimStore } from "../../stores/sim";
import { useUiStore } from "../../stores/ui";
import ModalDialog from "./ModalDialog.vue";

const sim = useSimStore();
const ui = useUiStore();
const hours = ref(24);

async function apply() {
  if (!(hours.value >= 0.1 && hours.value <= 720)) return;
  if (await sim.command({ type: "timeOff", hours: hours.value })) {
    await ui.closeDialog();
    ui.notify(`The rat spent ${hours.value} h in its home cage and is back in the chamber.`);
  }
}
</script>

<template>
  <ModalDialog title="Remove Rat for Time Off" :width="440" @close="ui.closeDialog()">
    <p>The rat goes back to its home cage and returns after:</p>
    <label class="row">
      <input v-model.number="hours" type="number" min="0.1" max="720" step="1" />
      hours
    </label>
    <p class="muted">
      Extinction fades during the rest, so a response extinguished before the time off can recover partially afterwards
      (spontaneous recovery). The record is marked; the program clock does not jump.
    </p>
    <template #footer>
      <button @click="ui.closeDialog()">Cancel</button>
      <button class="primary" @click="apply">Remove Rat</button>
    </template>
  </ModalDialog>
</template>
