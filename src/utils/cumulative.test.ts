import { describe, expect, it } from "vitest";
import { niceTickMinutes, penHeightAt, penSegments } from "./cumulative";

describe("penSegments", () => {
  it("steps up at each press", () => {
    const segs = penSegments([1, 2, 3], 0, 10, 100);
    expect(segs).toHaveLength(1);
    const last = segs[0][segs[0].length - 1];
    expect(last).toEqual({ t: 10, y: 3 });
  });

  it("resets to the bottom when the paper is full", () => {
    const presses = Array.from({ length: 7 }, (_, i) => i + 1);
    const segs = penSegments(presses, 0, 10, 3);
    expect(segs).toHaveLength(3);
    expect(segs[1][0]).toEqual({ t: 3, y: 0 });
    expect(segs[2][segs[2].length - 1]).toEqual({ t: 10, y: 1 });
  });

  it("starts at the right height when scrolled", () => {
    const presses = [1, 2, 3, 4, 5];
    const segs = penSegments(presses, 3.5, 10, 100);
    expect(segs[0][0]).toEqual({ t: 3.5, y: 3 });
  });
});

describe("penHeightAt", () => {
  it("counts the press at that instant", () => {
    expect(penHeightAt([1, 2, 3], 2, 100)).toBe(2);
  });

  it("draws a sweep-completing press at the top", () => {
    expect(penHeightAt([1, 2, 3], 3, 3)).toBe(3);
  });
});

describe("niceTickMinutes", () => {
  it("chooses wider ticks when zoomed out", () => {
    expect(niceTickMinutes(60)).toBe(1);
    expect(niceTickMinutes(5)).toBe(15);
  });
});
