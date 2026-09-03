import { describe, expect, it, vi } from "vitest";
import { invokeCommand } from "@/lib/tauri-client";
import { fetchPlannerTactic } from "./fetch-planner-tactic";
import { fetchPlannerTacticOptions } from "./fetch-planner-tactic-options";
import { plannerKeys } from "./planner-keys";
import { plannerTacticOptionsQueryOptions } from "./planner-tactic-options-query-options";
import { plannerTacticQueryOptions } from "./planner-tactic-query-options";
import { savePlannerTactic } from "./save-planner-tactic";

vi.mock("@/lib/tauri-client", () => ({ invokeCommand: vi.fn() }));

const contextA = { saveId: 1, contextToken: "token-a" };
const contextB = { saveId: 2, contextToken: "token-b" };
const recreatedA = { saveId: 1, contextToken: "token-a-new" };
const tactic = { lanes: [] };

describe("Planner tactic captured-context API", () => {
  it("keys reads by both save ID and context token", () => {
    expect(plannerKeys.tactic(contextA)).toEqual([
      "planner",
      "tactic",
      contextA,
    ]);
    expect(plannerKeys.tactic(contextA)).not.toEqual(
      plannerKeys.tactic(contextB),
    );
    expect(plannerKeys.tactic(contextA)).not.toEqual(
      plannerKeys.tactic(recreatedA),
    );
    expect(plannerKeys.tacticOptions(contextA)).not.toEqual(
      plannerKeys.tacticOptions(contextB),
    );
    expect(plannerKeys.tacticOptions(contextA)).not.toEqual(
      plannerKeys.tacticOptions(recreatedA),
    );
    expect(plannerTacticQueryOptions(contextA).queryKey).toEqual(
      plannerKeys.tactic(contextA),
    );
    expect(plannerTacticOptionsQueryOptions(contextA).queryKey).toEqual(
      plannerKeys.tacticOptions(contextA),
    );
  });

  it("passes exact camelCase context arguments to every tactic command", async () => {
    const invoke = vi.mocked(invokeCommand);
    invoke.mockResolvedValue(tactic);

    await fetchPlannerTactic(contextA);
    await fetchPlannerTacticOptions(contextA);
    await savePlannerTactic(contextA, tactic);

    expect(invoke).toHaveBeenNthCalledWith(1, "get_planner_tactic", contextA);
    expect(invoke).toHaveBeenNthCalledWith(
      2,
      "get_planner_tactic_options",
      contextA,
    );
    expect(invoke).toHaveBeenNthCalledWith(3, "save_planner_tactic", {
      ...contextA,
      tactic,
    });
  });
});
