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
          { group: "current", label: "Roles", active: true },
          { group: "custom", label: "Custom", active: false },
        ]}
        disabled={false}
        onToggleGroup={onToggleGroup}
      />,
    );

    const roles = screen.getByRole("button", {
      name: "Add Tactic (Roles)",
    });
    const custom = screen.getByRole("button", {
      name: "Add Tactic (Custom)",
    });
    expect(roles).toHaveAttribute("aria-pressed", "true");
    expect(custom).toHaveAttribute("aria-pressed", "false");

    await user.click(roles);
    await user.click(custom);
    expect(onToggleGroup.mock.calls).toEqual([["current"], ["custom"]]);
  });

  it("does not invoke callbacks while disabled or for a group that cannot add", async () => {
    const user = userEvent.setup();
    const onToggleGroup = vi.fn();
    const { rerender } = render(
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

    rerender(
      <TacticColumnToggles
        groups={[
          { group: "current", label: "Roles", active: false },
          { group: "custom", label: "Custom", active: false, canAdd: false },
        ]}
        disabled={false}
        onToggleGroup={onToggleGroup}
      />,
    );
    const custom = screen.getByRole("button", { name: "Add Tactic (Custom)" });
    expect(custom).toBeDisabled();
    await user.click(custom);
    expect(onToggleGroup).not.toHaveBeenCalled();
  });
});
