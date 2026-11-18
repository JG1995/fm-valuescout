import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { TacticColumnToggles } from "./tactic-column-toggles";

describe("TacticColumnToggles", () => {
  it("reports layout-derived state and selects each score group", async () => {
    const user = userEvent.setup();
    const onToggleGroup = vi.fn();
    render(
      <TacticColumnToggles
        groups={[
          { group: "current", label: "Role Score", active: true },
          { group: "moneyball", label: "Moneyball Score", active: false },
        ]}
        disabled={false}
        onToggleGroup={onToggleGroup}
      />,
    );

    const roles = screen.getByRole("button", {
      name: "Add Tactic (Role Score)",
    });
    const moneyball = screen.getByRole("button", {
      name: "Add Tactic (Moneyball Score)",
    });
    expect(roles).toHaveAttribute("aria-pressed", "true");
    expect(moneyball).toHaveAttribute("aria-pressed", "false");

    await user.click(roles);
    await user.click(moneyball);
    expect(onToggleGroup.mock.calls).toEqual([["current"], ["moneyball"]]);
  });

  it("does not invoke callbacks while disabled", async () => {
    const user = userEvent.setup();
    const onToggleGroup = vi.fn();
    render(
      <TacticColumnToggles
        groups={[
          { group: "current", label: "Current", active: false },
          { group: "potential", label: "Potential", active: false },
        ]}
        disabled
        onToggleGroup={onToggleGroup}
      />,
    );

    await user.click(
      screen.getByRole("button", { name: "Add Tactic (Current)" }),
    );
    await user.click(
      screen.getByRole("button", { name: "Add Tactic (Potential)" }),
    );
    expect(onToggleGroup).not.toHaveBeenCalled();
  });
});
