// Geometry of the cumulative record: a pen that steps up at every response, moves right with
// time and resets to the bottom when it reaches the top of the paper.
import { lowerBound } from "../stores/session";

export interface PenPoint {
  t: number;
  /** Responses since the last reset, 0..capacity. */
  y: number;
}

/**
 * Staircase segments of the pen between `t0` and `t1`. A new segment starts after each reset.
 */
export function penSegments(presses: number[], t0: number, t1: number, capacity: number): PenPoint[][] {
  const segments: PenPoint[][] = [];
  const start = lowerBound(presses, t0);
  let y = start % capacity;
  let current: PenPoint[] = [{ t: t0, y }];
  for (let i = start; i < presses.length && presses[i] < t1; i++) {
    const t = presses[i];
    current.push({ t, y });
    y += 1;
    current.push({ t, y });
    if (y >= capacity) {
      segments.push(current);
      y = 0;
      current = [{ t, y: 0 }];
    }
  }
  current.push({ t: t1, y });
  segments.push(current);
  return segments;
}

/** Height of the pen at time `t` (counting presses up to and including `t`). */
export function penHeightAt(presses: number[], t: number, capacity: number): number {
  const count = lowerBound(presses, t + 1e-6);
  const y = count % capacity;
  // A reinforced press that completed a sweep is drawn at the top, not the bottom.
  return y === 0 && count > 0 ? capacity : y;
}

/** A tick spacing (in minutes) that keeps labels at least `minPx` apart. */
export function niceTickMinutes(pxPerMinute: number, minPx = 60): number {
  for (const m of [0.5, 1, 2, 5, 10, 15, 30, 60, 120, 240]) {
    if (m * pxPerMinute >= minPx) return m;
  }
  return 480;
}
