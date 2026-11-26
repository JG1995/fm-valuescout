import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { InfoTip, Tooltip } from "@/components/ui/tooltip/tooltip";

describe("Tooltip", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("describes its trigger and shows once the pointer rests on it", () => {
    render(
      <Tooltip content="Ranks players within their own position">
        <button type="button">Percentiles</button>
      </Tooltip>,
    );
    const trigger = screen.getByRole("button", { name: "Percentiles" });
    const tooltip = screen.getByRole("tooltip", { hidden: true });

    expect(trigger).toHaveAccessibleDescription(
      "Ranks players within their own position",
    );
    expect(tooltip).not.toBeVisible();

    fireEvent.pointerEnter(trigger);
    expect(tooltip).not.toBeVisible();
    act(() => vi.advanceTimersByTime(300));
    expect(tooltip).toBeVisible();

    fireEvent.pointerLeave(trigger);
    act(() => vi.advanceTimersByTime(100));
    expect(tooltip).not.toBeVisible();
  });

  it("leaves when its trigger is pressed", () => {
    render(
      <Tooltip content="Ranks players within their own position">
        <button type="button">Percentiles</button>
      </Tooltip>,
    );
    const trigger = screen.getByRole("button", { name: "Percentiles" });
    const tooltip = screen.getByRole("tooltip", { hidden: true });

    fireEvent.pointerEnter(trigger);
    act(() => vi.advanceTimersByTime(300));
    expect(tooltip).toBeVisible();

    fireEvent.pointerDown(trigger);
    expect(tooltip).not.toBeVisible();
  });

  it("shows on keyboard focus and leaves on Escape", () => {
    render(
      <Tooltip content="Kept through the next Load Data">
        <button type="button">Keep</button>
      </Tooltip>,
    );
    const tooltip = screen.getByRole("tooltip", { hidden: true });

    act(() => screen.getByRole("button", { name: "Keep" }).focus());
    expect(tooltip).toBeVisible();

    fireEvent.keyDown(document, { key: "Escape" });
    expect(tooltip).not.toBeVisible();
  });

  it("places the description on a control inside the trigger", () => {
    render(
      <Tooltip content="Only positions a player can play">
        {(tooltipId) => (
          <label>
            <input type="checkbox" aria-describedby={tooltipId} />
            Position familiarity
          </label>
        )}
      </Tooltip>,
    );

    expect(
      screen.getByRole("checkbox", { name: "Position familiarity" }),
    ).toHaveAccessibleDescription("Only positions a player can play");
  });

  it("makes a trigger that is not a control reachable by keyboard", () => {
    render(
      <Tooltip content="At least one career appearance" focusable>
        <span>Graduates</span>
      </Tooltip>,
    );

    expect(screen.getByText("Graduates")).toHaveAttribute("tabindex", "0");
  });

  it("offers a named button where nothing else can carry the tooltip", () => {
    render(<InfoTip label="About the benefit">How much a loan helps</InfoTip>);

    expect(
      screen.getByRole("button", { name: "About the benefit" }),
    ).toHaveAccessibleDescription("How much a loan helps");
  });
});
