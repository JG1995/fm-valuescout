import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";
import { invokeCommand } from "@/lib/tauri-client";
import {
  resolvePlannerTacticIpcMock,
  resolvePlannerTacticOptionsIpcMock,
} from "@/testing/planner-ipc-mock";
import { type PlannerContext, plannerKeys } from "../api/planner-keys";
import type { PlannerTactic } from "../types/tactic";
import { PlannerTacticEditor } from "./planner-tactic-editor";
import {
  TacticContextBoundary,
  type TacticContextBoundaryState,
} from "./tactic-context-boundary";

vi.mock("@/lib/tauri-client", () => ({ invokeCommand: vi.fn() }));

const contextA = { saveId: 1, contextToken: "token-a" };
const contextB = { saveId: 2, contextToken: "token-b" };

function queryClient() {
  return new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
}

function provider(client: QueryClient, children: ReactNode) {
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

function BoundaryState({ state }: { state: TacticContextBoundaryState }) {
  return (
    <>
      <output>
        {JSON.stringify({
          hasTactic: state.tactic !== undefined,
          hasOptions: state.options !== undefined,
          isPending: state.isPending,
          initialError: state.initialError?.message ?? null,
          isRefetchError: state.isRefetchError,
          refreshError: state.refreshError?.message ?? null,
          readOnly: state.readOnly,
        })}
      </output>
      <button type="button" onClick={state.retryBoth}>
        Retry both
      </button>
    </>
  );
}

describe("TacticContextBoundary", () => {
  it.each(["get_planner_tactic", "get_planner_tactic_options"])(
    "reports an initial error when %s fails without data",
    async (failedCommand) => {
      vi.mocked(invokeCommand).mockImplementation(async (command) => {
        if (command === failedCommand) {
          throw new Error(`${failedCommand} failed`);
        }
        return command === "get_planner_tactic"
          ? resolvePlannerTacticIpcMock()
          : resolvePlannerTacticOptionsIpcMock();
      });

      render(
        provider(
          queryClient(),
          <TacticContextBoundary context={contextA}>
            {(state) => <BoundaryState state={state} />}
          </TacticContextBoundary>,
        ),
      );

      await waitFor(() =>
        expect(screen.getByRole("status")).toHaveTextContent(
          `"initialError":"${failedCommand} failed"`,
        ),
      );
      expect(screen.getByRole("status")).toHaveTextContent(
        '"isRefetchError":false',
      );
      expect(screen.getByRole("status")).toHaveTextContent('"readOnly":false');
      const callsBeforeRetry = vi.mocked(invokeCommand).mock.calls.length;

      await userEvent.click(screen.getByRole("button", { name: "Retry both" }));

      await waitFor(() =>
        expect(vi.mocked(invokeCommand).mock.calls).toHaveLength(
          callsBeforeRetry + 2,
        ),
      );
      expect(
        vi
          .mocked(invokeCommand)
          .mock.calls.slice(-2)
          .map(([command]) => command),
      ).toEqual(["get_planner_tactic", "get_planner_tactic_options"]);
    },
  );

  it("reports refresh failure only while both cached values remain available", async () => {
    const client = queryClient();
    client.setQueryData(
      plannerKeys.tactic(contextA),
      resolvePlannerTacticIpcMock(),
    );
    client.setQueryData(
      plannerKeys.tacticOptions(contextA),
      resolvePlannerTacticOptionsIpcMock(),
    );
    vi.mocked(invokeCommand).mockRejectedValue(new Error("refresh failed"));

    render(
      provider(
        client,
        <TacticContextBoundary context={contextA}>
          {(state) => <BoundaryState state={state} />}
        </TacticContextBoundary>,
      ),
    );

    await waitFor(() =>
      expect(screen.getByRole("status")).toHaveTextContent(
        '"isRefetchError":true',
      ),
    );
    expect(screen.getByRole("status")).toHaveTextContent('"hasTactic":true');
    expect(screen.getByRole("status")).toHaveTextContent('"hasOptions":true');
    expect(screen.getByRole("status")).toHaveTextContent('"initialError":null');
    expect(screen.getByRole("status")).toHaveTextContent('"readOnly":true');
  });
});

type Deferred<T> = {
  promise: Promise<T>;
  resolve: (value: T) => void;
};

function deferred<T>(): Deferred<T> {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((next) => {
    resolve = next;
  });
  return { promise, resolve };
}

function contextKey(context: PlannerContext) {
  return `${context.saveId}:${context.contextToken}`;
}

describe("PlannerTacticEditor captured context", () => {
  it("remounts local state and settles a delayed save only into its originating context", async () => {
    const user = userEvent.setup();
    const client = queryClient();
    const tacticA = resolvePlannerTacticIpcMock();
    const tacticB = resolvePlannerTacticIpcMock();
    tacticB.lanes[0].ipWeight = 0.3;
    client.setQueryData(plannerKeys.tactic(contextA), tacticA);
    client.setQueryData(plannerKeys.tactic(contextB), tacticB);
    const persisted = new Map<string, PlannerTactic>([
      [contextKey(contextA), tacticA],
      [contextKey(contextB), tacticB],
    ]);
    const pendingByContext = new Map<string, Deferred<PlannerTactic>>([
      [contextKey(contextA), deferred<PlannerTactic>()],
    ]);
    vi.mocked(invokeCommand).mockImplementation((command, args) => {
      if (command !== "save_planner_tactic") {
        throw new Error(`Unexpected command ${command}`);
      }
      const request = args as PlannerContext & { tactic: PlannerTactic };
      const pending = pendingByContext.get(contextKey(request));
      if (!pending) {
        throw new Error("No test-local response for context");
      }
      return pending.promise.then((saved) => {
        persisted.set(contextKey(request), saved);
        return saved;
      });
    });
    const options = resolvePlannerTacticOptionsIpcMock();
    const editor = (context: PlannerContext, tactic: PlannerTactic) => (
      <PlannerTacticEditor
        key={contextKey(context)}
        context={context}
        activeSaveRefreshError={false}
        isActiveSaveUnavailable={false}
        tactic={tactic}
        options={options}
      />
    );
    const { rerender } = render(provider(client, editor(contextA, tacticA)));

    const weightA = screen.getByRole("slider", {
      name: "IP/OOP score weight",
    });
    weightA.focus();
    await user.keyboard(
      "{ArrowRight}{ArrowRight}{ArrowRight}{ArrowRight}{ArrowRight}",
    );
    expect(weightA).toHaveValue("55");
    await user.click(screen.getByRole("button", { name: "Save tactic" }));

    rerender(provider(client, editor(contextB, tacticB)));
    expect(
      screen.getByRole("slider", { name: "IP/OOP score weight" }),
    ).toHaveValue("30");
    expect(screen.queryByText("Tactic saved.")).not.toBeInTheDocument();

    const savedA = resolvePlannerTacticIpcMock();
    savedA.lanes[0].ipWeight = 0.55;
    await act(async () => {
      pendingByContext.get(contextKey(contextA))?.resolve(savedA);
      await pendingByContext.get(contextKey(contextA))?.promise;
    });

    await waitFor(() =>
      expect(
        client.getQueryData<PlannerTactic>(plannerKeys.tactic(contextA))
          ?.lanes[0].ipWeight,
      ).toBe(0.55),
    );
    expect(
      client.getQueryData<PlannerTactic>(plannerKeys.tactic(contextB))?.lanes[0]
        .ipWeight,
    ).toBe(0.3);
    expect(persisted.get(contextKey(contextA))?.lanes[0].ipWeight).toBe(0.55);
    expect(persisted.get(contextKey(contextB))?.lanes[0].ipWeight).toBe(0.3);
    expect(
      screen.getByRole("slider", { name: "IP/OOP score weight" }),
    ).toHaveValue("30");
    expect(screen.queryByText("Tactic saved.")).not.toBeInTheDocument();
  });
});
