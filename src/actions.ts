// High-level user actions, shared by the menu bar, the toolbar and keyboard shortcuts.
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { ask, open, save } from "@tauri-apps/plugin-dialog";
import * as backend from "./api/backend";
import type { ExportDataset } from "./bindings/ExportDataset";
import type { RatTemplate } from "./bindings/RatTemplate";
import { usePrefsStore } from "./stores/prefs";
import { useLayoutStore, type PanelId, PANEL_TITLES } from "./stores/layout";
import { useSimStore } from "./stores/sim";
import { useUiStore } from "./stores/ui";

const FILE_FILTER = [{ name: "Rattus experiment", extensions: ["rattus"] }];
const CSV_FILTER = [{ name: "CSV spreadsheet", extensions: ["csv"] }];

function report(error: unknown) {
  useUiStore().notify(backend.errorMessage(error));
}

/** Asks before throwing away unsaved work. Returns true when it is fine to continue. */
export async function confirmDiscard(): Promise<boolean> {
  const sim = useSimStore();
  if (!sim.dirty) return true;
  return ask("The current experiment has unsaved changes. Discard them?", {
    title: "Unsaved changes",
    kind: "warning",
    okLabel: "Discard",
    cancelLabel: "Cancel",
  });
}

export async function newRat(template: RatTemplate) {
  if (!(await confirmDiscard())) return;
  const sim = useSimStore();
  const seed = Math.floor(Math.random() * 2 ** 31);
  if (await sim.command({ type: "newRat", template, seed })) {
    sim.markSaved(null);
    useUiStore().notify(
      template === "naive" ? "A new, naive rat is in the chamber." : "A bar-trained rat (VR-25) is in the chamber.",
    );
  }
}

export async function openFile() {
  if (!(await confirmDiscard())) return;
  try {
    const path = await open({ multiple: false, directory: false, filters: FILE_FILTER });
    if (typeof path !== "string") return;
    await backend.openExperiment(path, usePrefsStore().prefs.randomStartOnOpen);
    useSimStore().markSaved(path);
  } catch (e) {
    report(e);
  }
}

export async function saveFile(saveAs = false) {
  const sim = useSimStore();
  try {
    let path = saveAs ? null : sim.filePath;
    if (!path) {
      const chosen = await save({ filters: FILE_FILTER, defaultPath: "experiment.rattus" });
      if (!chosen) return;
      path = chosen.endsWith(".rattus") ? chosen : `${chosen}.rattus`;
    }
    await backend.saveExperiment(path);
    sim.markSaved(path);
    useUiStore().notify("Experiment saved.");
  } catch (e) {
    report(e);
  }
}

const DATASET_NAMES: Record<ExportDataset, string> = {
  events: "events",
  trials: "classical-trials",
  associations: "mind-windows",
};

export async function exportData(dataset: ExportDataset) {
  try {
    const chosen = await save({ filters: CSV_FILTER, defaultPath: `rattus-${DATASET_NAMES[dataset]}.csv` });
    if (!chosen) return;
    const path = chosen.toLowerCase().endsWith(".csv") ? chosen : `${chosen}.csv`;
    await backend.exportCsv(path, dataset);
    useUiStore().notify("Data exported.");
  } catch (e) {
    report(e);
  }
}

// Each data window registers how to turn what it shows into text for Edit → Copy.
const copyProviders = new Map<PanelId, () => string>();

export function registerCopy(id: PanelId, provider: () => string): () => void {
  copyProviders.set(id, provider);
  return () => {
    if (copyProviders.get(id) === provider) copyProviders.delete(id);
  };
}

export async function copyActiveWindow() {
  const layout = useLayoutStore();
  const ui = useUiStore();
  const id = layout.active;
  const provider = copyProviders.get(id);
  if (!provider || !layout.panels[id].open) {
    ui.notify("Select a data window (for example the Cumulative Record) to copy its data.");
    return;
  }
  try {
    await writeText(provider());
    ui.notify(`${PANEL_TITLES[id]} data copied. Paste it into a spreadsheet.`);
  } catch (e) {
    report(e);
  }
}

export function givePellet() {
  return useSimStore().command({ type: "givePellet" });
}

export function markRecord() {
  return useSimStore().command({ type: "markRecord", label: null });
}

export function togglePause() {
  const sim = useSimStore();
  return sim.control({ type: sim.host.paused ? "resume" : "pause" });
}

export async function toggleIsolation() {
  const sim = useSimStore();
  const prefs = usePrefsStore();
  if (!sim.host.isolated && sim.host.isolatedSpeed !== prefs.prefs.acceleratedSpeed) {
    await sim.control({ type: "setIsolatedSpeed", speed: prefs.prefs.acceleratedSpeed });
  }
  await sim.control({ type: "setIsolated", isolated: !sim.host.isolated });
}

export function setSpeed(speed: number) {
  return useSimStore().control({ type: "setSpeed", speed });
}

export async function startClassical() {
  const sim = useSimStore();
  if (await sim.command({ type: "startClassical" })) {
    const layout = useLayoutStore();
    for (const id of ["csStrength", "sensitivityFear", "suppression"] as PanelId[]) layout.open(id);
    useUiStore().notify("Classical conditioning experiment started. Isolate the rat to speed it up.");
  }
}

export function stopClassical() {
  return useSimStore().command({ type: "stopClassical" });
}
