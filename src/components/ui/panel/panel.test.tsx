import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { Panel } from "./panel";

describe("panel framing", () => {
  it("keeps feature actions operable when a redundant title is removed", async () => {
    const user = userEvent.setup();
    const upload = vi.fn();
    render(
      <Panel
        actions={
          <button type="button" onClick={upload}>
            Upload
          </button>
        }
        flush
      >
        <p>Dataset content</p>
      </Panel>,
    );
    expect(screen.queryByRole("heading")).toBeNull();
    expect(screen.getByText("Dataset content")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Upload" }));
    expect(upload).toHaveBeenCalledTimes(1);
  });

  it("uses the caller's nested heading level without losing its title", () => {
    render(
      <Panel title="Snapshot history" headingLevel={3}>
        History
      </Panel>,
    );
    expect(
      screen.getByRole("heading", { level: 3, name: "Snapshot history" }),
    ).toBeInTheDocument();
    expect(screen.queryByRole("heading", { level: 2 })).toBeNull();
  });
});
