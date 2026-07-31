import { describe, expect, it } from "vitest";
import { rolePhaseLabel } from "./role-phase";

describe("rolePhaseLabel", () => {
  it("maps wire and short phase strings to IP/OOP", () => {
    expect(rolePhaseLabel("in_possession")).toBe("IP");
    expect(rolePhaseLabel("out_of_possession")).toBe("OOP");
    expect(rolePhaseLabel("ip")).toBe("IP");
    expect(rolePhaseLabel("oop")).toBe("OOP");
  });

  it("uppercases unknown phases without inventing a short label", () => {
    expect(rolePhaseLabel("set_piece")).toBe("SET_PIECE");
  });
});
