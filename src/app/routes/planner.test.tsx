import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  createMemoryHistory,
  createRouter,
  RouterProvider,
} from "@tanstack/react-router";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import type { RouterContext } from "@/app/router-context";
import { plannerKeys } from "@/features/planner/api/planner-keys";
import type { PlannerTactic } from "@/features/planner/types/tactic";
import { snapshotKeys } from "@/features/snapshot/api/snapshot-keys";
import type { SnapshotSummary } from "@/features/snapshot/types/snapshot";
import { routeTree } from "@/routeTree.gen";
import {
  resolvePlannerClubFamilyIpcMock,
  resolvePlannerTacticIpcMock,
  setPlannerAvailableClubs,
  setPlannerTacticSaveError,
} from "@/testing/planner-ipc-mock";
import { resolveLoadDataIpcMock } from "@/testing/snapshot-ipc-mock";

function renderPlannerRoute() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  const router = createRouter({
    routeTree,
    context: { queryClient } satisfies RouterContext,
    defaultPreloadStaleTime: 0,
    history: createMemoryHistory({ initialEntries: ["/planner"] }),
  });

  render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );

  return { queryClient };
}

describe("planner route", () => {
  it("shows Load Data guidance when the active save has no snapshot", async () => {
    renderPlannerRoute();

    expect(
      await screen.findByRole("heading", { level: 1, name: "Squad Planner" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText("No data loaded for this save"),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Use Load Data to scan Football Manager/i),
    ).toBeInTheDocument();
  });

  it("persists a separate Reserves club in the club family", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona", "Barca Athletic", "Barcelona U19"]);
    renderPlannerRoute();

    const primaryClub = await screen.findByLabelText("Primary club");
    await user.selectOptions(primaryClub, "Barcelona");
    await user.click(screen.getByRole("button", { name: "Save club family" }));

    const addReserves = await screen.findByRole("button", {
      name: "Add Reserves source",
    });
    await user.click(addReserves);
    await user.selectOptions(
      screen.getByLabelText("Reserves club 1"),
      "Barca Athletic",
    );
    await user.click(screen.getByRole("button", { name: "Save club family" }));

    expect(
      (await screen.findAllByRole("option", { name: "Barca Athletic" })).length,
    ).toBeGreaterThan(0);
    await user.click(screen.getByRole("button", { name: "Add Youth source" }));
    expect(screen.getByLabelText("Youth club 1")).toBeInTheDocument();
    expect(screen.getByLabelText("Youth player level 1")).toBeInTheDocument();
    expect(
      resolvePlannerClubFamilyIpcMock().sources.some(
        (source) =>
          source.clubName === "Barca Athletic" &&
          source.team === "reserves" &&
          source.teamLevel === null,
      ),
    ).toBe(true);
    expect(
      within(screen.getByRole("main")).getByText(/Associated clubs/),
    ).toBeInTheDocument();
    expect(
      screen.getByText("11 linked lanes · 50% IP score weight"),
    ).toBeInTheDocument();
  });

  it("edits linked IP and OOP lanes with filtered roles and weight control", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    renderPlannerRoute();

    await screen.findByRole("heading", { level: 2, name: "Tactic editor" });

    const viewGroup = screen.getByRole("group", {
      name: "Tactic phase views",
    });
    const bothView = within(viewGroup).getByRole("button", { name: "Both" });
    expect(bothView).toHaveAttribute("aria-pressed", "true");
    bothView.focus();
    await user.keyboard("{ArrowLeft}");
    expect(
      within(viewGroup).getByRole("button", { name: "OOP" }),
    ).toHaveAttribute("aria-pressed", "true");
    expect(
      screen.queryByRole("heading", { level: 3, name: "In-Possession" }),
    ).not.toBeInTheDocument();
    await user.click(bothView);

    const firstIpLane = screen.getByRole("button", {
      name: "IP lane 1: GK, Goalkeeper",
    });
    firstIpLane.focus();
    await user.keyboard("{Enter}");

    const ipPosition = screen.getByRole("combobox", {
      name: "IP lane 1 position",
    });
    await user.selectOptions(ipPosition, "DL");

    const ipRole = screen.getByRole("combobox", { name: "IP lane 1 role" });
    expect(ipRole).toHaveValue("");
    expect(
      within(ipRole).queryByRole("option", { name: "Goalkeeper" }),
    ).not.toBeInTheDocument();
    await user.selectOptions(ipRole, "full_back_ip");

    const oopPosition = screen.getByRole("combobox", {
      name: "OOP lane 1 position",
    });
    await user.selectOptions(oopPosition, "DL");
    const oopRole = screen.getByRole("combobox", { name: "OOP lane 1 role" });
    expect(oopRole).toHaveValue("");
    await user.selectOptions(oopRole, "holding_full_back_oop");

    const weight = screen.getByRole("slider", { name: "IP score weight" });
    weight.focus();
    await user.keyboard(
      "{ArrowRight}{ArrowRight}{ArrowRight}{ArrowRight}{ArrowRight}",
    );
    expect(weight).toHaveValue("55");
    expect(screen.getByText("IP 55% / OOP 45%")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Save tactic" }));

    expect(resolvePlannerTacticIpcMock()).toMatchObject({ ipWeight: 0.55 });
    expect(resolvePlannerTacticIpcMock().lanes[0]).toMatchObject({
      ipPosition: "DL",
      ipRoleId: "full_back_ip",
      oopPosition: "DL",
      oopRoleId: "holding_full_back_oop",
    });
  });

  it("retains the edited tactic draft when save fails", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    setPlannerTacticSaveError("Tactic save failed");
    renderPlannerRoute();

    await screen.findByRole("heading", { level: 2, name: "Tactic editor" });
    const weight = screen.getByRole("slider", { name: "IP score weight" });
    weight.focus();
    await user.keyboard(
      "{ArrowRight}{ArrowRight}{ArrowRight}{ArrowRight}{ArrowRight}",
    );
    await user.click(screen.getByRole("button", { name: "Save tactic" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Tactic save failed",
    );
    expect(weight).toHaveValue("55");
    expect(resolvePlannerTacticIpcMock().ipWeight).toBe(0.5);
  });

  it("resets a dirty tactic draft when the active save changes", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    const { queryClient } = renderPlannerRoute();

    await screen.findByRole("heading", { level: 2, name: "Tactic editor" });
    const weight = screen.getByRole("slider", { name: "IP score weight" });
    weight.focus();
    await user.keyboard(
      "{ArrowRight}{ArrowRight}{ArrowRight}{ArrowRight}{ArrowRight}",
    );
    expect(weight).toHaveValue("55");

    queryClient.setQueryData(plannerKeys.tactic(), {
      ...resolvePlannerTacticIpcMock(),
      ipWeight: 0.2,
    });
    const snapshot = queryClient.getQueryData<SnapshotSummary>(
      snapshotKeys.current(),
    );
    if (!snapshot) {
      throw new Error("Expected a current snapshot in the planner query");
    }
    queryClient.setQueryData<SnapshotSummary | null>(
      snapshotKeys.current(),
      () => ({ ...snapshot, saveId: 2 }),
    );

    await waitFor(() =>
      expect(
        screen.getByRole("slider", { name: "IP score weight" }),
      ).toHaveValue("20"),
    );
  });

  it("blocks tactic saves while active-save data refreshes", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    const { queryClient } = renderPlannerRoute();

    await screen.findByRole("heading", { level: 2, name: "Tactic editor" });
    const weight = screen.getByRole("slider", { name: "IP score weight" });
    weight.focus();
    await user.keyboard(
      "{ArrowRight}{ArrowRight}{ArrowRight}{ArrowRight}{ArrowRight}",
    );
    expect(weight).toHaveValue("55");

    let resolveRefresh!: (tactic: PlannerTactic) => void;
    const refresh = new Promise<PlannerTactic>((resolve) => {
      resolveRefresh = resolve;
    });
    const refreshRequest = queryClient.fetchQuery({
      queryKey: plannerKeys.tactic(),
      queryFn: () => refresh,
    });

    const saveButton = screen.getByRole("button", { name: "Save tactic" });
    await waitFor(() => expect(saveButton).toBeDisabled());
    expect(screen.getByRole("status")).toHaveTextContent(
      "Refreshing active save",
    );
    await user.click(saveButton);
    expect(resolvePlannerTacticIpcMock().ipWeight).toBe(0.5);

    resolveRefresh(resolvePlannerTacticIpcMock());
    await refreshRequest;
    await waitFor(() => expect(saveButton).toBeEnabled());
  });

  it("keeps tactic saves blocked after an active-save refresh fails", async () => {
    const user = userEvent.setup();
    await resolveLoadDataIpcMock();
    setPlannerAvailableClubs(["Barcelona"]);
    const { queryClient } = renderPlannerRoute();

    await screen.findByRole("heading", { level: 2, name: "Tactic editor" });
    const weight = screen.getByRole("slider", { name: "IP score weight" });
    weight.focus();
    await user.keyboard(
      "{ArrowRight}{ArrowRight}{ArrowRight}{ArrowRight}{ArrowRight}",
    );

    const refreshRequest = queryClient.fetchQuery({
      queryKey: plannerKeys.tactic(),
      queryFn: () => Promise.reject(new Error("Tactic refresh failed")),
    });
    await expect(refreshRequest).rejects.toThrow("Tactic refresh failed");

    const saveButton = screen.getByRole("button", { name: "Save tactic" });
    await waitFor(() => expect(saveButton).toBeDisabled());
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Could not refresh the active save",
    );
    await user.click(saveButton);
    expect(resolvePlannerTacticIpcMock().ipWeight).toBe(0.5);
  });
});
