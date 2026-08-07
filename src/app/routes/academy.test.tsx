import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  createMemoryHistory,
  createRouter,
  RouterProvider,
} from "@tanstack/react-router";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { RouterContext } from "@/app/router-context";
import type { AcademyMember } from "@/features/academy/types/academy";
import { routeTree } from "@/routeTree.gen";
import {
  deferAcademyAssignment,
  deferAcademyRemoval,
  setAcademyAssignError,
  setAcademyCandidates,
  setAcademyClasses,
  setAcademyClassMembers,
  setAcademyCreateError,
  setAcademyDeleteError,
  setAcademyRemoveError,
} from "@/testing/academy-ipc-mock";
import { resolveSavePlannerClubFamilyIpcMock } from "@/testing/planner-ipc-mock";
import { resolveLoadDataIpcMock } from "@/testing/snapshot-ipc-mock";

function renderAcademyRoute(initialEntry = "/academy") {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, staleTime: 0 } },
  });
  const history = createMemoryHistory({ initialEntries: [initialEntry] });
  const router = createRouter({
    routeTree,
    context: { queryClient } satisfies RouterContext,
    defaultPreloadStaleTime: 0,
    history,
  });

  render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );

  return { history, queryClient, router };
}

function valueOrDefault<T>(value: T | undefined, fallback: T): T {
  return value === undefined ? fallback : value;
}

describe("academy route", () => {
  async function loadConfiguredSave() {
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: "Metro FC",
      sources: [],
    });
  }

  function academyMember(
    overrides: Partial<AcademyMember> &
      Pick<AcademyMember, "playerUid" | "lastKnownName">,
  ): AcademyMember {
    return {
      playerUid: overrides.playerUid,
      lastKnownName: overrides.lastKnownName,
      currentName: valueOrDefault(
        overrides.currentName,
        overrides.lastKnownName,
      ),
      state: valueOrDefault(overrides.state, "resolved"),
      age: valueOrDefault(overrides.age, 18),
      nationalities: valueOrDefault(overrides.nationalities, ["ENG"]),
      positions: valueOrDefault(overrides.positions, { ST: 20 }),
      currentClub: valueOrDefault(overrides.currentClub, "Metro FC"),
      parentClub: valueOrDefault(overrides.parentClub, null),
      teamLevel: valueOrDefault(overrides.teamLevel, "youth"),
      pa: valueOrDefault(overrides.pa, 150),
      determination: valueOrDefault(overrides.determination, 15),
      heightCm: valueOrDefault(overrides.heightCm, 180),
      preferredFoot: valueOrDefault(overrides.preferredFoot, "right"),
      seniorLeagueAppearances: valueOrDefault(
        overrides.seniorLeagueAppearances,
        null,
      ),
      goals: valueOrDefault(overrides.goals, null),
      assists: valueOrDefault(overrides.assists, null),
      internationalCaps: valueOrDefault(overrides.internationalCaps, null),
      saleFeeGbp: valueOrDefault(overrides.saleFeeGbp, null),
      isReleased: valueOrDefault(overrides.isReleased, null),
      isGraduate: valueOrDefault(overrides.isGraduate, null),
    };
  }

  it("opens the Youth Academy page for a loaded save", async () => {
    await loadConfiguredSave();
    renderAcademyRoute();

    expect(
      await screen.findByRole("heading", {
        level: 1,
        name: "Youth Academy",
      }),
    ).toBeInTheDocument();
  });

  it("shows persisted classes and keeps workspace selection in the URL", async () => {
    const user = userEvent.setup();
    await loadConfiguredSave();
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 3 }]);
    const { router } = renderAcademyRoute();

    expect(await screen.findByText("Class of 2026")).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "Overview" })).toHaveAttribute(
      "aria-selected",
      "true",
    );

    await user.click(
      screen.getByRole("button", { name: "Open Class of 2026" }),
    );
    expect(router.state.location.search).toEqual({ view: "class", classId: 7 });
    expect(screen.getByRole("tab", { name: "Class" })).toHaveAttribute(
      "aria-selected",
      "true",
    );

    await user.click(screen.getByRole("tab", { name: "Graduates" }));
    expect(router.state.location.search).toEqual({ view: "graduates" });
    expect(
      screen.getByText(/senior league appearances are not available/i),
    ).toBeInTheDocument();
  });

  it("orders Academy classes from oldest to newest", async () => {
    await loadConfiguredSave();
    setAcademyClasses([
      { id: 8, classYear: 2026, memberCount: 0 },
      { id: 7, classYear: 2025, memberCount: 0 },
    ]);
    renderAcademyRoute();

    expect(
      (await screen.findAllByRole("button", { name: /Open Class of/ })).map(
        (button) => button.getAttribute("aria-label"),
      ),
    ).toEqual(["Open Class of 2025", "Open Class of 2026"]);
  });

  it("shows an actionable empty state when the Class workspace has no classes", async () => {
    const user = userEvent.setup();
    await loadConfiguredSave();
    setAcademyClasses([]);
    renderAcademyRoute();

    await user.click(await screen.findByRole("tab", { name: "Class" }));

    expect(
      await screen.findByText("No academy classes available"),
    ).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Create class" }));
    expect(
      await screen.findByRole("dialog", { name: "Create academy class" }),
    ).toBeInTheDocument();
  });

  it("does not offer deletion for an automatically managed class", async () => {
    await loadConfiguredSave();
    setAcademyClasses([
      { id: 7, classYear: 2025, memberCount: 0, isAutomatic: true },
    ]);
    renderAcademyRoute("/academy?view=class&classId=7");

    expect(
      await screen.findByRole("heading", { level: 2, name: "Class of 2025" }),
    ).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Delete class" })).toBeNull();
  });

  it("shows reported senior counts and keeps unsupported aggregates unavailable", async () => {
    await loadConfiguredSave();
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 2 }]);
    setAcademyClassMembers(7, [
      academyMember({
        playerUid: 77,
        lastKnownName: "Senior snapshot player",
        teamLevel: "senior",
      }),
      academyMember({
        playerUid: 78,
        lastKnownName: "Youth snapshot player",
        teamLevel: "youth",
      }),
    ]);
    renderAcademyRoute();

    await waitFor(() =>
      expect(
        screen.getByTestId("academy-stat-reported-senior-players"),
      ).toHaveTextContent("1"),
    );
    expect(screen.getByTestId("academy-stat-graduates")).toHaveTextContent("—");
    expect(
      screen.getByTestId("academy-stat-goals").parentElement,
    ).toHaveTextContent(/not available from the current memory reader/i);
  });

  it("renders nullable career columns and applies the exact graduation rule", async () => {
    await loadConfiguredSave();
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 2 }]);
    setAcademyClassMembers(7, [
      academyMember({
        playerUid: 77,
        lastKnownName: "Graduate candidate",
        seniorLeagueAppearances: 2,
        isGraduate: false,
      }),
      academyMember({
        playerUid: 78,
        lastKnownName: "Non-graduate candidate",
        seniorLeagueAppearances: 0,
        isGraduate: true,
      }),
    ]);
    renderAcademyRoute("/academy?view=graduates");

    expect(await screen.findByText("Graduate candidate")).toBeInTheDocument();
    expect(
      screen.queryByText("Non-graduate candidate"),
    ).not.toBeInTheDocument();
    expect(
      within(
        screen.getByRole("table", { name: "Youth Academy graduates" }),
      ).getByRole("cell", { name: "2" }),
    ).toBeInTheDocument();

    const user = userEvent.setup();
    await user.click(screen.getByRole("tab", { name: "Class" }));
    expect(
      await screen.findByRole("columnheader", { name: "Senior league apps" }),
    ).toBeInTheDocument();
    expect(screen.getAllByText("—").length).toBeGreaterThan(0);
  });

  it("supports keyboard navigation across Academy workspaces", async () => {
    const user = userEvent.setup();
    await loadConfiguredSave();
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 3 }]);
    const { router } = renderAcademyRoute();

    const overview = await screen.findByRole("tab", { name: "Overview" });
    overview.focus();
    await user.keyboard("{End}");
    const classTab = screen.getByRole("tab", { name: "Class" });
    expect(classTab).toHaveFocus();
    expect(classTab).toHaveAttribute("aria-selected", "true");
    expect(router.state.location.search).toEqual({ view: "class", classId: 7 });

    await user.keyboard("{ArrowLeft}");
    const graduates = screen.getByRole("tab", { name: "Graduates" });
    expect(graduates).toHaveFocus();
    expect(router.state.location.search).toEqual({ view: "graduates" });

    await user.keyboard("{Home}");
    expect(overview).toHaveFocus();
    expect(router.state.location.search).toEqual({ view: "overview" });
  });

  it("creates a class with the snapshot year prefilled", async () => {
    const user = userEvent.setup();
    await loadConfiguredSave();
    setAcademyClasses([]);
    renderAcademyRoute();

    await user.click(
      await screen.findByRole("button", { name: "Create class" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Create academy class",
    });
    const year = within(dialog).getByRole("spinbutton", { name: "Class year" });
    expect(year).toHaveValue(2026);
    expect(within(dialog).getByText("Class of 2026")).toBeInTheDocument();
    await user.clear(year);
    await user.type(year, "2025");
    expect(within(dialog).getByText("Class of 2025")).toBeInTheDocument();
    await user.click(
      within(dialog).getByRole("button", { name: "Create class" }),
    );

    expect(await screen.findByText("Class of 2025")).toBeInTheDocument();
  });

  it("requires confirmation before deleting a class", async () => {
    const user = userEvent.setup();
    await loadConfiguredSave();
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 2 }]);
    renderAcademyRoute("/academy?view=class&classId=7");

    await user.click(
      await screen.findByRole("button", { name: "Delete class" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Delete Class of 2026?",
    });
    expect(
      within(dialog).getByText(/removes the class and its 2 tracked players/i),
    ).toBeInTheDocument();
    await user.click(
      within(dialog).getByRole("button", { name: "Delete class" }),
    );

    await waitFor(() => expect(screen.queryByText("Class of 2026")).toBeNull());
    expect(screen.getByRole("tab", { name: "Overview" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
  });

  it("keeps the delete confirmation visible when deletion fails", async () => {
    const user = userEvent.setup();
    await loadConfiguredSave();
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 2 }]);
    setAcademyDeleteError("The class could not be deleted");
    renderAcademyRoute("/academy?view=class&classId=7");

    await user.click(
      await screen.findByRole("button", { name: "Delete class" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Delete Class of 2026?",
    });
    await user.click(
      within(dialog).getByRole("button", { name: "Delete class" }),
    );

    expect(await within(dialog).findByRole("alert")).toHaveTextContent(
      "The class could not be deleted",
    );
    expect(
      screen.getByRole("heading", { level: 2, name: "Class of 2026" }),
    ).toBeInTheDocument();
    setAcademyDeleteError(null);
  });

  it("restores focus to the delete trigger after cancelling", async () => {
    const user = userEvent.setup();
    await loadConfiguredSave();
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 2 }]);
    renderAcademyRoute("/academy?view=class&classId=7");

    const trigger = await screen.findByRole("button", { name: "Delete class" });
    await user.click(trigger);
    const dialog = await screen.findByRole("dialog", {
      name: "Delete Class of 2026?",
    });
    await user.click(within(dialog).getByRole("button", { name: "Cancel" }));

    await waitFor(() => expect(trigger).toHaveFocus());
  });

  it("clears a previous delete error when reopened", async () => {
    const user = userEvent.setup();
    await loadConfiguredSave();
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 2 }]);
    setAcademyDeleteError("The class could not be deleted");
    renderAcademyRoute("/academy?view=class&classId=7");

    const trigger = await screen.findByRole("button", { name: "Delete class" });
    await user.click(trigger);
    let dialog = await screen.findByRole("dialog", {
      name: "Delete Class of 2026?",
    });
    await user.click(
      within(dialog).getByRole("button", { name: "Delete class" }),
    );
    expect(await within(dialog).findByRole("alert")).toBeInTheDocument();

    setAcademyDeleteError(null);
    await user.click(within(dialog).getByRole("button", { name: "Cancel" }));
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());

    await user.click(trigger);
    dialog = await screen.findByRole("dialog", {
      name: "Delete Class of 2026?",
    });
    expect(within(dialog).queryByRole("alert")).toBeNull();
  });

  it("keeps create errors and the form draft visible", async () => {
    const user = userEvent.setup();
    await loadConfiguredSave();
    setAcademyCreateError("Class of 2025 already exists");
    renderAcademyRoute();

    await user.click(
      await screen.findByRole("button", { name: "Create class" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Create academy class",
    });
    const year = within(dialog).getByRole("spinbutton", { name: "Class year" });
    await user.clear(year);
    await user.type(year, "2025");
    await user.click(
      within(dialog).getByRole("button", { name: "Create class" }),
    );

    expect(
      await within(dialog).findByText("Class of 2025 already exists"),
    ).toBeInTheDocument();
    expect(year).toHaveValue(2025);
    setAcademyCreateError(null);
  });

  it("guides an unconfigured save to Planner club setup", async () => {
    await resolveLoadDataIpcMock();
    renderAcademyRoute();

    expect(
      await screen.findByText("Set up your club family"),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: "Open Planner club setup" }),
    ).toHaveAttribute("href", "/planner?view=clubs");
  });

  it("shows Load Data guidance when the active save has no snapshot", async () => {
    renderAcademyRoute();

    expect(
      await screen.findByText("No data loaded for this save"),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Use Load Data to scan Football Manager/i),
    ).toBeInTheDocument();
  });

  it("recovers a deleted or unknown class URL to Overview", async () => {
    await loadConfiguredSave();
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 0 }]);
    const { router } = renderAcademyRoute("/academy?view=class&classId=99");

    await waitFor(() =>
      expect(router.state.location.search).toEqual({ view: "overview" }),
    );
    expect(screen.getByRole("tab", { name: "Overview" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
  });

  it("assigns only club-family candidates and refreshes the class roster", async () => {
    const user = userEvent.setup();
    await loadConfiguredSave();
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 0 }]);
    setAcademyCandidates([
      {
        playerUid: 77,
        name: "Club prospect",
        age: 18,
        positions: { ST: 20 },
        currentClub: "Metro FC",
      },
      {
        playerUid: 78,
        name: "Club midfielder",
        age: 17,
        positions: { MC: 20 },
        currentClub: "Metro FC",
      },
    ]);
    renderAcademyRoute("/academy?view=class&classId=7");

    await user.click(
      await screen.findByRole("button", { name: "Add players" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Add players to Class of 2026",
    });
    await user.type(
      within(dialog).getByRole("combobox", {
        name: "Search club-family players",
      }),
      "prospect",
    );
    expect(
      within(dialog).getByRole("option", { name: /Club prospect/i }),
    ).toBeInTheDocument();
    expect(
      within(dialog).queryByRole("option", { name: /Club midfielder/i }),
    ).toBeNull();

    await user.click(
      within(dialog).getByRole("option", { name: /Club prospect/i }),
    );
    expect(
      await screen.findByRole("cell", { name: "Club prospect" }),
    ).toBeInTheDocument();
    expect(
      within(
        screen.getByRole("tabpanel", { name: "Class", hidden: true }),
      ).getByTestId("academy-stat-tracked-players"),
    ).toHaveTextContent("1");
  });

  it("removes a member before making them available to another class", async () => {
    const user = userEvent.setup();
    await loadConfiguredSave();
    setAcademyClasses([
      { id: 7, classYear: 2026, memberCount: 1 },
      { id: 8, classYear: 2027, memberCount: 0 },
    ]);
    setAcademyCandidates([
      {
        playerUid: 77,
        name: "Club prospect",
        age: 18,
        positions: { ST: 20 },
        currentClub: "Metro FC",
      },
    ]);
    setAcademyClassMembers(7, [
      academyMember({ playerUid: 77, lastKnownName: "Club prospect" }),
    ]);
    renderAcademyRoute("/academy?view=class&classId=7");

    await user.click(
      await screen.findByRole("button", {
        name: "Remove Club prospect from Class of 2026",
      }),
    );
    await waitFor(() =>
      expect(screen.queryByRole("cell", { name: "Club prospect" })).toBeNull(),
    );

    await user.click(screen.getByRole("tab", { name: "Overview" }));
    await user.click(
      await screen.findByRole("button", { name: "Open Class of 2027" }),
    );
    await user.click(screen.getByRole("button", { name: "Add players" }));
    expect(
      await screen.findByRole("option", { name: /Club prospect/i }),
    ).toBeInTheDocument();
  });

  it("keeps a member visible when removal fails", async () => {
    const user = userEvent.setup();
    await loadConfiguredSave();
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 2 }]);
    setAcademyClassMembers(7, [
      academyMember({ playerUid: 77, lastKnownName: "Club prospect" }),
      academyMember({ playerUid: 78, lastKnownName: "Club midfielder" }),
    ]);
    setAcademyRemoveError("The player could not be removed");
    renderAcademyRoute("/academy?view=class&classId=7");

    await user.click(
      await screen.findByRole("button", {
        name: "Remove Club prospect from Class of 2026",
      }),
    );
    const prospectRow = screen.getByRole("row", { name: /^Club prospect/ });
    expect(await within(prospectRow).findByRole("alert")).toHaveTextContent(
      "The player could not be removed",
    );
    expect(
      within(screen.getByRole("row", { name: /^Club midfielder/ })).queryByRole(
        "alert",
      ),
    ).toBeNull();
    expect(
      screen.getByRole("cell", { name: "Club prospect" }),
    ).toBeInTheDocument();
    setAcademyRemoveError(null);
  });

  it("prevents concurrent removals while marking only the selected player pending", async () => {
    const user = userEvent.setup();
    await loadConfiguredSave();
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 2 }]);
    setAcademyClassMembers(7, [
      academyMember({ playerUid: 77, lastKnownName: "Club prospect" }),
      academyMember({ playerUid: 78, lastKnownName: "Club midfielder" }),
    ]);
    const releaseRemoval = deferAcademyRemoval();
    renderAcademyRoute("/academy?view=class&classId=7");

    const prospectRemove = await screen.findByRole("button", {
      name: "Remove Club prospect from Class of 2026",
    });
    const midfielderRemove = screen.getByRole("button", {
      name: "Remove Club midfielder from Class of 2026",
    });
    await user.click(prospectRemove);

    await waitFor(() => expect(prospectRemove).toBeDisabled());
    expect(midfielderRemove).toBeDisabled();
    expect(prospectRemove.querySelector("svg.animate-spin")).not.toBeNull();
    expect(midfielderRemove.querySelector("svg.animate-spin")).toBeNull();

    releaseRemoval();
    await waitFor(() =>
      expect(screen.queryByRole("cell", { name: "Club prospect" })).toBeNull(),
    );
  });

  it("keeps assignment errors and the picker visible", async () => {
    const user = userEvent.setup();
    await loadConfiguredSave();
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 0 }]);
    setAcademyCandidates([
      {
        playerUid: 77,
        name: "Club prospect",
        age: 18,
        positions: { ST: 20 },
        currentClub: "Metro FC",
      },
    ]);
    setAcademyAssignError("The player is already assigned elsewhere");
    renderAcademyRoute("/academy?view=class&classId=7");

    await user.click(
      await screen.findByRole("button", { name: "Add players" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Add players to Class of 2026",
    });
    await user.click(
      within(dialog).getByRole("option", { name: /Club prospect/i }),
    );

    expect(await within(dialog).findByRole("alert")).toHaveTextContent(
      "The player is already assigned elsewhere",
    );
    expect(
      within(dialog).getByRole("option", { name: /Club prospect/i }),
    ).toBeInTheDocument();
    setAcademyAssignError(null);
  });

  it("keeps departed and unresolved members with accessible warnings", async () => {
    await loadConfiguredSave();
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 2 }]);
    setAcademyClassMembers(7, [
      academyMember({
        playerUid: 77,
        lastKnownName: "Departed prospect",
        currentClub: "Other FC",
        state: "departed",
      }),
      academyMember({
        playerUid: 78,
        lastKnownName: "Missing prospect",
        currentName: null,
        age: null,
        currentClub: null,
        teamLevel: null,
        pa: null,
        determination: null,
        heightCm: null,
        preferredFoot: null,
        state: "unresolved",
      }),
    ]);
    renderAcademyRoute("/academy?view=class&classId=7");

    expect(
      await screen.findByRole("cell", { name: /^Departed prospect/ }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("cell", { name: /^Missing prospect/ }),
    ).toBeInTheDocument();
    expect(screen.getAllByRole("status")[0]).toHaveTextContent(
      "No longer in your club family",
    );
    expect(screen.getAllByRole("status")[1]).toHaveTextContent(
      "Unavailable in the current snapshot",
    );
    expect(
      screen.getByRole("row", { name: /^Missing prospect/ }),
    ).toHaveTextContent("—");
  });

  it("supports keyboard assignment and restores focus after closing the picker", async () => {
    const user = userEvent.setup();
    await loadConfiguredSave();
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 0 }]);
    setAcademyCandidates([
      {
        playerUid: 77,
        name: "Club prospect",
        age: 18,
        positions: { ST: 20 },
        currentClub: "Metro FC",
      },
    ]);
    renderAcademyRoute("/academy?view=class&classId=7");

    const trigger = await screen.findByRole("button", { name: "Add players" });
    await user.click(trigger);
    const dialog = await screen.findByRole("dialog", {
      name: "Add players to Class of 2026",
    });
    const search = within(dialog).getByRole("combobox", {
      name: "Search club-family players",
    });
    await user.keyboard("{Enter}");
    expect(
      await screen.findByRole("cell", { name: "Club prospect" }),
    ).toBeInTheDocument();

    await user.click(trigger);
    const reopened = await screen.findByRole("dialog", {
      name: "Add players to Class of 2026",
    });
    expect(
      await within(reopened).findByText(
        "No unclassified club-family players match this search.",
      ),
    ).toBeInTheDocument();
    await user.click(within(reopened).getByRole("button", { name: "Cancel" }));
    await waitFor(() => expect(trigger).toHaveFocus());
    expect(search).not.toBeInTheDocument();
  });

  it("keeps keyboard candidates visible and locks the picker while assigning", async () => {
    const user = userEvent.setup();
    const originalScrollIntoView = Object.getOwnPropertyDescriptor(
      HTMLElement.prototype,
      "scrollIntoView",
    );
    const scrollIntoView = vi.fn();
    Object.defineProperty(HTMLElement.prototype, "scrollIntoView", {
      configurable: true,
      value: scrollIntoView,
    });

    try {
      await loadConfiguredSave();
      setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 0 }]);
      setAcademyCandidates([
        {
          playerUid: 77,
          name: "Club prospect",
          age: null,
          positions: { ST: 20 },
          currentClub: "Metro FC",
        },
        {
          playerUid: 78,
          name: "Club midfielder",
          age: 17,
          positions: { MC: 20 },
          currentClub: "Metro FC",
        },
      ]);
      const releaseAssignment = deferAcademyAssignment();
      renderAcademyRoute("/academy?view=class&classId=7");

      await user.click(
        await screen.findByRole("button", { name: "Add players" }),
      );
      const dialog = await screen.findByRole("dialog", {
        name: "Add players to Class of 2026",
      });
      const search = within(dialog).getByRole("combobox", {
        name: "Search club-family players",
      });
      const prospect = await within(dialog).findByRole("option", {
        name: /Club prospect/i,
      });
      expect(prospect).toHaveTextContent("Metro FC · —");
      expect(prospect).not.toHaveTextContent("— years");

      scrollIntoView.mockClear();
      search.focus();
      await user.keyboard("{ArrowDown}");
      await waitFor(() =>
        expect(scrollIntoView).toHaveBeenCalledWith({ block: "nearest" }),
      );

      await user.keyboard("{Enter}");
      await waitFor(() => expect(search).toBeDisabled());
      expect(
        within(dialog).getByRole("option", { name: /Club midfielder/i }),
      ).toBeDisabled();
      expect(
        within(dialog).getByRole("button", { name: "Cancel" }),
      ).toBeDisabled();
      await user.keyboard("changed");
      expect(search).toHaveValue("");

      releaseAssignment();
      expect(
        await screen.findByRole("cell", { name: "Club midfielder" }),
      ).toBeInTheDocument();
    } finally {
      if (originalScrollIntoView) {
        Object.defineProperty(
          HTMLElement.prototype,
          "scrollIntoView",
          originalScrollIntoView,
        );
      } else {
        delete (HTMLElement.prototype as { scrollIntoView?: unknown })
          .scrollIntoView;
      }
    }
  });
});
