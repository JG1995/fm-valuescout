import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ComponentProps } from "react";
import { describe, expect, it, vi } from "vitest";
import {
  getLastClubDnaRemoveIpcArgs,
  getLastClubDnaSetIpcArgs,
  rejectBusyClubDnaGetRequest,
  rejectBusyClubDnaRemoveRequest,
  rejectBusyClubDnaSetRequest,
  resolveBusyClubDnaGetRequest,
  resolveBusyClubDnaRemoveRequest,
  resolveBusyClubDnaSetRequest,
  setClubDnaGetIpcMockMode,
  setClubDnaIpcMockDefinition,
  setClubDnaRemoveIpcMockMode,
  setClubDnaSetIpcMockMode,
} from "@/testing/club-dna-ipc-mock";
import { ClubDnaDefinition } from "./club-dna-definition";

const contextA = { saveId: 1, contextToken: "save-token-a" };
const contextB = { saveId: 2, contextToken: "save-token-b" };

type DefinitionProps = ComponentProps<typeof ClubDnaDefinition>;

function renderDefinition(overrides: Partial<DefinitionProps> = {}) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  const onSaved = vi.fn();
  const onRemoved = vi.fn();
  const props: DefinitionProps = {
    context: contextA,
    available: true,
    onSaved,
    onRemoved,
    ...overrides,
  };
  const result = render(
    <QueryClientProvider client={queryClient}>
      <ClubDnaDefinition {...props} />
    </QueryClientProvider>,
  );

  return {
    ...result,
    onSaved,
    onRemoved,
    rerenderDefinition(nextProps: Partial<DefinitionProps>) {
      Object.assign(props, nextProps);
      result.rerender(
        <QueryClientProvider client={queryClient}>
          <ClubDnaDefinition {...props} />
        </QueryClientProvider>,
      );
    },
  };
}

async function reopenDefinitionFor(
  user: ReturnType<typeof userEvent.setup>,
  rerenderDefinition: (nextProps: Partial<DefinitionProps>) => void,
  context: typeof contextA,
) {
  rerenderDefinition({ context });
  await waitFor(() =>
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
  );
  await user.click(screen.getByRole("button", { name: "Define DNA" }));
}

function expectOnlyBDefinition() {
  expect(screen.getByRole("checkbox", { name: "Pace" })).toBeChecked();
  expect(
    screen.getByRole("checkbox", { name: "Acceleration" }),
  ).not.toBeChecked();
  expect(screen.queryByRole("alert")).not.toBeInTheDocument();
}

describe("ClubDnaDefinition", () => {
  it("selects the full catalog and saves its ordered IDs with the supplied context", async () => {
    const user = userEvent.setup();
    const { onSaved } = renderDefinition();

    await user.click(screen.getByRole("button", { name: "Define DNA" }));
    expect(
      await screen.findByRole("dialog", { name: "Define Club DNA" }),
    ).toHaveTextContent(
      "Club DNA scales each selected 1–20 value by 5, gives every selected attribute equal weight, averages the values, and rounds to a whole 0–100 score.",
    );
    expect(
      screen.getByRole("button", { name: "Save Club DNA" }),
    ).toBeDisabled();

    const checkboxes = screen.getAllByRole("checkbox");
    expect(checkboxes).toHaveLength(60);
    for (const checkbox of checkboxes) {
      await user.click(checkbox);
    }
    expect(screen.getByText("Selected attributes (60)")).toBeInTheDocument();
    const selectedAttributes = screen.getByRole("list");
    expect(selectedAttributes).toHaveTextContent("Acceleration");
    expect(selectedAttributes).toHaveTextContent("Professionalism");

    await user.click(screen.getByRole("button", { name: "Save Club DNA" }));
    await waitFor(() => {
      expect(getLastClubDnaSetIpcArgs()).toEqual(
        expect.objectContaining({
          ...contextA,
          attributeIds: expect.arrayContaining([
            "attr.Acceleration",
            "attr.AerialReach",
            "hidden.Consistency",
            "personality.Professionalism",
          ]),
        }),
      );
      expect(getLastClubDnaSetIpcArgs()?.attributeIds).toHaveLength(60);
    });
    expect(onSaved).toHaveBeenCalledWith(
      expect.objectContaining({ created: true }),
      contextA,
    );
  });

  it("starts an edit from the persisted definition and discards a cancelled draft", async () => {
    const user = userEvent.setup();
    setClubDnaIpcMockDefinition(contextA, ["attr.Acceleration"]);
    renderDefinition();

    await user.click(screen.getByRole("button", { name: "Define DNA" }));
    expect(
      await screen.findByRole("checkbox", { name: "Acceleration" }),
    ).toBeChecked();
    await user.click(screen.getByRole("checkbox", { name: "Pace" }));
    await user.click(screen.getByRole("button", { name: "Cancel" }));

    await user.click(screen.getByRole("button", { name: "Define DNA" }));
    expect(
      await screen.findByRole("checkbox", { name: "Acceleration" }),
    ).toBeChecked();
    expect(screen.getByRole("checkbox", { name: "Pace" })).not.toBeChecked();
    await user.keyboard("{Escape}");
    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
  });

  it("saves a persisted edit with ordered IDs and reports created false", async () => {
    const user = userEvent.setup();
    setClubDnaIpcMockDefinition(contextA, ["attr.Acceleration"]);
    const { onSaved } = renderDefinition();

    await user.click(screen.getByRole("button", { name: "Define DNA" }));
    await screen.findByRole("checkbox", { name: "Acceleration" });
    await user.click(screen.getByRole("checkbox", { name: "Acceleration" }));
    await user.click(screen.getByRole("checkbox", { name: "Pace" }));
    await user.click(screen.getByRole("button", { name: "Save Club DNA" }));

    await waitFor(() =>
      expect(getLastClubDnaSetIpcArgs()).toEqual({
        ...contextA,
        attributeIds: ["attr.Pace"],
      }),
    );
    expect(onSaved).toHaveBeenCalledWith(
      {
        definition: { attributeIds: ["attr.Pace"] },
        created: false,
      },
      contextA,
    );
  });

  it("keeps a set failure in the edit form and allows retrying it", async () => {
    const user = userEvent.setup();
    setClubDnaSetIpcMockMode("error");
    renderDefinition();

    await user.click(screen.getByRole("button", { name: "Define DNA" }));
    await user.click(
      await screen.findByRole("checkbox", { name: "Acceleration" }),
    );
    await user.click(screen.getByRole("button", { name: "Save Club DNA" }));
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Could not save Club DNA",
    );
    expect(
      screen.getByRole("dialog", { name: "Define Club DNA" }),
    ).toBeInTheDocument();

    setClubDnaSetIpcMockMode("success");
    await user.click(screen.getByRole("checkbox", { name: "Pace" }));
    await user.click(screen.getByRole("button", { name: "Save Club DNA" }));
    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
  });

  it("disables saving while the context definition cannot load", async () => {
    const user = userEvent.setup();
    setClubDnaGetIpcMockMode("error");
    renderDefinition();

    await user.click(screen.getByRole("button", { name: "Define DNA" }));
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Could not load Club DNA",
    );
    expect(
      screen.getByRole("button", { name: "Save Club DNA" }),
    ).toBeDisabled();
  });

  it("preserves the edit draft through one destructive confirmation and returns focus to save", async () => {
    const user = userEvent.setup();
    setClubDnaIpcMockDefinition(contextA, ["attr.Acceleration"]);
    renderDefinition();

    await user.click(screen.getByRole("button", { name: "Define DNA" }));
    await screen.findByRole("checkbox", { name: "Acceleration" });
    await user.click(screen.getByRole("checkbox", { name: "Pace" }));
    await user.click(screen.getByRole("button", { name: "Remove Club DNA" }));
    expect(
      screen.getByRole("dialog", { name: "Remove Club DNA?" }),
    ).toBeInTheDocument();

    await user.keyboard("{Escape}");
    expect(screen.getByRole("checkbox", { name: "Pace" })).toBeChecked();
    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: "Save Club DNA" }),
      ).toHaveFocus();
    });
  });

  it("disables removal when availability changes during confirmation", async () => {
    const user = userEvent.setup();
    setClubDnaIpcMockDefinition(contextA, ["attr.Acceleration"]);
    const { rerenderDefinition } = renderDefinition();

    await user.click(screen.getByRole("button", { name: "Define DNA" }));
    await screen.findByRole("checkbox", { name: "Acceleration" });
    await user.click(screen.getByRole("button", { name: "Remove Club DNA" }));
    rerenderDefinition({ available: false });

    const remove = screen.getByRole("button", { name: "Remove definition" });
    expect(remove).toBeDisabled();
    await user.click(remove);
    expect(getLastClubDnaRemoveIpcArgs()).toBeUndefined();
  });

  it("keeps removal pending, retains its error in confirmation, and returns focus to the trigger after success", async () => {
    const user = userEvent.setup();
    setClubDnaIpcMockDefinition(contextA, ["attr.Acceleration"]);
    setClubDnaRemoveIpcMockMode("busy");
    const { onRemoved } = renderDefinition();

    const trigger = screen.getByRole("button", { name: "Define DNA" });
    await user.click(trigger);
    await screen.findByRole("checkbox", { name: "Acceleration" });
    await user.click(screen.getByRole("button", { name: "Remove Club DNA" }));
    await user.click(screen.getByRole("button", { name: "Remove definition" }));
    await user.keyboard("{Escape}");

    expect(
      screen.getByRole("dialog", { name: "Remove Club DNA?" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Cancel" })).toBeDisabled();
    expect(getLastClubDnaRemoveIpcArgs()).toEqual(contextA);

    rejectBusyClubDnaRemoveRequest(contextA);
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Could not remove Club DNA",
    );
    await user.click(screen.getByRole("button", { name: "Cancel" }));
    expect(
      screen.getByRole("button", { name: "Save Club DNA" }),
    ).toBeInTheDocument();

    setClubDnaRemoveIpcMockMode("success");
    await user.click(screen.getByRole("button", { name: "Remove Club DNA" }));
    await user.click(screen.getByRole("button", { name: "Remove definition" }));
    await waitFor(() =>
      expect(onRemoved).toHaveBeenCalledWith({ removed: true }, contextA),
    );
    await waitFor(() => expect(trigger).toHaveFocus());
  });

  it.each(["success", "error"] as const)(
    "keeps B data isolated when a late A get %s completes",
    async (outcome) => {
      const user = userEvent.setup();
      setClubDnaGetIpcMockMode("busy");
      const { rerenderDefinition } = renderDefinition();

      await user.click(screen.getByRole("button", { name: "Define DNA" }));
      rerenderDefinition({ context: contextB });
      await waitFor(() =>
        expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
      );
      await user.click(screen.getByRole("button", { name: "Define DNA" }));

      if (outcome === "success") {
        resolveBusyClubDnaGetRequest(contextA, {
          attributeIds: ["attr.Acceleration"],
        });
      } else {
        rejectBusyClubDnaGetRequest(contextA);
      }
      resolveBusyClubDnaGetRequest(contextB, { attributeIds: ["attr.Pace"] });

      await waitFor(expectOnlyBDefinition);
    },
  );

  it.each(["success", "error"] as const)(
    "suppresses a late A set %s after reopening B",
    async (outcome) => {
      const user = userEvent.setup();
      setClubDnaIpcMockDefinition(contextB, ["attr.Pace"]);
      setClubDnaSetIpcMockMode("busy");
      const { onSaved, rerenderDefinition } = renderDefinition();

      await user.click(screen.getByRole("button", { name: "Define DNA" }));
      await user.click(
        await screen.findByRole("checkbox", { name: "Acceleration" }),
      );
      await user.click(screen.getByRole("button", { name: "Save Club DNA" }));
      await waitFor(() => expect(getLastClubDnaSetIpcArgs()).toBeDefined());

      await reopenDefinitionFor(user, rerenderDefinition, contextB);
      await screen.findByRole("checkbox", { name: "Pace" });

      if (outcome === "success") {
        resolveBusyClubDnaSetRequest(contextA, {
          definition: { attributeIds: ["attr.Acceleration"] },
          created: true,
        });
      } else {
        rejectBusyClubDnaSetRequest(contextA);
      }

      await waitFor(expectOnlyBDefinition);
      expect(onSaved).not.toHaveBeenCalled();
    },
  );

  it.each(["success", "error"] as const)(
    "suppresses a late A remove %s after reopening B",
    async (outcome) => {
      const user = userEvent.setup();
      setClubDnaIpcMockDefinition(contextA, ["attr.Acceleration"]);
      setClubDnaIpcMockDefinition(contextB, ["attr.Pace"]);
      setClubDnaRemoveIpcMockMode("busy");
      const { onRemoved, rerenderDefinition } = renderDefinition();

      await user.click(screen.getByRole("button", { name: "Define DNA" }));
      await screen.findByRole("checkbox", { name: "Acceleration" });
      await user.click(screen.getByRole("button", { name: "Remove Club DNA" }));
      await user.click(
        screen.getByRole("button", { name: "Remove definition" }),
      );
      await waitFor(() =>
        expect(getLastClubDnaRemoveIpcArgs()).toEqual(contextA),
      );

      await reopenDefinitionFor(user, rerenderDefinition, contextB);
      await screen.findByRole("checkbox", { name: "Pace" });

      if (outcome === "success") {
        resolveBusyClubDnaRemoveRequest(contextA);
      } else {
        rejectBusyClubDnaRemoveRequest(contextA);
      }

      await waitFor(expectOnlyBDefinition);
      await user.click(screen.getByRole("button", { name: "Remove Club DNA" }));
      expect(screen.queryByRole("alert")).not.toBeInTheDocument();
      expect(onRemoved).not.toHaveBeenCalled();
    },
  );
});
