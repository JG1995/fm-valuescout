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
  it("groups Rust targets inside Senior sections and preserves the complete semantic payload", async () => {
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
    const senior = within(dialog).getByRole("group", { name: "Senior" });
    const coaching = within(senior).getByRole("group", { name: "Coaching" });
    const recruitment = within(senior).getByRole("group", {
      name: "Recruitment",
    });
    const medical = within(senior).getByRole("group", { name: "Medical" });

    expect(within(dialog).getByText("B Squad")).toBeInTheDocument();
    expect(within(dialog).queryByRole("group", { name: "Club" })).toBeNull();
    expect(
      within(coaching).getByRole("spinbutton", {
        name: "Head of Youth Development slots",
      }),
    ).toHaveAttribute("max", "1");
    expect(
      within(coaching).getByRole("spinbutton", {
        name: "Assistant Manager slots",
      }),
    ).toHaveAttribute("max", "50");
    expect(
      within(coaching).getByRole("spinbutton", { name: "Coaches slots" }),
    ).toBeInTheDocument();
    expect(
      within(coaching).getByRole("spinbutton", {
        name: "Set Piece Coach slots",
      }),
    ).toHaveAttribute("max", "1");
    for (const [headRole, ordinaryRole] of [
      ["Head Performance Analyst slots", "Performance Analyst slots"],
      ["Chief Scout slots", "Scout slots"],
      ["Head Physio slots", "Physio slots"],
      ["Head of Sports Science slots", "Sports Scientist slots"],
    ]) {
      const head = within(senior).getByRole("spinbutton", { name: headRole });
      const ordinary = within(senior).getByRole("spinbutton", {
        name: ordinaryRole,
      });
      expect(
        head.compareDocumentPosition(ordinary) &
          Node.DOCUMENT_POSITION_FOLLOWING,
      ).not.toBe(0);
    }
    expect(
      within(recruitment).getByRole("spinbutton", {
        name: "Recruitment Analyst slots",
      }),
    ).toHaveAttribute("max", "50");
    expect(
      within(medical).getByRole("spinbutton", { name: "Head Physio slots" }),
    ).toHaveAttribute("max", "1");
    expect(
      within(senior).queryByRole("spinbutton", { name: "Manager slots" }),
    ).toBeNull();
    const reserves = within(dialog).getByRole("group", { name: "B Squad" });
    expect(reserves).toBeInTheDocument();
    expect(
      within(reserves).queryByRole("spinbutton", {
        name: "Set Piece Coach slots",
      }),
    ).toBeNull();
    expect(
      within(dialog).queryByRole("spinbutton", { name: "Doctor slots" }),
    ).toBeNull();
    expect(
      within(dialog).queryByRole("spinbutton", {
        name: "Chief Doctor slots",
      }),
    ).toBeNull();
    expect(
      within(dialog).getAllByRole("spinbutton", {
        name: "Head of Youth Development slots",
      }),
    ).toHaveLength(1);
    expect(within(dialog).getAllByRole("spinbutton")).toHaveLength(28);

    const headOfYouthDevelopment = within(coaching).getByRole("spinbutton", {
      name: "Head of Youth Development slots",
    });
    const coaches = within(coaching).getByRole("spinbutton", {
      name: "Coaches slots",
    });
    await user.clear(headOfYouthDevelopment);
    await user.type(headOfYouthDevelopment, "1");
    await user.clear(coaches);
    await user.type(coaches, "50");
    await user.click(
      within(dialog).getByRole("button", { name: "Save slots" }),
    );

    await waitFor(() => {
      const request = getLastStaffAssignmentTargetsIpcArgs() as {
        targets?: { scope: string; jobId: string; slotCount: number }[];
      };
      expect(request).toEqual({
        expectedSaveContextToken: "save-token-a",
        targets: expect.arrayContaining([
          {
            scope: "club",
            jobId: "head_of_youth_development",
            slotCount: 1,
          },
          { scope: "senior", jobId: "coaches", slotCount: 50 },
        ]),
      });
      expect(request.targets).toHaveLength(28);
      expect(
        new Set(
          request.targets?.map((target) => `${target.scope}:${target.jobId}`),
        ).size,
      ).toBe(28);
    });
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Slot counts saved.",
    );
    await waitFor(() => expect(trigger).toHaveFocus());
  });

  it("renders Club sections once when Senior is disabled", async () => {
    const user = userEvent.setup();
    const targets = fixtureStaffAssignmentTargets();
    targets.teams = targets.teams.filter(({ team }) => team !== "senior");
    targets.targets = targets.targets.filter(({ scope }) => scope !== "senior");
    setStaffAssignmentTargetsIpcMock(targets);
    renderModal();

    await user.click(
      await screen.findByRole("button", { name: "Configure slots" }),
    );
    const dialog = await screen.findByRole("dialog");
    const club = within(dialog).getByRole("group", { name: "Club" });

    expect(within(dialog).queryByRole("group", { name: "Senior" })).toBeNull();
    expect(
      within(club).getByRole("group", { name: "Coaching" }),
    ).toBeInTheDocument();
    expect(
      within(club).getByRole("group", { name: "Recruitment" }),
    ).toBeInTheDocument();
    expect(
      within(club).getByRole("group", { name: "Medical" }),
    ).toBeInTheDocument();
    expect(
      within(club).getAllByRole("spinbutton", {
        name: "Head of Youth Development slots",
      }),
    ).toHaveLength(1);
  });

  it("uses each Rust maximum for local validation and shows a Rust save error", async () => {
    const user = userEvent.setup();
    renderModal();

    await user.click(
      await screen.findByRole("button", { name: "Configure slots" }),
    );
    const dialog = await screen.findByRole("dialog");
    const headOfYouthDevelopment = within(dialog).getByRole("spinbutton", {
      name: "Head of Youth Development slots",
    });
    await user.clear(headOfYouthDevelopment);
    await user.type(headOfYouthDevelopment, "2");

    expect(
      within(dialog).getByText("Enter a whole number from 0 to 1."),
    ).toBeInTheDocument();
    expect(
      within(dialog).getByRole("button", { name: "Save slots" }),
    ).toBeDisabled();

    await user.clear(headOfYouthDevelopment);
    await user.type(headOfYouthDevelopment, "1");
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
