<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { onBeforeUnmount, onMounted, ref, watch } from "vue";
import * as actions from "./actions";
import { inTauri } from "./api/backend";
import { unlockAudio } from "./audio/sounds";
import { useChamberSounds } from "./audio/useChamberSounds";
import ChamberView from "./components/ChamberView.vue";
import FloatingPanel from "./components/FloatingPanel.vue";
import MenuBar from "./components/MenuBar.vue";
import Toolbar from "./components/Toolbar.vue";
import AboutDialog from "./components/dialogs/AboutDialog.vue";
import ClassicalDesignDialog from "./components/dialogs/ClassicalDesignDialog.vue";
import OperantDesignDialog from "./components/dialogs/OperantDesignDialog.vue";
import PreferencesDialog from "./components/dialogs/PreferencesDialog.vue";
import QuickGuideDialog from "./components/dialogs/QuickGuideDialog.vue";
import TimeOffDialog from "./components/dialogs/TimeOffDialog.vue";
import BehaviorLogPanel from "./components/panels/BehaviorLogPanel.vue";
import CsStrengthPanel from "./components/panels/CsStrengthPanel.vue";
import CumulativeRecordPanel from "./components/panels/CumulativeRecordPanel.vue";
import OperantAssociationsPanel from "./components/panels/OperantAssociationsPanel.vue";
import RatioPanel from "./components/panels/RatioPanel.vue";
import SensitivityFearPanel from "./components/panels/SensitivityFearPanel.vue";
import { useLayoutStore } from "./stores/layout";
import { useSimStore } from "./stores/sim";
import { useUiStore } from "./stores/ui";

const sim = useSimStore();
const ui = useUiStore();
const layout = useLayoutStore();
const workspace = ref<HTMLElement | null>(null);
let observer: ResizeObserver | null = null;

useChamberSounds();

// Errors from commands are shown briefly.
watch(
  () => sim.lastError,
  (message) => {
    if (message && sim.connected) {
      ui.notify(message);
      sim.lastError = null;
    }
  },
);

function typing(target: EventTarget | null): boolean {
  const el = target as HTMLElement | null;
  return (
    !!el && (el.tagName === "INPUT" || el.tagName === "SELECT" || el.tagName === "TEXTAREA" || el.isContentEditable)
  );
}

function onKey(e: KeyboardEvent) {
  unlockAudio();
  if (ui.dialog !== null) return;
  const mod = e.ctrlKey || e.metaKey;
  const key = e.key.toLowerCase();
  if (mod) {
    const handled: Record<string, () => unknown> = {
      s: () => actions.saveFile(e.shiftKey),
      o: actions.openFile,
      n: () => actions.newRat("naive"),
      m: actions.markRecord,
      i: actions.toggleIsolation,
      ",": () => ui.openDialog("preferences"),
    };
    if (key === "c" && !typing(e.target) && !window.getSelection()?.toString()) {
      e.preventDefault();
      void actions.copyActiveWindow();
      return;
    }
    if (handled[key]) {
      e.preventDefault();
      void handled[key]();
    }
    return;
  }
  if (typing(e.target)) return;
  if (e.key === " ") {
    e.preventDefault();
    if (!e.repeat) void actions.givePellet();
  } else if (key === "p") {
    void actions.togglePause();
  } else if (e.key === "F1") {
    e.preventDefault();
    void ui.openDialog("guide");
  }
}

onMounted(async () => {
  window.addEventListener("keydown", onKey);
  window.addEventListener("pointerdown", unlockAudio, { once: true });
  if (workspace.value) {
    const r = workspace.value.getBoundingClientRect();
    layout.init(Math.floor(r.width), Math.floor(r.height));
    observer = new ResizeObserver(([entry]) => {
      layout.resize(Math.floor(entry.contentRect.width), Math.floor(entry.contentRect.height));
    });
    observer.observe(workspace.value);
  }
  await sim.connect();
  if (inTauri()) {
    await getCurrentWindow().onCloseRequested(async (event) => {
      if (!(await actions.confirmDiscard())) event.preventDefault();
    });
  }
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKey);
  observer?.disconnect();
});
</script>

<template>
  <div class="app">
    <MenuBar />
    <Toolbar />
    <main ref="workspace" class="workspace">
      <div v-if="!sim.connected && sim.lastError" class="offline">{{ sim.lastError }}</div>
      <FloatingPanel id="chamber" :closable="false">
        <ChamberView />
      </FloatingPanel>
      <FloatingPanel id="cumulative">
        <CumulativeRecordPanel />
      </FloatingPanel>
      <FloatingPanel id="operant">
        <OperantAssociationsPanel />
      </FloatingPanel>
      <FloatingPanel id="behavior">
        <BehaviorLogPanel />
      </FloatingPanel>
      <FloatingPanel id="csStrength">
        <CsStrengthPanel />
      </FloatingPanel>
      <FloatingPanel id="sensitivityFear">
        <SensitivityFearPanel />
      </FloatingPanel>
      <FloatingPanel id="suppression">
        <RatioPanel kind="suppression" />
      </FloatingPanel>
      <FloatingPanel id="movement">
        <RatioPanel kind="movement" />
      </FloatingPanel>
    </main>

    <PreferencesDialog v-if="ui.dialog === 'preferences'" />
    <OperantDesignDialog v-else-if="ui.dialog === 'operantDesign'" />
    <ClassicalDesignDialog v-else-if="ui.dialog === 'classicalDesign'" />
    <TimeOffDialog v-else-if="ui.dialog === 'timeOff'" />
    <AboutDialog v-else-if="ui.dialog === 'about'" />
    <QuickGuideDialog v-else-if="ui.dialog === 'guide'" />

    <transition name="fade">
      <div v-if="ui.toast" :key="ui.toast.id" class="toast">{{ ui.toast.text }}</div>
    </transition>
  </div>
</template>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  height: 100%;
}
.workspace {
  flex: 1;
  position: relative;
  overflow: hidden;
  background: var(--bg);
}
.offline {
  position: absolute;
  top: 40%;
  left: 50%;
  transform: translate(-50%, -50%);
  z-index: 500;
  background: #fff4dc;
  border: 1px solid #e8c779;
  padding: 12px 18px;
  border-radius: 8px;
}
.toast {
  position: fixed;
  bottom: 18px;
  left: 50%;
  transform: translateX(-50%);
  background: #1f2a36;
  color: #f1f5fa;
  padding: 8px 16px;
  border-radius: 6px;
  box-shadow: var(--shadow);
  z-index: 3000;
  max-width: 70vw;
}
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.25s;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
