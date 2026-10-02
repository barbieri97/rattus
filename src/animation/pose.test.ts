import { describe, expect, it } from "vitest";
import type { Behavior } from "../bindings/Behavior";
import { BEHAVIOR_LABELS } from "../utils/labels";
import { blend, envelope, NEUTRAL, poseFor, type Pose } from "./pose";

const BEHAVIORS = Object.keys(BEHAVIOR_LABELS) as Behavior[];

describe("poseFor", () => {
  it("gives finite values for every behaviour throughout a bout", () => {
    for (const b of BEHAVIORS) {
      for (let i = 0; i <= 20; i++) {
        const pose = poseFor(b, i * 0.17, i / 20, i * 0.9);
        for (const [key, value] of Object.entries(pose)) {
          expect(Number.isFinite(value), `${b}.${key}`).toBe(true);
        }
      }
    }
  });

  it("starts and ends postures near neutral so bouts join smoothly", () => {
    for (const b of ["rear", "rearWall", "beg", "groom", "faceWipe", "barPress"] as Behavior[]) {
      expect(Math.abs(poseFor(b, 0, 0, 0).pitch)).toBeLessThan(1);
      expect(Math.abs(poseFor(b, 3, 1, 0).pitch)).toBeLessThan(1);
      expect(poseFor(b, 1, 0.5, 0).pitch).toBeLessThan(-15);
    }
  });

  it("pushes the lever in the middle of a press", () => {
    const before = poseFor("barPress", 0.2, 0.35, 0).fore;
    const during = poseFor("barPress", 0.3, 0.5, 0).fore;
    expect(during).toBeGreaterThan(before);
  });

  it("turns the face to the observer when begging", () => {
    expect(poseFor("beg", 1, 0.5, 0).front).toBeGreaterThan(0.9);
  });
});

describe("envelope", () => {
  it("is zero at the ends and one in the middle", () => {
    expect(envelope(0)).toBe(0);
    expect(envelope(1)).toBe(0);
    expect(envelope(0.5)).toBe(1);
  });
});

describe("blend", () => {
  const a: Pose = { ...NEUTRAL, pitch: -60, roll: 360 };
  const b: Pose = { ...NEUTRAL, pitch: 0, roll: 0 };

  it("returns the endpoints", () => {
    expect(blend(a, b, 0).pitch).toBe(-60);
    expect(blend(a, b, 1).pitch).toBe(0);
    expect(blend(a, b, 0.5).pitch).toBe(-30);
  });

  it("does not spin backwards after a full roll", () => {
    expect(Math.abs(blend(a, b, 0.5).roll)).toBeLessThan(1);
  });
});
