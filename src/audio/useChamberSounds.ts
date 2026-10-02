// Plays the chamber's sounds as the snapshots change.
import { watch } from "vue";
import { usePrefsStore } from "../stores/prefs";
import { useSimStore } from "../stores/sim";
import * as sounds from "./sounds";

export function useChamberSounds(): void {
  const sim = useSimStore();
  const prefs = usePrefsStore();

  watch(
    () => prefs.prefs.soundProofing,
    (v) => sounds.setVolume(v),
    { immediate: true },
  );

  let lastDispense: number | null = null;
  let lastShock: number | null = null;

  watch(
    () => sim.snapshot,
    (snap) => {
      if (!snap) return;
      const silent = sim.host.isolated || sim.host.paused;
      const c = snap.chamber;
      if (lastDispense !== null && c.dispenseCount > lastDispense && !silent) sounds.playClick();
      if (lastShock !== null && c.shockCount > lastShock && !silent) sounds.playShock();
      lastDispense = c.dispenseCount;
      lastShock = c.shockCount;
      if (c.toneDb !== null && !silent) sounds.startTone(c.toneDb);
      else sounds.stopTone();
      if (c.bellOn && !silent) sounds.startBell();
      else sounds.stopBell();
    },
  );

  // A new rat or file: counters restart, so do not play anything for the jump.
  watch(
    () => sim.resetVersion,
    () => {
      lastDispense = null;
      lastShock = null;
      sounds.stopAll();
    },
  );
}
