<script setup lang="ts">
import { getVersion } from "@tauri-apps/api/app";
import { onMounted, ref } from "vue";
import { inTauri } from "../../api/backend";
import { useUiStore } from "../../stores/ui";
import ModalDialog from "./ModalDialog.vue";

const ui = useUiStore();
const version = ref("");
onMounted(async () => {
  if (inTauri()) version.value = await getVersion();
});
</script>

<template>
  <ModalDialog title="About Rattus" :width="460" @close="ui.closeDialog()">
    <div class="about">
      <img src="/favicon.svg" alt="" width="72" height="72" />
      <div>
        <h2>
          Rattus <span class="muted">{{ version }}</span>
        </h2>
        <p>A virtual rat in an operant chamber, for learning about classical and operant conditioning.</p>
        <p>Open source under the MIT License.<br />github.com/barbieri97/rattus</p>
        <p class="muted small">
          Inspired by the educational program "Sniffy, the Virtual Rat". Rattus is an independent project and is not
          affiliated with its authors or publishers.
        </p>
      </div>
    </div>
    <template #footer>
      <button class="primary" @click="ui.closeDialog()">OK</button>
    </template>
  </ModalDialog>
</template>

<style scoped>
.about {
  display: flex;
  gap: 16px;
}
h2 {
  margin: 0 0 6px;
  font-size: 18px;
}
p {
  margin: 0 0 8px;
}
.small {
  font-size: 12px;
}
</style>
