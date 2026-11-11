import { describe, expect, it } from "vitest";
import { cardCapacity } from "./card-capacity";

const MIN = { width: 300, height: 200 };

describe("cardCapacity", () => {
  it.each([
    // Three cards need their two 1px rules as well.
    [
      { width: 901, height: 200 },
      { columns: 2, rows: 1 },
    ],
    [
      { width: 902, height: 200 },
      { columns: 3, rows: 1 },
    ],
    [
      { width: 902, height: 400 },
      { columns: 3, rows: 1 },
    ],
    [
      { width: 1806, height: 401 },
      { columns: 6, rows: 2 },
    ],
    [
      { width: 120, height: 80 },
      { columns: 1, rows: 1 },
    ],
  ])("fits %o as %o", (box, expected) => {
    expect(cardCapacity(box, MIN)).toEqual(expected);
  });
});
