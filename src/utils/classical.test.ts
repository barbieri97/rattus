import { describe, expect, it } from "vitest";
import { defaultDesign, estimatedMinutes, totalTrials, validateDesign } from "./classical";

describe("classical design helpers", () => {
  it("accepts the default design", () => {
    const d = defaultDesign();
    expect(validateDesign(d)).toBeNull();
    expect(totalTrials(d)).toBe(10);
    expect(estimatedMinutes(d)).toBe(55);
  });

  it("rejects trials without a stimulus and bad intensities", () => {
    const d = defaultDesign();
    d.stages[0].trialTypes[0].cs = { light: false, toneDb: null, bell: false };
    expect(validateDesign(d)).toMatch(/stimulus/);
    d.stages[0].trialTypes[0].cs = { light: false, toneDb: 120, bell: false };
    expect(validateDesign(d)).toMatch(/dB/);
  });
});
