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
import { routeTree } from "@/routeTree.gen";
import {
  setAcademyClasses,
  setAcademyCreateError,
  setAcademyDeleteError,
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

describe("academy route", () => {
  async function loadConfiguredSave() {
    await resolveLoadDataIpcMock();
    resolveSavePlannerClubFamilyIpcMock({
      primaryClub: "Metro FC",
      sources: [],
    });
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
});
