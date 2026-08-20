import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { renderWithProviders } from "@/testing/render-with-providers";
import {
  getLastSuggestPlayersArgs,
  setSearchPlayersOverride,
  setSuggestPlayersOverride,
} from "@/testing/search-ipc-mock";
import type { PlayerSummary } from "../types/player-summary";
import { SUGGEST_DEBOUNCE_MS } from "./global-player-search";

function playerNamed(name: string, ca: number): PlayerSummary {
  return {
    uid: ca,
    name,
    age: 25,
    birthYear: 2001,
    birthDayOfYear: 80,
    nationalities: ["ENG"],
    club: "Test FC",
    division: "Premier Division",
    ca,
    pa: ca + 5,
    marketValueGbp: ca * 100_000,
  };
}

describe("top-bar global player search", () => {
  beforeEach(() => {
    setSuggestPlayersOverride(null);
    setSearchPlayersOverride([
      playerNamed("Alex Morgan", 160),
      playerNamed("Alexis Sanchez", 145),
      playerNamed("Patrice Evra", 130),
    ]);
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("focuses the search field on Ctrl+K from anywhere", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    const field = await screen.findByRole("combobox", {
      name: "Search players",
    });
    expect(field).not.toHaveFocus();

    await user.keyboard("{Control>}k{/Control}");
    expect(field).toHaveFocus();
  });

  it("debounces suggest_players by 200ms before querying", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    const user = userEvent.setup({
      advanceTimers: vi.advanceTimersByTime,
    });
    setSuggestPlayersOverride([
      { uid: 1, name: "Alex Morgan", ca: 160 },
      { uid: 2, name: "Alexis Sanchez", ca: 145 },
    ]);
    renderWithProviders();

    const field = await screen.findByRole("combobox", {
      name: "Search players",
    });
    await user.click(field);
    await user.type(field, "Alex");

    expect(getLastSuggestPlayersArgs()).toBeNull();

    await vi.advanceTimersByTimeAsync(SUGGEST_DEBOUNCE_MS - 1);
    expect(getLastSuggestPlayersArgs()).toBeNull();

    await vi.advanceTimersByTimeAsync(1);
    await waitFor(() => {
      expect(getLastSuggestPlayersArgs()?.query).toBe("Alex");
    });

    const list = await screen.findByRole("listbox", {
      name: "Player suggestions",
    });
    const options = within(list).getAllByRole("option");
    expect(options[0]).toHaveTextContent("Alex Morgan");
    expect(options[0]).toHaveTextContent("160");
    expect(options[1]).toHaveTextContent("Alexis Sanchez");
  });

  it("navigates to /players/$uid when a hit is activated", async () => {
    const user = userEvent.setup();
    setSuggestPlayersOverride([{ uid: 1, name: "Alex Morgan", ca: 160 }]);
    const { router } = renderWithProviders();

    const field = await screen.findByRole("combobox", {
      name: "Search players",
    });
    await user.type(field, "Alex");

    const option = await screen.findByRole("option", {
      name: /Alex Morgan/i,
    });
    await user.click(option);

    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/players/1");
      expect(router.state.location.search).toEqual({ view: "general" });
    });
  });

  it("activates the highlighted hit with ArrowDown and Enter", async () => {
    const user = userEvent.setup();
    setSuggestPlayersOverride([
      { uid: 1, name: "Alex Morgan", ca: 160 },
      { uid: 2, name: "Alexis Sanchez", ca: 145 },
    ]);
    const { router } = renderWithProviders();

    const field = await screen.findByRole("combobox", {
      name: "Search players",
    });
    await user.type(field, "Alex");
    await screen.findByRole("listbox", { name: "Player suggestions" });

    await user.keyboard("{ArrowDown}{Enter}");

    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/players/2");
    });
  });

  it("hides suggestions immediately when the query is cleared by typing", async () => {
    const user = userEvent.setup();
    setSuggestPlayersOverride([{ uid: 1, name: "Alex Morgan", ca: 160 }]);
    renderWithProviders();

    const field = await screen.findByRole("combobox", {
      name: "Search players",
    });
    await user.type(field, "Alex");
    await screen.findByRole("listbox", { name: "Player suggestions" });

    await user.clear(field);
    expect(
      screen.queryByRole("listbox", { name: "Player suggestions" }),
    ).not.toBeInTheDocument();
  });

  it("clears a non-empty field on Escape before dismissing the popover", async () => {
    const user = userEvent.setup();
    setSuggestPlayersOverride([{ uid: 1, name: "Alex Morgan", ca: 160 }]);
    renderWithProviders();

    const field = await screen.findByRole("combobox", {
      name: "Search players",
    });
    await user.type(field, "Alex");
    await screen.findByRole("listbox", { name: "Player suggestions" });

    await user.keyboard("{Escape}");
    expect(field).toHaveValue("");
    expect(
      screen.queryByRole("listbox", { name: "Player suggestions" }),
    ).not.toBeInTheDocument();
  });
});
