import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import {
  formatPlayerBasicCell,
  formatTableDynamicCell,
  TABLE_NUMERIC_CELL_CLASS,
  TABLE_TEXT_CELL_CLASS,
  TableScoreContent,
} from "./table-cells";

describe("shared table cells", () => {
  it("keeps text and numeric cell geometry identical to current callers", () => {
    expect(TABLE_TEXT_CELL_CLASS).toBe(
      "h-table-row-height-two-line max-w-0 truncate px-2 align-middle text-body-sm",
    );
    expect(TABLE_NUMERIC_CELL_CLASS).toBe(
      "h-table-row-height-two-line whitespace-nowrap px-2 align-middle text-right font-mono text-mono-sm text-on-surface tabular-nums",
    );
  });

  it("renders loading for missing rows and missing honesty for absent values", () => {
    expect(formatTableDynamicCell(undefined, "ca")).toBe("…");
    expect(formatTableDynamicCell({ dynamicValues: {} }, "ca")).toBe("—");
    expect(formatTableDynamicCell({ dynamicValues: { ca: null } }, "ca")).toBe(
      "—",
    );
    expect(formatTableDynamicCell({ dynamicValues: { ca: 72 } }, "ca")).toBe(
      "72",
    );
  });

  it("renders zero dynamic values as a value instead of missing", () => {
    expect(formatTableDynamicCell({ dynamicValues: { ca: 0 } }, "ca")).toBe(
      "0",
    );
  });

  it("formats height with its centimeter unit while keeping loading and missing", () => {
    expect(
      formatTableDynamicCell({ dynamicValues: { height: 188 } }, "height"),
    ).toBe("188 cm");
    expect(formatTableDynamicCell(undefined, "height")).toBe("…");
    expect(formatTableDynamicCell({ dynamicValues: {} }, "height")).toBe("—");
    expect(
      formatTableDynamicCell({ dynamicValues: { height: null } }, "height"),
    ).toBe("—");
  });

  describe("player basic cells", () => {
    const row = {
      name: "Ada Example",
      age: 25,
      nationalities: ["England"],
      club: "Example FC",
      division: "Premier Division",
      ca: 140,
      pa: 165,
      marketValueGbp: 2_500_000,
    };

    it("presents values with currency and accessible titles", () => {
      expect(formatPlayerBasicCell(row, "name")).toEqual({
        text: "Ada Example",
        title: "Ada Example",
        numeric: false,
      });
      expect(formatPlayerBasicCell(row, "age")).toEqual({
        text: "25",
        title: "25",
        numeric: false,
      });
      expect(formatPlayerBasicCell(row, "ca")).toEqual({
        text: "140",
        numeric: true,
      });
      expect(formatPlayerBasicCell(row, "value")).toEqual({
        text: "€2.5M",
        numeric: true,
      });
    });

    it("presents loading placeholders and missing value honesty", () => {
      expect(formatPlayerBasicCell(undefined, "ca")).toEqual({
        text: "…",
        numeric: true,
      });
      expect(formatPlayerBasicCell(undefined, "name")).toEqual({
        text: "…",
        numeric: false,
      });
      expect(formatPlayerBasicCell(undefined, "age")).toEqual({
        text: "…",
        numeric: false,
      });
      expect(formatPlayerBasicCell({ ...row, age: null }, "age")).toEqual({
        text: "—",
        numeric: false,
      });
      expect(
        formatPlayerBasicCell({ ...row, marketValueGbp: null }, "value"),
      ).toEqual({ text: "—", numeric: true });
      expect(formatPlayerBasicCell({ ...row, club: null }, "club")).toEqual({
        text: "—",
        numeric: false,
      });
    });
  });

  it("renders scores with the table badge and preserves loading vs missing", () => {
    const { rerender } = render(
      <TableScoreContent score={82} roleName="Deep-lying playmaker" />,
    );
    expect(
      screen.getByRole("img", {
        name: "Deep-lying playmaker: 82, Excellent",
      }),
    ).toBeInTheDocument();

    rerender(<TableScoreContent score={undefined} roleName="Scout" />);
    expect(screen.getByText("—")).toBeInTheDocument();

    rerender(
      <TableScoreContent score={undefined} roleName="Scout" isLoading />,
    );
    expect(screen.getByText("…")).toBeInTheDocument();
  });

  it("prefers loading over a retained numeric score", () => {
    render(<TableScoreContent score={72} roleName="Scout" isLoading />);
    expect(screen.getByText("…")).toBeInTheDocument();
    expect(screen.queryByRole("img", { name: "Scout: 72, Good" })).toBeNull();
  });
});
