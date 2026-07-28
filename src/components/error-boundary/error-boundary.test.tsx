import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { ErrorBoundary } from "./error-boundary";

function ThrowingChild({ shouldThrow }: { shouldThrow: boolean }) {
  if (shouldThrow) {
    throw new Error("boom");
  }

  return <p>Healthy child</p>;
}

describe("error boundary", () => {
  it("renders fallback UI when a child throws", () => {
    const consoleError = vi
      .spyOn(console, "error")
      .mockImplementation(() => {});

    const { rerender } = render(
      <ErrorBoundary
        fallback={({ error, reset }) => (
          <div>
            <p>{error.message}</p>
            <button type="button" onClick={reset}>
              Retry
            </button>
          </div>
        )}
      >
        <ThrowingChild shouldThrow={false} />
      </ErrorBoundary>,
    );

    expect(screen.getByText("Healthy child")).toBeInTheDocument();

    rerender(
      <ErrorBoundary
        fallback={({ error, reset }) => (
          <div>
            <p>{error.message}</p>
            <button type="button" onClick={reset}>
              Retry
            </button>
          </div>
        )}
      >
        <ThrowingChild shouldThrow={true} />
      </ErrorBoundary>,
    );

    expect(screen.getByText("boom")).toBeInTheDocument();

    consoleError.mockRestore();
  });

  it("renders children again after reset when the error is resolved", async () => {
    const user = userEvent.setup();
    const consoleError = vi
      .spyOn(console, "error")
      .mockImplementation(() => {});

    const boundary = ({
      error,
      reset,
    }: {
      error: Error;
      reset: () => void;
    }) => (
      <div>
        <p>{error.message}</p>
        <button type="button" onClick={reset}>
          Retry
        </button>
      </div>
    );

    const { rerender } = render(
      <ErrorBoundary fallback={boundary}>
        <ThrowingChild shouldThrow={true} />
      </ErrorBoundary>,
    );

    expect(screen.getByText("boom")).toBeInTheDocument();

    rerender(
      <ErrorBoundary fallback={boundary}>
        <ThrowingChild shouldThrow={false} />
      </ErrorBoundary>,
    );

    expect(screen.getByText("boom")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Retry" }));

    expect(screen.getByText("Healthy child")).toBeInTheDocument();

    consoleError.mockRestore();
  });
});
