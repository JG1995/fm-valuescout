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
        currentActive
        potentialActive={false}
        disabled={false}
        onToggleGroup={onToggleGroup}
      />,
    );

    const current = screen.getByRole("button", {
      name: "Add Tactic (Current)",
    });
    const potential = screen.getByRole("button", {
      name: "Add Tactic (Potential)",
    });
    expect(current).toHaveAttribute("aria-pressed", "true");
    expect(potential).toHaveAttribute("aria-pressed", "false");

    await user.click(current);
    await user.click(potential);
    expect(onToggleGroup.mock.calls).toEqual([["current"], ["potential"]]);
  });

  it("does not invoke callbacks while disabled", async () => {
    const user = userEvent.setup();
    const onToggleGroup = vi.fn();
    render(
      <TacticColumnToggles
        currentActive={false}
        potentialActive={false}
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
