import { act, fireEvent, render, screen } from "@testing-library/react";
import { Trash2 } from "lucide-react";
import { describe, expect, it, vi } from "vitest";
import { Button } from "@/components/ui/button/button";

describe("Button", () => {
  it("keeps the idle label out of the accessible name while loading", () => {
    render(
      <Button loading loadingLabel="Scanning…">
        Load Data
      </Button>,
    );

    expect(
      screen.getByRole("button", { name: "Scanning…" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("button")).toBeDisabled();
  });

  it("keeps the pending label out of the accessible name when idle", () => {
    render(<Button loadingLabel="Scanning…">Load Data</Button>);

    expect(
      screen.getByRole("button", { name: "Load Data" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("button")).toBeEnabled();
  });

  it("reserves the widest label so the button does not resize on load", () => {
    render(<Button loadingLabel="Scanning…">Load Data</Button>);

    // Both labels stay in the DOM sharing one grid cell; only one is exposed.
    expect(screen.getByText("Load Data")).toBeInTheDocument();
    expect(screen.getByText("Scanning…")).toBeInTheDocument();
  });

  it("keeps its name while a tick confirms the action", () => {
    render(<Button done>Save tactic</Button>);

    expect(screen.getByRole("button", { name: "Save tactic" })).toBeEnabled();
  });

  it("shows an icon-only button's name as a tooltip without describing it twice", () => {
    vi.useFakeTimers();
    try {
      render(<Button size="icon" icon={Trash2} aria-label="Delete filter" />);
      const button = screen.getByRole("button", { name: "Delete filter" });

      expect(button).not.toHaveAttribute("title");
      expect(screen.queryByText("Delete filter")).toBeNull();

      fireEvent.pointerEnter(button);
      act(() => vi.advanceTimersByTime(300));

      expect(screen.getByText("Delete filter")).toBeVisible();
      expect(button).not.toHaveAccessibleDescription();
    } finally {
      vi.useRealTimers();
    }
  });

  it("uses the error token for destructive actions", () => {
    render(<Button variant="destructive">Clear squad</Button>);

    expect(screen.getByRole("button", { name: "Clear squad" })).toHaveClass(
      "bg-error",
    );
  });
});
