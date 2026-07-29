import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { renderWithProviders } from "@/testing/render-with-providers";
import {
  resolveBusyLoadDataRequest,
  setLoadDataIpcMockMode,
} from "@/testing/snapshot-ipc-mock";

describe("snapshot panels", () => {
  beforeEach(() => {
    setLoadDataIpcMockMode("success");
  });

  afterEach(() => {
    resolveBusyLoadDataRequest();
  });

  it("shows empty snapshot guidance on open", async () => {
    renderWithProviders();

    expect(await screen.findByText(/^Snapshot$/i)).toBeInTheDocument();
    expect(
      screen.getByText(/No snapshot loaded for the active save/i),
    ).toBeInTheDocument();
    expect(screen.getByText(/^Saves$/i)).toBeInTheDocument();
    expect(screen.getByRole("combobox", { name: "Active save" })).toHaveValue(
      "1",
    );
  });

  it("shows snapshot metadata and sanity list after Load Data", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    await screen.findByText(/No snapshot loaded for the active save/i);
    await user.click(await screen.findByRole("button", { name: "Load Data" }));

    expect(
      await screen.findByText(/Loaded 3 players into the database/i),
    ).toBeInTheDocument();
    expect(screen.getByText(/In database:/i)).toBeInTheDocument();
    expect(screen.getByText("Alex Morgan")).toBeInTheDocument();
    expect(screen.getByText("165")).toBeInTheDocument();
    expect(screen.getByText("Metro FC")).toBeInTheDocument();
  });

  it("shows truncated banner after capped Load Data", async () => {
    setLoadDataIpcMockMode("truncatedSuccess");
    const user = userEvent.setup();
    renderWithProviders();

    await screen.findByText(/No snapshot loaded/i);
    await user.click(await screen.findByRole("button", { name: "Load Data" }));

    expect(
      await screen.findByText(
        /Incomplete snapshot: scan was capped at 500 players/i,
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Loaded 500 players into the database/i),
    ).toBeInTheDocument();
  });

  it("creates and switches saves", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    await screen.findByRole("combobox", { name: "Active save" });
    await user.type(screen.getByLabelText("New save"), "Youth intake");
    await user.click(screen.getByRole("button", { name: "Create save" }));

    const select = await screen.findByRole("combobox", { name: "Active save" });
    expect(select).toHaveValue("1");
    expect(
      screen.getByRole("option", { name: "Youth intake" }),
    ).toBeInTheDocument();
    await user.selectOptions(select, "2");
    expect(select).toHaveValue("2");
  });

  it("clears and restores snapshot overview when switching saves", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    await screen.findByText(/No snapshot loaded for the active save/i);
    await user.click(await screen.findByRole("button", { name: "Load Data" }));
    expect(await screen.findByText("Alex Morgan")).toBeInTheDocument();

    await user.type(screen.getByLabelText("New save"), "Youth intake");
    await user.click(screen.getByRole("button", { name: "Create save" }));

    const select = await screen.findByRole("combobox", { name: "Active save" });
    await user.selectOptions(select, "2");

    expect(
      await screen.findByText(/No snapshot loaded for the active save/i),
    ).toBeInTheDocument();
    expect(screen.queryByText("Alex Morgan")).not.toBeInTheDocument();
    expect(
      screen.queryByText(/Loaded 3 players into the database/i),
    ).not.toBeInTheDocument();

    await user.selectOptions(select, "1");
    expect(await screen.findByText("Alex Morgan")).toBeInTheDocument();
  });

  it("retargets the rename field when the top bar switches save", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    await user.type(await screen.findByLabelText("New save"), "Youth intake");
    await user.click(screen.getByRole("button", { name: "Create save" }));

    const select = await screen.findByRole("combobox", { name: "Active save" });
    await user.selectOptions(select, "2");

    // A draft left over from the previous save would rename the new one to the
    // old name on the next submit.
    expect(await screen.findByLabelText("Rename active save")).toHaveValue(
      "Youth intake",
    );
  });

  it("renames the active save", async () => {
    const user = userEvent.setup();
    renderWithProviders();

    const renameInput = await screen.findByLabelText("Rename active save");
    await user.clear(renameInput);
    await user.type(renameInput, "Main career");
    await user.click(screen.getByRole("button", { name: "Rename save" }));

    expect(
      await screen.findByRole("combobox", { name: "Active save" }),
    ).toHaveDisplayValue("Main career");
  });
});
