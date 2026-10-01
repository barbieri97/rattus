<script setup lang="ts">
// The application's menu bar: File, Edit, Experiment, Windows and Help.
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import * as actions from "../actions";
import { PANEL_ORDER, PANEL_TITLES, useLayoutStore } from "../stores/layout";
import { useSimStore } from "../stores/sim";
import { useUiStore } from "../stores/ui";

interface Item {
  label: string;
  shortcut?: string;
  action: () => unknown;
  disabled?: boolean;
  checked?: boolean;
}
type Entry = Item | "separator";
interface Menu {
  label: string;
  items: Entry[];
}

const sim = useSimStore();
const ui = useUiStore();
const layout = useLayoutStore();
const openMenu = ref<number | null>(null);
const mod = navigator.platform.toLowerCase().includes("mac") ? "⌘" : "Ctrl+";

const menus = computed<Menu[]>(() => {
  const running = sim.classical !== null;
  return [
    {
      label: "File",
      items: [
        { label: "New Naive Rat", shortcut: `${mod}N`, action: () => actions.newRat("naive") },
        { label: "New Bar-Trained Rat (VR-25)", action: () => actions.newRat("barTrained") },
        "separator",
        { label: "Open…", shortcut: `${mod}O`, action: actions.openFile },
        { label: "Save", shortcut: `${mod}S`, action: () => actions.saveFile(false) },
        { label: "Save As…", shortcut: `${mod}Shift+S`, action: () => actions.saveFile(true) },
        "separator",
        { label: "Export Events (CSV)…", action: () => actions.exportData("events") },
        { label: "Export Classical Trials (CSV)…", action: () => actions.exportData("trials") },
        { label: "Export Mind Windows (CSV)…", action: () => actions.exportData("associations") },
      ],
    },
    {
      label: "Edit",
      items: [{ label: "Copy Window Data", shortcut: `${mod}C`, action: actions.copyActiveWindow }],
    },
    {
      label: "Experiment",
      items: [
        { label: "Give Pellet", shortcut: "Space", action: actions.givePellet },
        { label: "Mark Record", shortcut: `${mod}M`, action: actions.markRecord },
        "separator",
        { label: "Design Operant Conditioning Experiment…", action: () => ui.openDialog("operantDesign") },
        { label: "Design Classical Conditioning Experiment…", action: () => ui.openDialog("classicalDesign") },
        running
          ? { label: "Stop Classical Experiment", action: actions.stopClassical }
          : { label: "Run Classical Experiment", action: actions.startClassical },
        "separator",
        {
          label: "Isolate Rat (Accelerate Time)",
          shortcut: `${mod}I`,
          checked: sim.host.isolated,
          action: actions.toggleIsolation,
        },
        { label: "Remove Rat for Time Off…", action: () => ui.openDialog("timeOff") },
        "separator",
        { label: sim.host.paused ? "Resume" : "Pause", shortcut: "P", action: actions.togglePause },
      ],
    },
    {
      label: "Windows",
      items: [
        ...PANEL_ORDER.filter((id) => id !== "chamber").map((id) => ({
          label: PANEL_TITLES[id],
          checked: layout.panels[id].open,
          action: () => layout.toggle(id),
        })),
        "separator",
        { label: "Reset Window Layout", action: layout.reset },
      ],
    },
    {
      label: "Help",
      items: [
        { label: "Quick Guide", shortcut: "F1", action: () => ui.openDialog("guide") },
        { label: "Preferences…", shortcut: `${mod},`, action: () => ui.openDialog("preferences") },
        "separator",
        { label: "About Rattus", action: () => ui.openDialog("about") },
      ],
    },
  ];
});

function run(item: Item) {
  openMenu.value = null;
  if (!item.disabled) void item.action();
}

function toggle(i: number) {
  openMenu.value = openMenu.value === i ? null : i;
}

function hover(i: number) {
  if (openMenu.value !== null) openMenu.value = i;
}

function onDocumentDown(e: PointerEvent) {
  if (!(e.target as HTMLElement).closest(".menubar")) openMenu.value = null;
}

onMounted(() => document.addEventListener("pointerdown", onDocumentDown));
onBeforeUnmount(() => document.removeEventListener("pointerdown", onDocumentDown));
</script>

<template>
  <nav class="menubar" @keydown.esc="openMenu = null">
    <div v-for="(menu, i) in menus" :key="menu.label" class="menu">
      <button class="menu-title" :class="{ open: openMenu === i }" @click="toggle(i)" @mouseenter="hover(i)">
        {{ menu.label }}
      </button>
      <div v-if="openMenu === i" class="dropdown" role="menu">
        <template v-for="(entry, j) in menu.items" :key="j">
          <div v-if="entry === 'separator'" class="separator" />
          <button v-else class="item" role="menuitem" :disabled="entry.disabled" @click="run(entry)">
            <span class="check">{{ entry.checked ? "✓" : "" }}</span>
            <span class="label">{{ entry.label }}</span>
            <span class="shortcut">{{ entry.shortcut }}</span>
          </button>
        </template>
      </div>
    </div>
  </nav>
</template>

<style scoped>
.menubar {
  display: flex;
  background: var(--menu);
  border-bottom: 1px solid var(--panel-border);
  padding: 0 4px;
  height: 28px;
  align-items: stretch;
  position: relative;
  z-index: 1000;
}
.menu {
  position: relative;
}
.menu-title {
  border: none;
  background: transparent;
  border-radius: 0;
  height: 100%;
  padding: 0 10px;
}
.menu-title:hover,
.menu-title.open {
  background: var(--menu-hover);
}
.dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  min-width: 260px;
  background: #ffffff;
  border: 1px solid var(--panel-border);
  box-shadow: var(--shadow);
  padding: 4px 0;
}
.item {
  display: flex;
  width: 100%;
  border: none;
  border-radius: 0;
  background: transparent;
  text-align: left;
  padding: 5px 12px 5px 6px;
  gap: 4px;
  white-space: nowrap;
}
.item:hover:not(:disabled) {
  background: var(--menu-hover);
}
.check {
  width: 16px;
  text-align: center;
}
.label {
  flex: 1;
}
.shortcut {
  color: var(--muted);
  margin-left: 24px;
}
.separator {
  height: 1px;
  background: #dde3ea;
  margin: 4px 0;
}
</style>
