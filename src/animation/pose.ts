// Poses of the rat drawing for each behaviour.
//
// The rat is drawn in profile facing right. Angles are in degrees, clockwise positive (SVG):
// a negative body pitch raises the front of the body (rearing); limb angles are relative to
// the body. Each behaviour is a pure function of the time since its bout started, so the
// animation follows program time and speeds up with the simulation.
import type { Behavior } from "../bindings/Behavior";

export interface Pose {
  /** Body pitch around the hip; negative raises the front. */
  pitch: number;
  /** Vertical offset of the whole rat in local units; negative is up. */
  lift: number;
  /** Horizontal scale of the body (crouching shortens it). */
  stretch: number;
  /** Forward shift of the whole rat, e.g. to reach the wall when rearing against it. */
  shift: number;
  /** Head angle relative to the body; positive tips the snout down. */
  head: number;
  headX: number;
  headY: number;
  /** Near and far forelimbs and hind limbs, relative to the body. */
  fore: number;
  foreFar: number;
  hind: number;
  hindFar: number;
  /** Tail curl, -1 (down) to 1 (curled up). */
  tail: number;
  /** Tail wave amplitude. */
  tailWave: number;
  /** Ear angle; positive lays the ear back. */
  ear: number;
  /** Rotation of the whole rat (rolling over). */
  roll: number;
  /** Jaw movement, 0 to 1. */
  chew: number;
  /** 0 = head in profile, 1 = face turned toward the observer. */
  front: number;
  /** Trembling amplitude. */
  jitter: number;
  /** Whisker twitch, -1 to 1. */
  whisker: number;
}

export const NEUTRAL: Pose = {
  pitch: 0,
  lift: 0,
  stretch: 1,
  shift: 0,
  head: 0,
  headX: 0,
  headY: 0,
  fore: 0,
  foreFar: 0,
  hind: 0,
  hindFar: 0,
  tail: 0.15,
  tailWave: 0.3,
  ear: 0,
  roll: 0,
  chew: 0,
  front: 0,
  jitter: 0,
  whisker: 0,
};

const TAU = Math.PI * 2;

function clamp01(x: number): number {
  return Math.min(1, Math.max(0, x));
}

export function smooth(x: number): number {
  const c = clamp01(x);
  return c * c * (3 - 2 * c);
}

/** Rises smoothly during the first `rise` fraction of a bout and falls during the last `fall`. */
export function envelope(progress: number, rise = 0.2, fall = 0.2): number {
  return Math.min(smooth(progress / rise), smooth((1 - progress) / fall));
}

function wave(hz: number, t: number, phase = 0): number {
  return Math.sin(TAU * hz * t + phase);
}

/**
 * Pose for `behavior`, `t` seconds into a bout that is `progress` (0..1) complete.
 * `stride` is the walking cycle phase in radians, advanced by the distance walked.
 */
export function poseFor(behavior: Behavior, t: number, progress: number, stride: number): Pose {
  const p = clamp01(progress);
  const pose: Pose = { ...NEUTRAL, tailWave: 0.3 + 0.2 * wave(0.4, t) };
  switch (behavior) {
    case "walk": {
      const w = Math.sin(stride);
      pose.lift = -1.5 * Math.abs(Math.sin(stride));
      pose.fore = 30 * w;
      pose.foreFar = -30 * w;
      pose.hind = -28 * w;
      pose.hindFar = 28 * w;
      pose.head = 3 * Math.sin(stride * 2);
      pose.tailWave = 0.8;
      break;
    }
    case "sniff": {
      const e = envelope(p, 0.15, 0.15);
      pose.head = (14 + 5 * wave(5, t)) * e;
      pose.headX = 2 * wave(5, t) * e;
      pose.whisker = wave(9, t);
      pose.fore = 6 * e;
      break;
    }
    case "lookAround": {
      pose.head = -8 + 14 * wave(0.5, t);
      pose.ear = 10 * wave(1.3, t);
      pose.headY = -1;
      break;
    }
    case "rear": {
      const e = envelope(p, 0.25, 0.2);
      pose.pitch = -58 * e;
      pose.hind = 58 * e;
      pose.hindFar = 58 * e;
      pose.fore = -25 * e;
      pose.foreFar = -15 * e;
      pose.head = 42 * e + 5 * wave(1, t) * e;
      pose.tail = 0.1;
      break;
    }
    case "rearWall": {
      const e = envelope(p, 0.25, 0.2);
      pose.pitch = -64 * e;
      pose.shift = 40 * e;
      pose.hind = 64 * e;
      pose.hindFar = 64 * e;
      pose.fore = -60 * e;
      pose.foreFar = -52 * e;
      pose.head = 48 * e + 4 * wave(1.5, t) * e;
      pose.whisker = wave(7, t);
      break;
    }
    case "barPress": {
      const e = envelope(p, 0.3, 0.25);
      // The paws push down in the middle of the bout, when the lever moves.
      const push = Math.max(0, 1 - Math.abs(p - 0.5) / 0.15);
      pose.pitch = -50 * e + 6 * push;
      pose.shift = 40 * e;
      pose.hind = 50 * e - 6 * push;
      pose.hindFar = pose.hind;
      pose.fore = -72 * e + 34 * push;
      pose.foreFar = -64 * e + 34 * push;
      pose.head = 34 * e;
      break;
    }
    case "groom": {
      const e = envelope(p, 0.15, 0.15);
      pose.pitch = -22 * e;
      pose.hind = 22 * e;
      pose.hindFar = 22 * e;
      pose.head = 58 * e + 6 * wave(2, t) * e;
      pose.fore = (-40 + 20 * wave(3, t)) * e;
      pose.foreFar = (-40 + 20 * wave(3, t, Math.PI)) * e;
      pose.tail = 0.4;
      break;
    }
    case "faceWipe": {
      const e = envelope(p, 0.2, 0.2);
      pose.pitch = -40 * e;
      pose.hind = 40 * e;
      pose.hindFar = 40 * e;
      pose.head = 14 * e;
      pose.fore = (-112 + 26 * wave(2.5, t)) * e;
      pose.foreFar = (-112 + 26 * wave(2.5, t, 0.7)) * e;
      pose.ear = 8 * e;
      break;
    }
    case "scratch": {
      const e = envelope(p, 0.15, 0.15);
      pose.pitch = -18 * e;
      pose.hind = (18 - 75 + 22 * wave(9, t)) * e;
      pose.hindFar = 18 * e;
      pose.head = 26 * e;
      pose.ear = 12 * e;
      break;
    }
    case "drink": {
      const e = envelope(p, 0.15, 0.15);
      pose.pitch = -26 * e;
      pose.shift = 20 * e;
      pose.hind = 26 * e;
      pose.hindFar = 26 * e;
      pose.head = -16 * e;
      pose.headX = 1.5 * wave(6, t) * e;
      pose.chew = (0.5 + 0.5 * wave(6, t)) * e;
      pose.fore = -10 * e;
      break;
    }
    case "eat": {
      const e = envelope(p, 0.15, 0.1);
      pose.pitch = 6 * e;
      pose.shift = 6 * e;
      pose.hind = -6 * e;
      pose.head = 26 * e;
      pose.chew = (0.5 + 0.5 * wave(4, t)) * e;
      pose.fore = -25 * e;
      pose.foreFar = -20 * e;
      break;
    }
    case "beg": {
      const e = envelope(p, 0.25, 0.2);
      pose.pitch = -72 * e;
      pose.hind = 72 * e;
      pose.hindFar = 72 * e;
      pose.fore = (-95 + 6 * wave(2, t)) * e;
      pose.foreFar = (-95 + 6 * wave(2, t)) * e;
      pose.head = (58 + 7 * wave(2, t)) * e;
      pose.headY = 2 * wave(2, t) * e;
      pose.front = e;
      break;
    }
    case "roll": {
      const r = smooth(p);
      pose.roll = 360 * r;
      pose.lift = -16 * Math.sin(Math.PI * p);
      pose.fore = -40 * Math.sin(Math.PI * p);
      pose.foreFar = pose.fore;
      pose.hind = -30 * Math.sin(Math.PI * p);
      pose.hindFar = pose.hind;
      pose.tail = 0.6;
      break;
    }
    case "freeze": {
      pose.lift = 4;
      pose.stretch = 0.92;
      pose.head = 10;
      pose.ear = 25;
      pose.tail = -0.4;
      pose.tailWave = 0;
      pose.jitter = 0.7;
      pose.fore = 6;
      pose.hind = 10;
      break;
    }
    case "startle": {
      const hop = p < 0.35 ? Math.sin((Math.PI * p) / 0.35) : 0;
      pose.lift = -42 * hop;
      pose.pitch = -16 * hop;
      pose.fore = -45 * hop;
      pose.foreFar = -35 * hop;
      pose.hind = 45 * hop;
      pose.hindFar = 35 * hop;
      pose.jitter = 2.2 * (1 - p);
      pose.ear = 30;
      pose.tail = 0.9 * hop;
      pose.tailWave = 1;
      break;
    }
    case "orient": {
      const e = envelope(p, 0.2, 0.3);
      pose.pitch = -12 * e;
      pose.hind = 12 * e;
      pose.hindFar = 12 * e;
      pose.head = -24 * e;
      pose.ear = -20 * e;
      pose.whisker = wave(8, t) * e;
      break;
    }
  }
  return pose;
}

function normalizeRoll(r: number): number {
  const m = ((r % 360) + 360) % 360;
  return m > 180 ? m - 360 : m;
}

/** Linear blend from pose `a` (k = 0) to pose `b` (k = 1). */
export function blend(a: Pose, b: Pose, k: number): Pose {
  const w = clamp01(k);
  const out = { ...b };
  for (const key of Object.keys(NEUTRAL) as (keyof Pose)[]) {
    if (key === "roll") {
      const from = normalizeRoll(a.roll);
      const to = w >= 1 ? b.roll : normalizeRoll(b.roll);
      out.roll = from + (to - from) * w;
    } else {
      out[key] = a[key] + (b[key] - a[key]) * w;
    }
  }
  return out;
}
