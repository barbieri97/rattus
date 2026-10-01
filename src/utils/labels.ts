// Human-readable names for values coming from the simulation.
import type { Behavior } from "../bindings/Behavior";
import type { CsSpec } from "../bindings/CsSpec";
import type { OperantDesign } from "../bindings/OperantDesign";
import type { Schedule } from "../bindings/Schedule";
import type { UsLevel } from "../bindings/UsLevel";

export const BEHAVIOR_LABELS: Record<Behavior, string> = {
  walk: "Walking",
  sniff: "Sniffing",
  lookAround: "Looking around",
  rear: "Rearing",
  rearWall: "Rearing against the wall",
  groom: "Grooming",
  faceWipe: "Wiping face",
  scratch: "Scratching",
  drink: "Drinking",
  eat: "Eating",
  barPress: "Pressing the bar",
  beg: "Begging",
  roll: "Rolling over",
  freeze: "Freezing",
  startle: "Startled by shock",
  orient: "Orienting",
};

export function scheduleLabel(s: Schedule): string {
  switch (s.kind) {
    case "continuous":
      return "CRF";
    case "fixedRatio":
      return `FR-${s.n}`;
    case "variableRatio":
      return `VR-${s.n}`;
    case "fixedInterval":
      return `FI-${s.seconds} s`;
    case "variableInterval":
      return `VI-${s.seconds} s`;
  }
}

export function designLabel(d: OperantDesign): string {
  switch (d.reinforcer) {
    case "food":
      return `${scheduleLabel(d.schedule)}, food`;
    case "soundOnly":
      return `${scheduleLabel(d.schedule)}, sound only`;
    case "none":
      return "Extinction";
  }
}

export const US_LABELS: Record<UsLevel, string> = {
  none: "No shock",
  low: "Low shock",
  medium: "Medium shock",
  high: "High shock",
};

export function csLabel(cs: CsSpec): string {
  const parts: string[] = [];
  if (cs.light) parts.push("Light");
  if (cs.toneDb !== null) parts.push(`Tone ${Math.round(cs.toneDb)} dB`);
  if (cs.bell) parts.push("Bell");
  return parts.length ? parts.join(" + ") : "None";
}
