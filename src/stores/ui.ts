// Which dialog is open, and short messages for the user.
import { defineStore } from "pinia";
import { ref } from "vue";
import { useSimStore } from "./sim";

export type DialogName = "preferences" | "operantDesign" | "classicalDesign" | "timeOff" | "about" | "guide";

export const useUiStore = defineStore("ui", () => {
  const dialog = ref<DialogName | null>(null);
  const toast = ref<{ text: string; id: number } | null>(null);
  let pausedForDialog = false;
  let toastTimer: ReturnType<typeof setTimeout> | undefined;

  /** Opens a dialog; the simulation waits while it is open. */
  async function openDialog(name: DialogName) {
    const sim = useSimStore();
    if (dialog.value === null && !sim.host.paused && name !== "about" && name !== "guide") {
      pausedForDialog = true;
      await sim.control({ type: "pause" });
    }
    dialog.value = name;
  }

  async function closeDialog() {
    dialog.value = null;
    if (pausedForDialog) {
      pausedForDialog = false;
      await useSimStore().control({ type: "resume" });
    }
  }

  function notify(text: string) {
    toast.value = { text, id: Date.now() };
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => (toast.value = null), 3500);
  }

  return { dialog, toast, openDialog, closeDialog, notify };
});
