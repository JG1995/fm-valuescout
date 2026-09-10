import { describe, expect, it } from "vitest";
import { graphicsResultQueryOptions } from "./graphics-query-options";

describe("graphics result queries", () => {
  it("keys results by generation and never uses prior data as a placeholder", () => {
    const options = graphicsResultQueryOptions(3, "personPortrait", 42);

    expect(options.queryKey).toEqual([
      "graphics",
      "result",
      { generation: 3, kind: "personPortrait", uid: 42 },
    ]);
    expect(options.placeholderData).toBeUndefined();
  });
});
