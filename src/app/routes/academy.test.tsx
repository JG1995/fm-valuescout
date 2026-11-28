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
  setAcademyOutcomeError,
  setAcademyRemoveError,
} from "@/testing/academy-ipc-mock";
import {
  resolveSavePlannerClubFamilyIpcMock,
  setPlannerAvailableClubs,
} from "@/testing/planner-ipc-mock";
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

type IpcCall = { cmd: string; args?: Record<string, unknown> };

/**
 * Records every IPC request and holds the `held` command's requests until the
 * test releases them.
 */
function recordIpc(held?: string) {
  const internals = (
    window as unknown as {
      __TAURI_INTERNALS__: {
        invoke: (
          cmd: string,
          args?: Record<string, unknown>,
          options?: unknown,
        ) => Promise<unknown>;
      };
    }
  ).__TAURI_INTERNALS__;
  const invoke = internals.invoke;
  const calls: IpcCall[] = [];
  const releases: (() => void)[] = [];
  internals.invoke = (cmd, args, options) => {
    calls.push({ cmd, args });
    if (cmd !== held) return invoke(cmd, args, options);
    return new Promise((resolve) => {
      releases.push(() => resolve(invoke(cmd, args, options)));
    });
  };
  return { calls, releases };
}

describe("academy route", () => {
  async function loadConfiguredSave() {
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: { clubName: "Metro FC", clubUid: 1 },
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
      reportedCareerAppearances: valueOrDefault(
        overrides.reportedCareerAppearances,
        null,
      ),
      goals: valueOrDefault(overrides.goals, null),
      assists: valueOrDefault(overrides.assists, null),
      outcome: valueOrDefault(overrides.outcome, null),
      isGraduate: valueOrDefault(overrides.isGraduate, null),
      loanedOut: overrides.loanedOut ?? false,
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

  it("reads the class the URL names alongside the class list", async () => {
    await loadConfiguredSave();
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 0 }]);
    const ipc = recordIpc("list_academy_classes");
    renderAcademyRoute("/academy?view=class&classId=7");

    await waitFor(() => expect(ipc.releases).toHaveLength(1));
    await waitFor(() =>
      expect(ipc.calls).toContainEqual({
        cmd: "get_academy_class",
        args: { classId: 7 },
      }),
    );
    for (const release of ipc.releases) release();
    expect(
      await screen.findByRole("combobox", { name: "Academy class" }),
    ).toHaveValue("7");
  });

  it("paints the Overview with its class statistics", async () => {
    await loadConfiguredSave();
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 1 }]);
    setAcademyClassMembers(7, [
      academyMember({ playerUid: 77, lastKnownName: "Club prospect" }),
    ]);
    renderAcademyRoute("/academy");

    await screen.findByRole("heading", { level: 1, name: "Youth Academy" });
    expect(
      screen.getByRole("region", { name: "Academy outcomes" }),
    ).toHaveAttribute("aria-busy", "false");
  });

  it("reads the managed-club options only on the view whose dialogs offer them, without holding it", async () => {
    await loadConfiguredSave();
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 1 }]);
    setAcademyClassMembers(7, [
      academyMember({ playerUid: 77, lastKnownName: "Club prospect" }),
    ]);
    const ipc = recordIpc("list_managed_club_options");
    const optionReads = () =>
      ipc.calls.filter((call) => call.cmd === "list_managed_club_options");
    const { history } = renderAcademyRoute("/academy");

    await screen.findByRole("heading", { level: 1, name: "Youth Academy" });
    expect(optionReads()).toHaveLength(0);

    history.push("/academy?view=class&classId=7");

    // The Class view paints while the slow options read is still held.
    expect(
      await screen.findByRole("row", { name: /^Club prospect/ }),
    ).toBeInTheDocument();
    expect(optionReads()).toHaveLength(1);
    for (const release of ipc.releases) release();
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
      screen.getByText("Needs season stats from your linked FM save."),
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

  it("switches Academy classes from the Class workspace", async () => {
    const user = userEvent.setup();
    await loadConfiguredSave();
    setAcademyClasses([
      { id: 8, classYear: 2026, memberCount: 0 },
      { id: 7, classYear: 2025, memberCount: 0 },
    ]);
    const { router } = renderAcademyRoute("/academy?view=class&classId=7");

    const classPicker = await screen.findByRole("combobox", {
      name: "Academy class",
    });
    expect(classPicker).toHaveValue("7");
    expect(
      within(classPicker)
        .getAllByRole("option")
        .map((option) => option.textContent),
    ).toEqual(["Class of 2025", "Class of 2026"]);

    await user.selectOptions(classPicker, "8");

    await waitFor(() =>
      expect(router.state.location.search).toEqual({
        view: "class",
        classId: 8,
      }),
    );
    expect(classPicker).toHaveValue("8");
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
      within(
        screen.getByRole("region", { name: "Academy outcomes" }),
      ).getByText("Goals"),
    ).toHaveAccessibleDescription(
      "Needs season stats from your linked FM save.",
    );

    const outcomes = screen.getByRole("region", {
      name: "Academy outcomes",
    });
    expect(
      within(outcomes).getByTestId("academy-stat-graduates"),
    ).toHaveTextContent("—");
    expect(
      within(outcomes).getByTestId("academy-stat-sale-income"),
    ).toHaveTextContent("€0");
    expect(
      within(outcomes).getByTestId("academy-stat-goals"),
    ).toHaveTextContent("—");
    expect(
      within(outcomes).getByTestId("academy-stat-released-players"),
    ).toHaveTextContent("0");
    expect(
      within(outcomes).getByTestId("academy-stat-assists"),
    ).toHaveTextContent("—");

    const context = screen.getByRole("region", {
      name: "Academy context",
    });
    expect(
      within(context).getByTestId("academy-stat-classes"),
    ).toHaveTextContent("1");
    expect(
      within(context).getByTestId("academy-stat-tracked-players"),
    ).toHaveTextContent("2");
    expect(
      within(context).getByTestId("academy-stat-reported-senior-players"),
    ).toHaveTextContent("1");
  });

  it("adds supported outcome highlights to each class card", async () => {
    await loadConfiguredSave();
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 1 }]);
    setAcademyClassMembers(7, [
      academyMember({
        playerUid: 77,
        lastKnownName: "Sold graduate",
        reportedCareerAppearances: 1,
        goals: null,
        assists: null,
        outcome: {
          status: "sold",
          buyingClub: "North FC",
          saleFeeEur: 120_000,
          recorded: true,
        },
      }),
    ]);
    renderAcademyRoute();

    const card = await screen.findByRole("button", {
      name: "Open Class of 2026",
    });
    await waitFor(() => expect(card).toHaveTextContent("1 graduate"));
    expect(card).toHaveTextContent("€120k sale income");
  });

  it("renders nullable career columns and applies the exact graduation rule", async () => {
    await loadConfiguredSave();
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 2 }]);
    setAcademyClassMembers(7, [
      academyMember({
        playerUid: 77,
        lastKnownName: "Graduate candidate",
        reportedCareerAppearances: 2,
        isGraduate: false,
      }),
      academyMember({
        playerUid: 78,
        lastKnownName: "Non-graduate candidate",
        reportedCareerAppearances: 0,
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
    const roster = await screen.findByRole("table", {
      name: "Still at club (2)",
    });
    expect(
      within(roster).getByRole("columnheader", { name: "Career apps" }),
    ).toBeInTheDocument();
    expect(
      within(roster).getByRole("columnheader", { name: "Fee" }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("columnheader", { name: "Reported team" }),
    ).not.toBeInTheDocument();
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
    expect(screen.getByRole("combobox", { name: "Academy class" })).toHaveValue(
      "7",
    );
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

  it("guides an unconfigured save to My Club managed club", async () => {
    await resolveLoadDataIpcMock();
    renderAcademyRoute();

    expect(
      await screen.findByText("Academy needs your managed club"),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: "Open Managed Club" }),
    ).toHaveAttribute("href", "/my-club#managed-club");
  });

  it("shows Load Data guidance when the active save has no snapshot", async () => {
    renderAcademyRoute();

    expect(
      await screen.findByText("No data loaded for this save"),
    ).toBeInTheDocument();
    expect(
      screen.getByText("Use Load Data to read it from Football Manager."),
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

  it("recovers a Class URL without an identifier to Overview", async () => {
    await loadConfiguredSave();
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 0 }]);
    const { router } = renderAcademyRoute("/academy?view=class");

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
        name: "Search managed-club players",
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

  it("renders only playable Academy positions from complete nullable maps", async () => {
    const user = userEvent.setup();
    await loadConfiguredSave();
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 1 }]);
    setAcademyClassMembers(7, [
      academyMember({
        playerUid: 77,
        lastKnownName: "Complete prospect",
        positions: { AMR: 20, MR: 17, MC: 15, AMC: 14, GK: 0, SW: null },
      }),
    ]);
    setAcademyCandidates([
      {
        playerUid: 78,
        name: "Candidate prospect",
        age: 18,
        positions: { AMR: 20, MR: 17, MC: 15, AMC: 14, GK: 0, SW: null },
        currentClub: "Metro FC",
      },
    ]);
    renderAcademyRoute("/academy?view=class&classId=7");

    const row = await screen.findByRole("row", {
      name: /^Complete prospect/,
    });
    expect(row).toHaveTextContent("AMR, MR, MC");
    expect(row).not.toHaveTextContent("AMC");
    expect(row).not.toHaveTextContent("GK");
    expect(row).not.toHaveTextContent("SW");

    await user.click(screen.getByRole("button", { name: "Add players" }));
    const dialog = await screen.findByRole("dialog", {
      name: "Add players to Class of 2026",
    });
    const candidate = await within(dialog).findByRole("option", {
      name: /Candidate prospect/i,
    });
    expect(candidate).toHaveTextContent("AMR, MR, MC");
    expect(candidate).not.toHaveTextContent("AMC");
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
      within(
        await screen.findByRole("row", { name: /^Club prospect/ }),
      ).getByRole("button", { name: "Remove" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Remove Club prospect from Class of 2026?",
    });
    await user.click(
      within(dialog).getByRole("button", { name: "Remove from class" }),
    );
    await waitFor(() =>
      expect(screen.queryByRole("cell", { name: "Club prospect" })).toBeNull(),
    );
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Add players" })).toHaveFocus(),
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
      within(
        await screen.findByRole("row", { name: /^Club prospect/ }),
      ).getByRole("button", { name: "Remove" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Remove Club prospect from Class of 2026?",
    });
    await user.click(
      within(dialog).getByRole("button", { name: "Remove from class" }),
    );
    expect(await within(dialog).findByRole("alert")).toHaveTextContent(
      "The player could not be removed",
    );
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

    const prospectRemove = within(
      await screen.findByRole("row", { name: /^Club prospect/ }),
    ).getByRole("button", { name: "Remove" });
    const midfielderRemove = within(
      screen.getByRole("row", { name: /^Club midfielder/ }),
    ).getByRole("button", { name: "Remove" });
    await user.click(prospectRemove);
    const dialog = await screen.findByRole("dialog", {
      name: "Remove Club prospect from Class of 2026?",
    });
    await user.click(
      within(dialog).getByRole("button", { name: "Remove from class" }),
    );

    await waitFor(() => expect(prospectRemove).toBeDisabled());
    expect(midfielderRemove).toBeDisabled();
    expect(
      within(dialog).getByRole("button", { name: "Removing…" }),
    ).toBeDisabled();

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

  function movedMember() {
    return academyMember({
      playerUid: 77,
      lastKnownName: "Moved prospect",
      currentClub: "Other FC",
      state: "departed",
      outcome: {
        status: "sold",
        buyingClub: "Other FC",
        saleFeeEur: null,
        recorded: false,
      },
    });
  }

  it("groups a member who left by what the snapshot shows", async () => {
    await loadConfiguredSave();
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 3 }]);
    setAcademyClassMembers(7, [
      academyMember({ playerUid: 76, lastKnownName: "Club prospect" }),
      movedMember(),
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
        outcome: {
          status: "released",
          buyingClub: null,
          saleFeeEur: null,
          recorded: false,
        },
      }),
    ]);
    renderAcademyRoute("/academy?view=class&classId=7");

    expect(
      await screen.findByRole("heading", { level: 3, name: "Sold (1)" }),
    ).toBeInTheDocument();
    for (const name of ["Still at club (1)", "Released (1)"]) {
      expect(
        screen.getByRole("heading", { level: 3, name }),
      ).toBeInTheDocument();
    }
    const moved = screen.getByRole("row", {
      name: /^Moved prospect Sold to Other FC/,
    });
    expect(
      within(moved).getByRole("button", { name: "Release" }),
    ).toBeInTheDocument();

    // Only the member the snapshot no longer holds needs a word of explanation.
    expect(screen.getByRole("status")).toHaveTextContent(
      "Unavailable in the current snapshot",
    );
    const missing = screen.getByRole("row", { name: /^Missing prospect/ });
    expect(missing).toHaveTextContent("—");
    expect(
      within(missing).getByRole("button", { name: "Sell" }),
    ).toBeInTheDocument();
    // Nothing was recorded, so there is nothing to restore or to release again.
    for (const name of ["Restore", "Release"]) {
      expect(within(missing).queryByRole("button", { name })).toBeNull();
    }
  });

  it("records the fee of a sale the snapshot shows, with its club filled in", async () => {
    const user = userEvent.setup();
    await loadConfiguredSave();
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 1 }]);
    setAcademyClassMembers(7, [movedMember()]);
    renderAcademyRoute("/academy?view=class&classId=7");

    await user.click(
      within(
        await screen.findByRole("row", {
          name: /^Moved prospect Sold to Other FC/,
        }),
      ).getByRole("button", { name: "Sell" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Record sale for Moved prospect",
    });
    expect(
      within(dialog).getByRole("combobox", { name: "Buying club" }),
    ).toHaveValue("Other FC");
    expect(
      within(dialog).queryByRole("button", {
        name: "Clear record",
      }),
    ).toBeNull();
    await user.type(
      within(dialog).getByRole("spinbutton", { name: "Sale fee (€)" }),
      "250000",
    );
    await user.click(within(dialog).getByRole("button", { name: "Save sale" }));

    expect(
      await within(
        screen.getByRole("row", { name: /^Moved prospect Sold to Other FC/ }),
      ).findByText("€250k"),
    ).toBeInTheDocument();
  });

  it("marks a member the club has loaned out without a warning", async () => {
    await loadConfiguredSave();
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 1 }]);
    setAcademyClassMembers(7, [
      academyMember({
        playerUid: 77,
        lastKnownName: "Loaned prospect",
        currentClub: "Borrower FC",
        loanedOut: true,
      }),
    ]);
    renderAcademyRoute("/academy?view=class&classId=7");

    const row = (
      await screen.findByRole("cell", { name: /^Loaned prospect/ })
    ).closest("tr");
    expect(row).toHaveTextContent("On loan at Borrower FC");
    expect(row).toHaveClass("bg-info/10");
    expect(screen.queryByRole("status")).toBeNull();
  });

  it("records manual outcomes in labelled roster groups", async () => {
    const user = userEvent.setup();
    await loadConfiguredSave();
    setPlannerAvailableClubs([
      { clubName: "Metro FC", clubUid: 1 },
      { clubName: "Rovers FC", clubUid: 2 },
    ]);
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 1 }]);
    setAcademyClassMembers(7, [
      academyMember({ playerUid: 77, lastKnownName: "Club prospect" }),
    ]);
    renderAcademyRoute("/academy?view=class&classId=7");

    expect(
      await screen.findByRole("heading", {
        level: 3,
        name: "Still at club (1)",
      }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { level: 3, name: "Sold (0)" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { level: 3, name: "Released (0)" }),
    ).toBeInTheDocument();

    const initialRow = screen.getByRole("row", { name: /^Club prospect/ });
    const sellInitial = within(initialRow).getByRole("button", {
      name: "Sell",
    });
    expect(
      within(initialRow).getByRole("button", { name: "Release" }),
    ).toBeInTheDocument();
    expect(
      within(initialRow).getByRole("button", { name: "Remove" }),
    ).toBeInTheDocument();

    await user.click(sellInitial);
    const dialog = await screen.findByRole("dialog", {
      name: "Record sale for Club prospect",
    });
    const buyingClub = within(dialog).getByRole("combobox", {
      name: "Buying club",
    });
    await user.type(buyingClub, "Rov");
    expect(
      await within(dialog).findByRole("option", { name: "Rovers FC" }),
    ).toBeInTheDocument();
    await user.keyboard("{Escape}");
    expect(dialog).toBeInTheDocument();
    expect(buyingClub).toHaveValue("Rov");
    expect(
      within(dialog).queryByRole("option", { name: "Rovers FC" }),
    ).toBeNull();
    await user.type(buyingClub, "e");
    await user.click(
      await within(dialog).findByRole("option", { name: "Rovers FC" }),
    );
    const fee = within(dialog).getByRole("spinbutton", {
      name: "Sale fee (€)",
    });
    await user.type(fee, "1250000");
    await user.click(within(dialog).getByRole("button", { name: "Save sale" }));

    expect(await screen.findByText("Sold to Rovers FC")).toBeInTheDocument();
    expect(
      within(
        screen.getByRole("row", {
          name: /^Club prospect Sold to Rovers FC/,
        }),
      ).getByText("€1.3M"),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { level: 3, name: "Sold (1)" }),
    ).toBeInTheDocument();

    const sellRelease = within(
      screen.getByRole("row", { name: /^Club prospect Sold to Rovers FC/ }),
    ).getByRole("button", { name: "Release" });
    await user.click(sellRelease);
    const releaseDialog = await screen.findByRole("dialog", {
      name: "Mark Club prospect as released?",
    });
    await user.click(
      within(releaseDialog).getByRole("button", { name: "Cancel" }),
    );
    await waitFor(() => expect(sellRelease).toHaveFocus());

    await user.click(sellRelease);
    await user.click(
      within(
        await screen.findByRole("dialog", {
          name: "Mark Club prospect as released?",
        }),
      ).getByRole("button", { name: "Mark released" }),
    );
    const releasedRow = await screen.findByRole("row", {
      name: /^Club prospect Released/,
    });
    expect(releasedRow).toHaveClass("bg-surface-container-low");
    expect(
      screen.getByRole("heading", { level: 3, name: "Released (1)" }),
    ).toBeInTheDocument();

    await user.click(
      within(releasedRow).getByRole("button", { name: "Restore" }),
    );
    await user.click(
      within(
        await screen.findByRole("dialog", {
          name: "Clear the recorded outcome for Club prospect?",
        }),
      ).getByRole("button", { name: "Clear record" }),
    );
    expect(
      await screen.findByRole("heading", {
        level: 3,
        name: "Still at club (1)",
      }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { level: 3, name: "Sold (0)" }),
    ).toBeInTheDocument();
  });

  it("keeps the outcome dialog open when saving a sale fails", async () => {
    const user = userEvent.setup();
    await loadConfiguredSave();
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 1 }]);
    setAcademyClassMembers(7, [
      academyMember({ playerUid: 77, lastKnownName: "Club prospect" }),
    ]);
    setAcademyOutcomeError("Could not save the sale");
    renderAcademyRoute("/academy?view=class&classId=7");

    await user.click(
      within(
        await screen.findByRole("row", { name: /^Club prospect/ }),
      ).getByRole("button", { name: "Sell" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Record sale for Club prospect",
    });
    await user.type(
      within(dialog).getByRole("combobox", { name: "Buying club" }),
      "Rovers FC",
    );
    await user.type(
      within(dialog).getByRole("spinbutton", { name: "Sale fee (€)" }),
      "1000000",
    );
    await user.click(within(dialog).getByRole("button", { name: "Save sale" }));

    expect(await within(dialog).findByRole("alert")).toHaveTextContent(
      "Could not save the sale",
    );
    expect(
      within(dialog).getByRole("combobox", { name: "Buying club" }),
    ).toHaveValue("Rovers FC");
    setAcademyOutcomeError(null);
  });

  it("restores a sold player from the sale edit dialog", async () => {
    const user = userEvent.setup();
    await loadConfiguredSave();
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 1 }]);
    setAcademyClassMembers(7, [
      academyMember({
        playerUid: 77,
        lastKnownName: "Club prospect",
        outcome: {
          status: "sold",
          buyingClub: "Rovers FC",
          saleFeeEur: 1_250_000,
          recorded: true,
        },
      }),
    ]);
    renderAcademyRoute("/academy?view=class&classId=7");

    await user.click(
      within(
        await screen.findByRole("row", {
          name: /^Club prospect Sold to Rovers FC/,
        }),
      ).getByRole("button", { name: "Sell" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Edit sale for Club prospect",
    });
    await user.click(
      within(dialog).getByRole("button", {
        name: "Clear record",
      }),
    );

    expect(
      await screen.findByRole("heading", {
        level: 3,
        name: "Still at club (1)",
      }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { level: 3, name: "Sold (0)" }),
    ).toBeInTheDocument();
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
      name: "Search managed-club players",
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
      await within(reopened).findByText("No players match this search."),
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
        name: "Search managed-club players",
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
