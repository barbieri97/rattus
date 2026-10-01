import { describe, expect, it } from "vitest";
import { toTsv } from "./tsv";

describe("toTsv", () => {
  it("joins cells with tabs and cleans them", () => {
    expect(
      toTsv(
        ["a", "b"],
        [
          [1.234567, "x\ty"],
          [null, true],
        ],
      ),
    ).toBe("a\tb\n1.2346\tx y\n\ttrue\n");
  });
});
