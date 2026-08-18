import {
  QueryClient,
  QueryClientProvider,
  useQuery,
} from "@tanstack/react-query";
import {
  createMemoryHistory,
  createRouter,
  RouterProvider,
} from "@tanstack/react-router";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import type { RouterContext } from "@/app/router-context";
import { academyClassesQueryOptions } from "@/features/academy/api/academy-classes-query-options";
import { setBridgeStatusIpcMockMode } from "@/features/memory-read/api/bridge-status-ipc-mock";
import {
  DEFAULT_PLAYER_CAP,
  useLoadDataPreferences,
} from "@/features/memory-read/stores/use-load-data-preferences";
import { staffKeys } from "@/features/staff/api/staff-keys";
import { routeTree } from "@/routeTree.gen";
import {
  deferAcademyClassesFetch,
  setAcademyClasses,
} from "@/testing/academy-ipc-mock";
import { renderWithProviders } from "@/testing/render-with-providers";
import {
  getLastLoadDataIpcArgs,
  resolveBusyLoadDataRequest,
  setLoadDataIpcMockMode,
} from "@/testing/snapshot-ipc-mock";

function AcademyCacheProbe() {
  const { data: classes } = useQuery(academyClassesQueryOptions);
  return (
    <output data-testid="academy-cache-value">
      {classes?.[0]?.classYear ?? "none"}
    </output>
  );
}

async function renderTopBarWithAcademyProbe() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, staleTime: 60_000 } },
  });
  await queryClient.fetchQuery(academyClassesQueryOptions);
  const router = createRouter({
    routeTree,
    context: { queryClient } satisfies RouterContext,
    defaultPreloadStaleTime: 0,
    history: createMemoryHistory({ initialEntries: ["/settings"] }),
  });

  render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
      <AcademyCacheProbe />
    </QueryClientProvider>,
  );
}

// Load Data lives in the shell top bar, so its outcome banner is asserted here
// rather than against the bridge panel that used to own the button.
describe("app top bar", () => {
  beforeEach(() => {
    setBridgeStatusIpcMockMode("ready");
    setLoadDataIpcMockMode("success");
    useLoadDataPreferences.setState({
      playerCapEnabled: false,
      playerCap: DEFAULT_PLAYER_CAP,
    });
  });

  afterEach(() => {
    resolveBusyLoadDataRequest();
  });

  it("reports ingest success after load_data", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));

    expect(
      await screen.findByText(/Loaded 3 players into the database/i),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Scan 1\.2s, ingest 400ms, total 1\.6s/i),
    ).toBeInTheDocument();
  });

  it("sends unlimited maxAccepted when the player cap is off", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    await screen.findByText(/Loaded 3 players into the database/i);

    expect(getLastLoadDataIpcArgs()).toEqual({ maxAccepted: null });
  });

  it("sends the configured player cap when the toggle is on", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(
      await screen.findByRole("checkbox", { name: "Cap players" }),
    );
    const limitField = await screen.findByLabelText("Player limit");
    expect(limitField).toHaveValue(DEFAULT_PLAYER_CAP);

    await user.clear(limitField);
    await user.type(limitField, "250");
    await user.click(screen.getByRole("button", { name: "Load Data" }));
    await screen.findByText(/Loaded 3 players into the database/i);

    expect(getLastLoadDataIpcArgs()).toEqual({ maxAccepted: 250 });
  });

  it("warns that a capped scan produced a partial ingest", async () => {
    setLoadDataIpcMockMode("truncatedSuccess");
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));

    expect(
      await screen.findByText(/the scan was capped at 500 players/i),
    ).toBeInTheDocument();
  });

  it("reports a scan failure from load_data", async () => {
    setLoadDataIpcMockMode("scanFailed");
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));

    expect(await screen.findByText(/Scan failed/i)).toBeInTheDocument();
    expect(
      screen.getByText(/scan produced zero player candidates/i),
    ).toBeInTheDocument();
  });

  it("reports an ingest failure from load_data", async () => {
    setLoadDataIpcMockMode("ingestFailed");
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));

    expect(await screen.findByText(/Ingest failed/i)).toBeInTheDocument();
    expect(screen.getByText(/dump validation failed/i)).toBeInTheDocument();
  });

  it.each([
    ["success", /Loaded 3 players into the database/i],
    ["scanFailed", /Scan failed/i],
  ] as const)("dismisses a completed %s outcome", async (mode, message) => {
    setLoadDataIpcMockMode(mode);
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    expect(await screen.findByText(message)).toBeInTheDocument();

    await user.click(
      screen.getByRole("button", { name: "Dismiss Load Data outcome" }),
    );

    expect(screen.queryByText(message)).not.toBeInTheDocument();
  });

  it("drops a failure banner once the user switches save", async () => {
    setLoadDataIpcMockMode("scanFailed");
    const user = userEvent.setup();
    renderWithProviders({ initialEntries: ["/settings"] });

    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    expect(await screen.findByText(/Scan failed/i)).toBeInTheDocument();

    await user.type(await screen.findByLabelText("New save"), "Youth intake");
    await user.click(screen.getByRole("button", { name: "Create save" }));
    await user.selectOptions(
      screen.getByRole("combobox", { name: "Active save" }),
      "2",
    );

    // The failure described the previous save's scan, not this one's.
    expect(screen.queryByText(/Scan failed/i)).not.toBeInTheDocument();
  });

  it("swaps the button label for the scan phase while the request is pending", async () => {
    setLoadDataIpcMockMode("busy");
    const user = userEvent.setup();
    renderWithProviders();

    await user.click(await screen.findByRole("button", { name: "Load Data" }));

    expect(
      await screen.findByRole("button", { name: "Scanning…" }),
    ).toBeDisabled();
  });

  it("refetches an active Academy query after Load Data", async () => {
    const user = userEvent.setup();
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 0 }]);
    await renderTopBarWithAcademyProbe();

    expect(await screen.findByTestId("academy-cache-value")).toHaveTextContent(
      "2026",
    );
    setAcademyClasses([{ id: 8, classYear: 2027, memberCount: 0 }]);

    await user.click(screen.getByRole("button", { name: "Load Data" }));

    await waitFor(() =>
      expect(screen.getByTestId("academy-cache-value")).toHaveTextContent(
        "2027",
      ),
    );
  });

  it("invalidates cached Staff data after Load Data", async () => {
    const user = userEvent.setup();
    const { queryClient } = renderWithProviders();
    const staffProbeKey = [...staffKeys.all, "probe"];
    queryClient.setQueryData(staffProbeKey, []);

    await user.click(await screen.findByRole("button", { name: "Load Data" }));

    await waitFor(() =>
      expect(queryClient.getQueryState(staffProbeKey)?.isInvalidated).toBe(
        true,
      ),
    );
  });

  it("refetches an active Academy query after switching saves", async () => {
    const user = userEvent.setup();
    setAcademyClasses([{ id: 7, classYear: 2026, memberCount: 0 }]);
    await renderTopBarWithAcademyProbe();

    expect(await screen.findByTestId("academy-cache-value")).toHaveTextContent(
      "2026",
    );
    await user.type(screen.getByLabelText("New save"), "Youth intake");
    await user.click(screen.getByRole("button", { name: "Create save" }));
    await user.selectOptions(
      await screen.findByRole("combobox", { name: "Active save" }),
      "2",
    );

    setAcademyClasses([{ id: 8, classYear: 2027, memberCount: 0 }]);
    const releaseClassesFetch = deferAcademyClassesFetch();
    await user.selectOptions(
      screen.getByRole("combobox", { name: "Active save" }),
      "1",
    );

    await waitFor(() =>
      expect(screen.getByTestId("academy-cache-value")).toHaveTextContent(
        "none",
      ),
    );
    releaseClassesFetch();
    await waitFor(() =>
      expect(screen.getByTestId("academy-cache-value")).toHaveTextContent(
        "2027",
      ),
    );
  });

  it("invalidates cached Staff data after switching saves", async () => {
    const user = userEvent.setup();
    const { queryClient } = renderWithProviders({
      initialEntries: ["/settings"],
    });
    const staffProbeKey = [...staffKeys.all, "probe"];
    queryClient.setQueryData(staffProbeKey, []);

    await user.type(await screen.findByLabelText("New save"), "Youth intake");
    await user.click(screen.getByRole("button", { name: "Create save" }));
    await user.selectOptions(
      await screen.findByRole("combobox", { name: "Active save" }),
      "2",
    );

    await waitFor(() =>
      expect(queryClient.getQueryState(staffProbeKey)?.isInvalidated).toBe(
        true,
      ),
    );
  });
});
