// User preferences, kept in the webview's local storage.
import { defineStore } from "pinia";
import { ref, watch } from "vue";
import { loadJson, saveJson } from "../utils/storage";

export interface Preferences {
  /** Loudness of every sound from the chamber, 0 (quiet) to 1 (loud). */
  soundProofing: number;
  /** Draw graphs with a reduced, high-contrast set of colours. */
  fewerColours: boolean;
  /** Scale the chamber drawing with its window instead of drawing it at a fixed size. */
  scaleChamber: boolean;
  /** When opening a file, put the rat near the middle of the chamber at random. */
  randomStartOnOpen: boolean;
  /** Program seconds per real second while the rat is isolated. */
  acceleratedSpeed: number;
}

export const DEFAULT_PREFERENCES: Preferences = {
  soundProofing: 0.6,
  fewerColours: false,
  scaleChamber: true,
  randomStartOnOpen: true,
  acceleratedSpeed: 600,
};

const KEY = "rattus.preferences";

export function sanitizePreferences(raw: Partial<Preferences> | null): Preferences {
  const p = { ...DEFAULT_PREFERENCES };
  if (!raw) return p;
  if (typeof raw.soundProofing === "number" && Number.isFinite(raw.soundProofing)) {
    p.soundProofing = Math.min(1, Math.max(0, raw.soundProofing));
  }
  if (typeof raw.fewerColours === "boolean") p.fewerColours = raw.fewerColours;
  if (typeof raw.scaleChamber === "boolean") p.scaleChamber = raw.scaleChamber;
  if (typeof raw.randomStartOnOpen === "boolean") p.randomStartOnOpen = raw.randomStartOnOpen;
  if (typeof raw.acceleratedSpeed === "number" && [60, 300, 600, 1800, 3600].includes(raw.acceleratedSpeed)) {
    p.acceleratedSpeed = raw.acceleratedSpeed;
  }
  return p;
}

export const usePrefsStore = defineStore("prefs", () => {
  const prefs = ref<Preferences>(sanitizePreferences(loadJson<Preferences>(KEY)));
  watch(prefs, (value) => saveJson(KEY, value), { deep: true });

  function update(next: Preferences) {
    prefs.value = sanitizePreferences(next);
  }

  return { prefs, update };
});
