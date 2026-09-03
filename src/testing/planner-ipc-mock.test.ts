import { describe, expect, it } from "vitest";
import {
  resolvePlannerTacticIpcMock,
  resolvePlannerTacticOptionsIpcMock,
  resolveSavePlannerTacticIpcMock,
} from "@/testing/planner-ipc-mock";
import {
  resolveCreateSaveIpcMock,
  resolveDeleteSaveIpcMock,
} from "@/testing/snapshot-ipc-mock";

function expectPlannerError(operation: () => unknown, message: string) {
  let error: unknown;
  try {
    operation();
  } catch (caught) {
    error = caught;
  }
  expect(error instanceof Error ? error.message : error).toBe(message);
}

describe("Planner IPC mock save lifecycle", () => {
  it("accepts a created save context only after the save exists", () => {
    const context = { saveId: 2, contextToken: "save-token-2" };
    expectPlannerError(
      () => resolvePlannerTacticIpcMock(context),
      "Save 2 not found",
    );

    const created = resolveCreateSaveIpcMock({ name: "Created save" });
    expect(created).toMatchObject({
      id: context.saveId,
      contextToken: context.contextToken,
    });
    expect(resolvePlannerTacticOptionsIpcMock(context).placements).toContain(
      "GK",
    );

    const tactic = { lanes: [] };
    expect(resolveSavePlannerTacticIpcMock({ ...context, tactic })).toEqual(
      tactic,
    );
    expect(resolvePlannerTacticIpcMock(context)).toEqual(tactic);
  });

  it("rejects every Planner tactic command after its save is deleted", () => {
    const created = resolveCreateSaveIpcMock({ name: "Deleted save" });
    const context = {
      saveId: created.id,
      contextToken: created.contextToken,
    };
    resolveDeleteSaveIpcMock(context);

    for (const operation of [
      () => resolvePlannerTacticIpcMock(context),
      () => resolvePlannerTacticOptionsIpcMock(context),
      () =>
        resolveSavePlannerTacticIpcMock({
          ...context,
          tactic: { lanes: [] },
        }),
    ]) {
      expectPlannerError(operation, `Save ${created.id} not found`);
    }
  });

  it("rejects an old token and accepts the same-ID replacement token", () => {
    const oldContext = { saveId: 1, contextToken: "save-token-1" };
    resolveSavePlannerTacticIpcMock({
      ...oldContext,
      tactic: { lanes: [] },
    });

    const { activeSave: replacement } = resolveDeleteSaveIpcMock(oldContext);
    expect(replacement.id).toBe(oldContext.saveId);
    expect(replacement.contextToken).not.toBe(oldContext.contextToken);
    expectPlannerError(
      () => resolvePlannerTacticIpcMock(oldContext),
      "Save changed or no longer exists",
    );

    const replacementContext = {
      saveId: replacement.id,
      contextToken: replacement.contextToken,
    };
    expect(
      resolvePlannerTacticOptionsIpcMock(replacementContext).placements,
    ).toContain("GK");
    expect(
      resolvePlannerTacticIpcMock(replacementContext).lanes,
    ).not.toHaveLength(0);
    expect(
      resolveSavePlannerTacticIpcMock({
        ...replacementContext,
        tactic: { lanes: [] },
      }),
    ).toEqual({ lanes: [] });
  });
});
