import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { mockIPC } from "@tauri-apps/api/mocks";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";
import type { AcademyClass } from "../types/academy";
import { AcademyClassCreationModal } from "./academy-class-creation-modal";

it("locks the class draft and dismissal until creation settles", async () => {
  const user = userEvent.setup();
  const created: AcademyClass = {
    id: 8,
    classYear: 2026,
    isAutomatic: false,
    memberCount: 0,
  };
  let settle: (() => void) | undefined;
  const pending = new Promise<AcademyClass>((resolve) => {
    settle = () => resolve(created);
  });
  const requests: unknown[] = [];
  mockIPC((cmd, args) => {
    if (cmd !== "create_academy_class")
      throw new Error(`Unexpected IPC: ${cmd}`);
    requests.push(args);
    return pending;
  });
  const onClose = vi.fn();
  const onCreated = vi.fn();
  render(
    <QueryClientProvider client={new QueryClient()}>
      <AcademyClassCreationModal
        open
        prefillYear={2026}
        onClose={onClose}
        onCreated={onCreated}
      />
    </QueryClientProvider>,
  );
  const year = screen.getByRole("spinbutton", { name: "Class year" });
  await user.click(screen.getByRole("button", { name: "Create class" }));
  await waitFor(() => expect(requests).toHaveLength(1));
  expect(year).toBeDisabled();
  await user.keyboard("{Escape}");
  await user.click(screen.getByRole("button", { name: "Close dialog" }));
  expect(onClose).not.toHaveBeenCalled();
  expect(requests).toHaveLength(1);
  if (!settle) throw new Error("Expected pending class creation");
  settle();
  await waitFor(() => expect(onCreated).toHaveBeenCalledWith(created));
  expect(year).toBeEnabled();
  expect(year).toHaveValue(2026);
  await user.click(screen.getByRole("button", { name: "Cancel" }));
  expect(onClose).toHaveBeenCalledOnce();
});
