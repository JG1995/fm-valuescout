import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
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
});
