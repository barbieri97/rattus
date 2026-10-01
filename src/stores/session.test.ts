import { describe, expect, it } from "vitest";
import { applyEvents, countBetween, emptySession, lowerBound } from "./session";

describe("applyEvents", () => {
  it("sorts events into the session", () => {
    const s = emptySession();
    applyEvents(s, [
      { type: "press", t: 1, reinforced: false },
      { type: "press", t: 2, reinforced: true },
      { type: "reinforcer", t: 2, kind: "food" },
      { type: "csOn", t: 5, cs: { light: true, toneDb: null, bell: false } },
      { type: "shock", t: 34, level: "medium" },
      { type: "csOff", t: 35 },
      { type: "mark", t: 40, label: "start" },
    ]);
    expect(s.presses).toEqual([1, 2]);
    expect(s.reinforcers).toHaveLength(1);
    expect(s.cs).toEqual([{ on: 5, off: 35, cs: { light: true, toneDb: null, bell: false } }]);
    expect(s.shocks).toHaveLength(1);
    expect(s.notes[0]).toEqual({ t: 40, label: "start", kind: "mark" });
  });
});

describe("lowerBound / countBetween", () => {
  it("finds positions in sorted arrays", () => {
    const a = [1, 2, 2, 5, 9];
    expect(lowerBound(a, 2)).toBe(1);
    expect(lowerBound(a, 6)).toBe(4);
    expect(lowerBound(a, 100)).toBe(5);
    expect(countBetween(a, 2, 9)).toBe(3);
  });
});
