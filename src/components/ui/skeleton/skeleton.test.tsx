import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { LoadingRegion, Skeleton, TableSkeleton } from "./skeleton";

describe("loading placeholders", () => {
  it("announces what is loading and hides the blocks from assistive technology", () => {
    render(
      <LoadingRegion label="Loading squad">
        <Skeleton className="h-4 w-20" />
      </LoadingRegion>,
    );

    const region = screen.getByRole("status");
    expect(region).toHaveTextContent("Loading squad");
    expect(region).toHaveAttribute("aria-busy", "true");
    expect(region.querySelector("[aria-hidden='true']")).not.toBeNull();
  });

  it("draws the requested number of table rows under one label", () => {
    render(<TableSkeleton label="Loading players" rows={3} />);

    const region = screen.getByRole("status");
    expect(region).toHaveTextContent("Loading players");
    // One header band plus three rows.
    expect(
      region.querySelectorAll(":scope > [aria-hidden='true']"),
    ).toHaveLength(4);
  });
});
