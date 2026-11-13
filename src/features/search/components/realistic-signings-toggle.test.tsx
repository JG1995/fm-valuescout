import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { FilterRule } from "../types/filter-rule";
import { RealisticSigningsToggle } from "./realistic-signings-toggle";

const ca: FilterRule = {
  id: "ca",
  field: "ca",
  op: "gte",
  value: { type: "integer", value: 150 },
};
const realistic: FilterRule = {
  id: "realistic",
  field: "realistic_signing",
  op: "is",
  value: { type: "bool", value: true },
};

describe("RealisticSigningsToggle", () => {
  it("adds the realistic-signing rule beside the other rules", async () => {
    const onRulesChange = vi.fn();
    render(
      <RealisticSigningsToggle
        rules={[ca]}
        clubName="Stevenage"
        onRulesChange={onRulesChange}
      />,
    );
    const toggle = screen.getByRole("checkbox", {
      name: "Only realistic signings",
    });
    expect(toggle).not.toBeChecked();
    expect(toggle).toHaveAccessibleDescription(/Stevenage/);

    await userEvent.click(toggle);

    expect(onRulesChange).toHaveBeenCalledWith([
      ca,
      { ...realistic, id: expect.any(String) },
    ]);
  });

  it("removes the rule when switched off", async () => {
    const onRulesChange = vi.fn();
    render(
      <RealisticSigningsToggle
        rules={[ca, realistic]}
        clubName="Stevenage"
        onRulesChange={onRulesChange}
      />,
    );
    const toggle = screen.getByRole("checkbox", {
      name: "Only realistic signings",
    });
    expect(toggle).toBeChecked();

    await userEvent.click(toggle);

    expect(onRulesChange).toHaveBeenCalledWith([ca]);
  });

  it("is off for a rule that asks for the unrealistic players", () => {
    render(
      <RealisticSigningsToggle
        rules={[{ ...realistic, op: "is_not" }]}
        clubName="Stevenage"
        onRulesChange={vi.fn()}
      />,
    );

    expect(screen.getByRole("checkbox")).not.toBeChecked();
  });

  it("is unavailable until a club is chosen", () => {
    render(
      <RealisticSigningsToggle
        rules={[]}
        clubName={null}
        onRulesChange={vi.fn()}
      />,
    );
    const toggle = screen.getByRole("checkbox", {
      name: "Only realistic signings",
    });

    expect(toggle).toBeDisabled();
    expect(toggle).toHaveAccessibleDescription(/Choose your club/);
  });
});
