import { render, screen } from "@testing-library/react";
import { hasFlag } from "country-flag-icons";
import { describe, expect, it } from "vitest";
import { OBSERVED_PLAYER_NATIONALITIES } from "@/testing/observed-player-nationalities";
import { NationalityCell, nationalityFlagFor } from "./nationality-cell";

describe("NationalityCell", () => {
  it("resolves every nationality in the representative snapshot", () => {
    expect(OBSERVED_PLAYER_NATIONALITIES).toHaveLength(224);

    for (const nationality of OBSERVED_PLAYER_NATIONALITIES) {
      const flag = nationalityFlagFor(nationality);

      expect(flag).toBeDefined();
      if (flag?.type === "package") {
        expect(hasFlag(flag.countryCode)).toBe(true);
      } else {
        expect(flag?.source).toMatch(/^data:image\/svg\+xml/);
      }
    }
  });

  it("renders all UK home nations with their stored-name labels", () => {
    render(
      <NationalityCell
        nationalities={["England", "Northern Ireland", "Scotland", "Wales"]}
      />,
    );

    expect(screen.getByRole("img", { name: "England" })).toHaveClass(
      "flag:GB-ENG",
    );
    expect(screen.getByRole("img", { name: "Northern Ireland" })).toHaveClass(
      "flag:GB-NIR",
    );
    expect(screen.getByRole("img", { name: "Scotland" })).toHaveClass(
      "flag:GB-SCT",
    );
    expect(screen.getByRole("img", { name: "Wales" })).toHaveClass(
      "flag:GB-WLS",
    );
    expect(screen.getByRole("img", { name: "England" })).toHaveAttribute(
      "title",
      "England",
    );
  });

  it("renders two and three nationalities in stored order", () => {
    const { rerender } = render(
      <NationalityCell nationalities={["England", "Wales"]} />,
    );

    expect(screen.getAllByRole("img").map((flag) => flag.ariaLabel)).toEqual([
      "England",
      "Wales",
    ]);

    rerender(
      <NationalityCell nationalities={["England", "Wales", "South Korea"]} />,
    );

    expect(screen.getAllByRole("img").map((flag) => flag.ariaLabel)).toEqual([
      "England",
      "Wales",
      "South Korea",
    ]);
  });

  it("renders a dash for empty values and truthful text for future values", () => {
    const { rerender } = render(<NationalityCell nationalities={[]} />);

    expect(screen.getByText("—")).toBeInTheDocument();

    rerender(<NationalityCell nationalities={["Atlantis"]} />);

    expect(screen.getByText("Atlantis")).toHaveAttribute("title", "Atlantis");
    expect(screen.queryByRole("img")).not.toBeInTheDocument();
  });
});
