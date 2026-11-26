import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { Switch } from "@/components/ui/switch/switch";

describe("Switch", () => {
  it("reports its state as a switch and toggles from its label", async () => {
    const onChange = vi.fn();
    render(<Switch label="Position familiarity" checked onChange={onChange} />);

    const toggle = screen.getByRole("switch", { name: "Position familiarity" });
    expect(toggle).toBeChecked();

    await userEvent.click(screen.getByText("Position familiarity"));
    expect(onChange).toHaveBeenCalledWith(false);
  });

  it("toggles from the keyboard and not while disabled", async () => {
    const onChange = vi.fn();
    const { rerender } = render(
      <Switch
        label="Position familiarity"
        checked={false}
        onChange={onChange}
      />,
    );

    await userEvent.tab();
    await userEvent.keyboard(" ");
    expect(onChange).toHaveBeenCalledWith(true);

    onChange.mockClear();
    rerender(
      <Switch
        label="Position familiarity"
        checked={false}
        disabled
        onChange={onChange}
      />,
    );
    await userEvent.click(screen.getByText("Position familiarity"));
    expect(onChange).not.toHaveBeenCalled();
  });
});
