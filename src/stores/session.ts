// Session data derived from the recorded events, kept in shapes that are quick to draw.
import type { CsSpec } from "../bindings/CsSpec";
import type { RecordEvent } from "../bindings/RecordEvent";
import type { ReinforcerKind } from "../bindings/ReinforcerKind";
import type { UsLevel } from "../bindings/UsLevel";

export interface CsInterval {
  on: number;
  /** Null while the CS is still on. */
  off: number | null;
  cs: CsSpec;
}

export type NoteKind = "mark" | "design" | "timeOff" | "classical";

export interface Note {
  t: number;
  label: string;
  kind: NoteKind;
}

export interface Session {
  /** Times of every bar press, in order. */
  presses: number[];
  reinforcers: { t: number; kind: ReinforcerKind }[];
  notes: Note[];
  cs: CsInterval[];
  shocks: { t: number; level: UsLevel }[];
}

export function emptySession(): Session {
  return { presses: [], reinforcers: [], notes: [], cs: [], shocks: [] };
}

/** Appends events to the session in place. Returns true if anything was added. */
export function applyEvents(session: Session, events: RecordEvent[]): boolean {
  for (const e of events) {
    switch (e.type) {
      case "press":
        session.presses.push(e.t);
        break;
      case "reinforcer":
        session.reinforcers.push({ t: e.t, kind: e.kind });
        break;
      case "mark":
        session.notes.push({ t: e.t, label: e.label, kind: "mark" });
        break;
      case "designChange":
        session.notes.push({ t: e.t, label: e.label, kind: "design" });
        break;
      case "timeOff":
        session.notes.push({ t: e.t, label: `${e.hours} h off`, kind: "timeOff" });
        break;
      case "classicalStart":
        session.notes.push({ t: e.t, label: "Classical experiment", kind: "classical" });
        break;
      case "classicalEnd":
        session.notes.push({
          t: e.t,
          label: e.completed ? "Experiment complete" : "Experiment stopped",
          kind: "classical",
        });
        break;
      case "csOn":
        session.cs.push({ on: e.t, off: null, cs: e.cs });
        break;
      case "csOff": {
        const open = session.cs[session.cs.length - 1];
        if (open && open.off === null) open.off = e.t;
        break;
      }
      case "shock":
        session.shocks.push({ t: e.t, level: e.level });
        break;
    }
  }
  return events.length > 0;
}

/** Index of the first element of a sorted array that is >= x. */
export function lowerBound(sorted: number[], x: number): number {
  let lo = 0;
  let hi = sorted.length;
  while (lo < hi) {
    const mid = (lo + hi) >> 1;
    if (sorted[mid] < x) lo = mid + 1;
    else hi = mid;
  }
  return lo;
}

/** Number of presses in [t0, t1). */
export function countBetween(sorted: number[], t0: number, t1: number): number {
  return lowerBound(sorted, t1) - lowerBound(sorted, t0);
}
