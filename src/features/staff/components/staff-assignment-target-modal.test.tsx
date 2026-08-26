import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import {
  fixtureStaffAssignmentTargets,
  getLastStaffAssignmentTargetsIpcArgs,
  setStaffAssignmentTargetsIpcMock,
  setStaffAssignmentTargetsIpcMockMode,
} from "@/testing/staff-ipc-mock";
import { StaffAssignmentTargetModal } from "./staff-assignment-target-modal";

const contextA = {
  saveId: 1,
  saveContextToken: "save-token-a",
  snapshotId: 1,
  snapshotContextToken: "snapshot-token-a",
};
const contextB = {
  ...contextA,
  snapshotContextToken: "snapshot-token-b",
};

function renderModal(context = contextA, contextKey = "context-a") {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  const result = render(
    <QueryClientProvider client={queryClient}>
      <StaffAssignmentTargetModal context={context} contextKey={contextKey} />
    </QueryClientProvider>,
  );

  return {
    ...result,
    rerenderModal(nextContext = contextB, nextContextKey = "context-b") {
      result.rerender(
        <QueryClientProvider client={queryClient}>
          <StaffAssignmentTargetModal
            context={nextContext}
            contextKey={nextContextKey}
          />
        </QueryClientProvider>,
      );
    },
  };
}

describe("StaffAssignmentTargetModal", () => {
  it("renders Rust-provided groups and submits every allowed pair", async () => {
    const user = userEvent.setup();
    const targets = fixtureStaffAssignmentTargets();
    targets.teams[1] = { ...targets.teams[1], displayName: "B Squad" };
    setStaffAssignmentTargetsIpcMock(targets);
    renderModal();

    const trigger = await screen.findByRole("button", {
      name: "Configure slots",
    });
    await user.click(trigger);
    const dialog = await screen.findByRole("dialog", {
      name: "Configure assignment slots",
    });

    expect(within(dialog).getByText("B Squad")).toBeInTheDocument();
    expect(within(dialog).getByText("Club")).toBeInTheDocument();
    expect(
      within(dialog).queryAllByRole("spinbutton", { name: "Manager slots" }),
    ).toHaveLength(2);
    expect(within(dialog).getAllByRole("spinbutton")).toHaveLength(35);

    const coaches = within(dialog).getAllByRole("spinbutton", {
      name: "Coaches slots",
    });
    await user.clear(coaches[0]);
    await user.type(coaches[0], "50");
    await user.click(
      within(dialog).getByRole("button", { name: "Save slots" }),
    );

    await waitFor(() =>
      expect(getLastStaffAssignmentTargetsIpcArgs()).toEqual({
        expectedSaveContextToken: "save-token-a",
        targets: expect.arrayContaining([
          { scope: "senior", jobId: "coaches", slotCount: 50 },
        ]),
      }),
    );
    expect(
      (
        getLastStaffAssignmentTargetsIpcArgs() as
          | { targets?: unknown[] }
          | undefined
      )?.targets,
    ).toHaveLength(35);
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Slot counts saved.",
    );
    await waitFor(() => expect(trigger).toHaveFocus());
  });

  it("keeps invalid counts local and shows a Rust save error", async () => {
    const user = userEvent.setup();
    renderModal();

    await user.click(
      await screen.findByRole("button", { name: "Configure slots" }),
    );
    const dialog = await screen.findByRole("dialog");
    const assistantManager = within(dialog).getAllByRole("spinbutton", {
      name: "Assistant Manager slots",
    })[0];
    await user.clear(assistantManager);
    await user.type(assistantManager, "51");

    expect(
      within(dialog).getByText("Enter a whole number from 0 to 50."),
    ).toBeInTheDocument();
    expect(
      within(dialog).getByRole("button", { name: "Save slots" }),
    ).toBeDisabled();

    await user.clear(assistantManager);
    await user.type(assistantManager, "0");
    setStaffAssignmentTargetsIpcMockMode("error");
    await user.click(
      within(dialog).getByRole("button", { name: "Save slots" }),
    );
    expect(await within(dialog).findByRole("alert")).toHaveTextContent(
      "Could not save assignment targets",
    );
  });

  it("locks the form while a save is pending", async () => {
    const user = userEvent.setup();
    setStaffAssignmentTargetsIpcMockMode("pending");
    renderModal();

    await user.click(
      await screen.findByRole("button", { name: "Configure slots" }),
    );
    const dialog = await screen.findByRole("dialog");
    await user.click(
      within(dialog).getByRole("button", { name: "Save slots" }),
    );

    expect(
      within(dialog).getByRole("button", { name: "Saving…" }),
    ).toBeDisabled();
    expect(
      within(dialog).getByRole("button", { name: "Cancel" }),
    ).toBeDisabled();
    expect(within(dialog).getAllByRole("spinbutton")[0]).toBeDisabled();
  });

  it("closes and discards a draft when the immutable context changes", async () => {
    const user = userEvent.setup();
    const { rerenderModal } = renderModal();

    await user.click(
      await screen.findByRole("button", { name: "Configure slots" }),
    );
    const dialog = await screen.findByRole("dialog");
    const assistantManager = within(dialog).getAllByRole("spinbutton", {
      name: "Assistant Manager slots",
    })[0];
    await user.clear(assistantManager);
    await user.type(assistantManager, "12");

    rerenderModal();
    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
    await user.click(
      await screen.findByRole("button", { name: "Configure slots" }),
    );
    expect(
      screen.getAllByRole("spinbutton", { name: "Assistant Manager slots" })[0],
    ).toHaveValue(0);
  });
});
