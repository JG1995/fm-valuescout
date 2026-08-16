import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  createMemoryHistory,
  createRouter,
  RouterProvider,
} from "@tanstack/react-router";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it } from "vitest";
import type { RouterContext } from "@/app/router-context";
import { playerKeys } from "@/features/player-profile/api/player-keys";
import { staffKeys } from "@/features/staff/api/staff-keys";
import { STAFF_PROFILE_ATTRIBUTE_GROUPS } from "@/features/staff/utils/staff-profile-attributes";
import { routeTree } from "@/routeTree.gen";
import { useLayoutStore } from "@/stores/use-layout-store";
import { resolveLoadDataIpcMock } from "@/testing/snapshot-ipc-mock";
import {
  fixtureStaffDetail,
  setStaffDetailOverride,
} from "@/testing/staff-ipc-mock";

function renderStaffProfileRoute(initialEntry: string) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, staleTime: 60_000 } },
  });
  const router = createRouter({
    routeTree,
    context: { queryClient } satisfies RouterContext,
    defaultPreloadStaleTime: 0,
    history: createMemoryHistory({ initialEntries: [initialEntry] }),
  });
  return {
    router,
    queryClient,
    ...render(
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>,
    ),
  };
}

describe("staff profile route", () => {
  beforeEach(() => {
    useLayoutStore.setState({ railExpanded: true });
    setStaffDetailOverride(undefined);
  });

  it("shows staff summary, grouped attributes, and current role fit", async () => {
    await resolveLoadDataIpcMock();
    setStaffDetailOverride(fixtureStaffDetail());
    renderStaffProfileRoute("/staff/101");

    expect(
      await screen.findByRole("heading", { level: 1, name: "Alex Coach" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Metro FC · Premier Division")).toBeInTheDocument();
    expect(screen.getByText("30/04/1982 (44)")).toBeInTheDocument();
    expect(screen.getAllByText("15").length).toBeGreaterThan(0);
    expect(screen.getByText("160")).toBeInTheDocument();
    expect(
      screen.queryByRole("tablist", { name: "Staff attribute groups" }),
    ).not.toBeInTheDocument();
    for (const group of STAFF_PROFILE_ATTRIBUTE_GROUPS) {
      expect(
        screen.getByRole("region", { name: group.title }),
      ).toBeInTheDocument();
      for (const key of group.keys) {
        const label = key.replaceAll(/([a-z])([A-Z])/g, "$1 $2");
        expect(screen.getAllByText(label, { exact: true })).toHaveLength(1);
      }
    }
    expect(screen.getAllByText("Coach — Fitness").length).toBeGreaterThan(0);
    expect(screen.getByText("Scout")).toBeInTheDocument();
    expect(
      screen.getByRole("img", {
        name: "Coach — Fitness current score: 85, Excellent",
      }),
    ).toHaveClass("text-score-4");
    expect(screen.queryByText("Wonderkid Mentality")).toBeNull();
    expect(screen.queryByText("Select a pitch position")).toBeNull();
  });

  it("retains every current staff attribute when hidden info is concealed", async () => {
    await resolveLoadDataIpcMock();
    setStaffDetailOverride(fixtureStaffDetail());
    const user = userEvent.setup();
    const { queryClient } = renderStaffProfileRoute("/staff/101?tab=mental");
    queryClient.setQueryData([...playerKeys.all, "probe"], []);
    queryClient.setQueryData([...staffKeys.all, "probe"], []);

    expect(
      await screen.findByRole("region", { name: "Mental" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("region", { name: "Coaching" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("region", { name: "Knowledge" }),
    ).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Hide hidden info" }));

    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: "Reveal hidden info" }),
      ).toBeInTheDocument();
      expect(screen.queryByText("160")).toBeNull();
      expect(screen.getByText("145")).toBeInTheDocument();
      expect(screen.getByText("Adaptability")).toBeInTheDocument();
    });
    expect(
      queryClient.getQueryState([...playerKeys.all, "probe"])?.isInvalidated,
    ).toBe(true);
    expect(
      queryClient.getQueryState([...staffKeys.all, "probe"])?.isInvalidated,
    ).toBe(true);
    expect(
      within(screen.getByRole("region", { name: "Mental" })).getByText(
        "Authority",
      ),
    ).toBeInTheDocument();
  });

  it("uses the player-profile tier and row presentation for staff attributes", async () => {
    await resolveLoadDataIpcMock();
    const staff = fixtureStaffDetail();
    setStaffDetailOverride({
      ...staff,
      attributes: {
        ...staff.attributes,
        Attacking: 3,
        Defending: 8,
        Fitness: 13,
        Possession: 18,
        Technical: null,
      },
    });
    renderStaffProfileRoute("/staff/101");

    const coaching = await screen.findByRole("region", { name: "Coaching" });
    for (const [value, tier, tierLabel] of [
      [3, 1, "Weak"],
      [8, 2, "Average"],
      [13, 3, "Good"],
      [18, 4, "Excellent"],
    ] as const) {
      const attribute = within(coaching).getByText(String(value));
      expect(attribute).toHaveAttribute("data-tier", String(tier));
      expect(attribute).toHaveAttribute("title", tierLabel);
      expect(attribute).toHaveClass(
        "inline-flex",
        "min-w-7",
        "justify-center",
        "rounded-sm",
        "bg-surface-container-high",
        `data-[tier=${tier}]:bg-score-${tier}/10`,
        `data-[tier=${tier}]:text-score-${tier}`,
      );
    }
    const missingValue = within(coaching).getByText("—");
    expect(missingValue).toHaveClass("text-on-surface-variant");
    expect(missingValue).not.toHaveAttribute("data-tier");
    expect(missingValue).not.toHaveClass("bg-surface-container-high");
    const attackingRow = within(coaching).getByText("Attacking").parentElement;
    expect(attackingRow).toHaveClass(
      "flex",
      "min-h-9",
      "min-w-0",
      "items-center",
      "justify-between",
      "gap-3",
      "border-b",
      "border-outline-variant/70",
    );
    expect(within(coaching).getByText("Attacking")).toHaveClass(
      "truncate",
      "text-body-md",
      "text-on-surface-variant",
    );
    expect(within(coaching).getByText("3").parentElement).toHaveClass(
      "shrink-0",
      "font-mono",
      "text-mono-sm",
      "tabular-nums",
    );
  });

  it("summarizes the highest available role score", async () => {
    await resolveLoadDataIpcMock();
    setStaffDetailOverride(
      fixtureStaffDetail({
        roleScores: [
          {
            roleId: "coach_fitness",
            displayName: "Coach — Fitness",
            score: 60,
          },
          { roleId: "scout", displayName: "Scout", score: 90 },
          { roleId: "physio", displayName: "Physio", score: null },
        ],
      }),
    );
    renderStaffProfileRoute("/staff/101");

    const summary = await screen.findByRole("region", {
      name: "Alex Coach summary",
    });
    expect(within(summary).getByText("Scout")).toBeInTheDocument();
    expect(within(summary).getByText("90")).toBeInTheDocument();
  });

  it("virtualizes role fit inside its own scrollport", async () => {
    await resolveLoadDataIpcMock();
    setStaffDetailOverride(
      fixtureStaffDetail({
        roleScores: [
          { roleId: "low", displayName: "Low role", score: 10 },
          { roleId: "tie_first", displayName: "First tied role", score: 95 },
          { roleId: "missing", displayName: "Unavailable role", score: null },
          { roleId: "tie_second", displayName: "Second tied role", score: 95 },
          ...Array.from({ length: 16 }, (_, index) => ({
            roleId: `middle_${index + 1}`,
            displayName: `Middle role ${index + 1}`,
            score: 90 - index,
          })),
        ],
      }),
    );
    renderStaffProfileRoute("/staff/101");

    const scrollport = await screen.findByTestId("staff-role-fit-scroller");
    expect(scrollport).toHaveAccessibleName("Staff role fit scores");
    const table = within(scrollport).getByRole("table");
    expect(table).toHaveAttribute("aria-rowcount", "21");
    const rows = within(table).getAllByRole("row");
    expect(rows.length).toBeLessThan(21);
    expect(rows[1]).toHaveTextContent("First tied role");
    expect(rows[2]).toHaveTextContent("Second tied role");
    for (const row of rows.slice(1)) {
      const index = Number(row.dataset.index);
      expect(row).toHaveAttribute("aria-rowindex", String(index + 2));
    }
  });

  it("renders an empty state for an unknown staff UID", async () => {
    await resolveLoadDataIpcMock();
    renderStaffProfileRoute("/staff/999");
    expect(
      await screen.findByText("Staff member not in this snapshot", {
        exact: true,
      }),
    ).toBeInTheDocument();
  });
});
