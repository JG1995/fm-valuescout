import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { StrictMode, useState } from "react";
import { describe, expect, it } from "vitest";
import { Modal } from "./modal";

function DialogHarness({ conditional }: { conditional: boolean }) {
  const [open, setOpen] = useState(false);
  const [opened, setOpened] = useState(false);
  return (
    <>
      <button
        type="button"
        onClick={() => {
          setOpened(true);
          setOpen(true);
        }}
      >
        Open dialog
      </button>
      {opened && (!conditional || open) && (
        <Modal
          open={open}
          title="Edit selection"
          onClose={() => setOpen(false)}
        >
          <input aria-label="Selection" />
        </Modal>
      )}
    </>
  );
}

describe("Modal focus lifecycle", () => {
  it.each([false, true])(
    "returns focus after dismissal under StrictMode (conditional: %s)",
    async (conditional) => {
      const user = userEvent.setup();
      render(
        <StrictMode>
          <DialogHarness conditional={conditional} />
        </StrictMode>,
      );
      const trigger = screen.getByRole("button", { name: "Open dialog" });
      await user.click(trigger);
      expect(screen.getByRole("textbox", { name: "Selection" })).toHaveFocus();
      await user.keyboard("{Escape}");
      await waitFor(() =>
        expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
      );
      await waitFor(() => expect(trigger).toHaveFocus());
    },
  );
});

function HandoffHarness() {
  const [step, setStep] = useState<"closed" | "first" | "second">("closed");
  return (
    <>
      <button type="button" onClick={() => setStep("first")}>
        Open first
      </button>
      <Modal
        open={step === "first"}
        title="First"
        onClose={() => setStep("closed")}
      >
        <button type="button" onClick={() => setStep("second")}>
          Continue
        </button>
      </Modal>
      <Modal
        open={step === "second"}
        title="Second"
        onClose={() => setStep("closed")}
      >
        <input aria-label="Second field" />
      </Modal>
    </>
  );
}

describe("Modal handoff", () => {
  it("leaves focus in a dialog that opened as it closed", async () => {
    const user = userEvent.setup();
    render(<HandoffHarness />);

    await user.click(screen.getByRole("button", { name: "Open first" }));
    await user.click(await screen.findByRole("button", { name: "Continue" }));
    const field = await screen.findByRole("textbox", { name: "Second field" });
    await waitFor(() =>
      expect(screen.queryByRole("dialog", { name: "First" })).toBeNull(),
    );

    expect(field).toHaveFocus();
  });
});
