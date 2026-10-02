// State of the simulation as seen by the interface, fed by frames from the Rust runner.
import { defineStore } from "pinia";
import { computed, ref, shallowRef } from "vue";
import * as backend from "../api/backend";
import type { AssocSample } from "../bindings/AssocSample";
import type { BehaviorStart } from "../bindings/BehaviorStart";
import type { ClassicalDesign } from "../bindings/ClassicalDesign";
import type { Command } from "../bindings/Command";
import type { Delta } from "../bindings/Delta";
import type { FullState } from "../bindings/FullState";
import type { HostControl } from "../bindings/HostControl";
import type { HostStatus } from "../bindings/HostStatus";
import type { SimMessage } from "../bindings/SimMessage";
import type { Snapshot } from "../bindings/Snapshot";
import type { TrialRecord } from "../bindings/TrialRecord";
import { applyEvents, emptySession, type Session } from "./session";

const LOG_LENGTH = 30;

/** Arrival times of the two latest frames, for interpolating between them. */
export interface FrameTiming {
  previousAt: number;
  latestAt: number;
}

export const useSimStore = defineStore("sim", () => {
  const snapshot = shallowRef<Snapshot | null>(null);
  const previous = shallowRef<Snapshot | null>(null);
  const timing: FrameTiming = { previousAt: 0, latestAt: 0 };
  const host = ref<HostStatus>({ paused: false, speed: 1, isolated: false, isolatedSpeed: 600 });
  const session = shallowRef<Session>(emptySession());
  const trials = shallowRef<TrialRecord[]>([]);
  const samples = shallowRef<AssocSample[]>([]);
  const behaviorLog = shallowRef<BehaviorStart[]>([]);
  const classicalDesign = shallowRef<ClassicalDesign | null>(null);
  /** Incremented whenever recorded data changes, for charts to redraw. */
  const dataVersion = ref(0);
  /** Incremented when the whole state is replaced (new rat, file opened). */
  const resetVersion = ref(0);
  const connected = ref(false);
  const lastError = ref<string | null>(null);
  const filePath = ref<string | null>(null);
  /** Changes since the last save or reset. */
  const changes = ref(0);

  const dirty = computed(() => changes.value > 0);
  const running = computed(() => !host.value.paused);
  const classical = computed(() => snapshot.value?.classical ?? null);

  function receiveSnapshot(next: Snapshot) {
    const now = performance.now();
    previous.value = snapshot.value;
    timing.previousAt = timing.latestAt || now;
    timing.latestAt = now;
    snapshot.value = next;
  }

  function applyDelta(delta: Delta) {
    let changed = applyEvents(session.value, delta.events);
    if (delta.events.length > 0) changes.value += delta.events.length;
    if (delta.trials.length > 0) {
      trials.value = trials.value.concat(delta.trials);
      changed = true;
    }
    if (delta.samples.length > 0) {
      samples.value = samples.value.concat(delta.samples);
      changed = true;
    }
    if (delta.behaviors.length > 0) {
      behaviorLog.value = behaviorLog.value.concat(delta.behaviors).slice(-LOG_LENGTH);
    }
    if (changed) dataVersion.value++;
  }

  function applyReset(full: FullState) {
    const fresh = emptySession();
    applyEvents(fresh, full.events);
    session.value = fresh;
    trials.value = full.trials;
    samples.value = full.samples;
    behaviorLog.value = [];
    classicalDesign.value = full.classicalDesign;
    host.value = full.host;
    previous.value = null;
    timing.latestAt = 0;
    receiveSnapshot(full.snapshot);
    dataVersion.value++;
    resetVersion.value++;
  }

  function handleMessage(message: SimMessage) {
    if (message.type === "frame") {
      host.value = message.data.host;
      receiveSnapshot(message.data.snapshot);
      applyDelta(message.data.delta);
    } else {
      applyReset(message.data);
    }
  }

  async function connect() {
    if (!backend.inTauri()) {
      lastError.value = "Rattus runs as a desktop app. Start it with `npm run tauri dev`.";
      return;
    }
    await backend.subscribe(handleMessage);
    connected.value = true;
  }

  async function command(c: Command): Promise<boolean> {
    try {
      await backend.simCommand(c);
      changes.value++;
      if (c.type === "setClassicalDesign") classicalDesign.value = c.design;
      return true;
    } catch (e) {
      lastError.value = backend.errorMessage(e);
      return false;
    }
  }

  async function control(c: HostControl) {
    try {
      host.value = await backend.hostControl(c);
    } catch (e) {
      lastError.value = backend.errorMessage(e);
    }
  }

  function markSaved(path: string | null) {
    filePath.value = path;
    changes.value = 0;
  }

  return {
    snapshot,
    previous,
    timing,
    host,
    session,
    trials,
    samples,
    behaviorLog,
    classicalDesign,
    dataVersion,
    resetVersion,
    connected,
    lastError,
    filePath,
    dirty,
    running,
    classical,
    handleMessage,
    connect,
    command,
    control,
    markSaved,
  };
});
