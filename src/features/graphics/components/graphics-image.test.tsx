import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { setGraphicsStatusIpcMock } from "../api/graphics-ipc-mock";
import { GraphicsImage } from "./graphics-image";

function renderImage() {
  return render(
    <QueryClientProvider client={new QueryClient()}>
      <GraphicsImage
        kind="personPortrait"
        uid={42}
        slot={{
          alt: "Player portrait",
          className: "portrait",
          fallback: <span>Initials</span>,
        }}
      />
    </QueryClientProvider>,
  );
}

describe("GraphicsImage", () => {
  it("uses the committed protocol URL and native loading hints", async () => {
    setGraphicsStatusIpcMock({
      generation: 7,
      selected: true,
      candidate: { available: false, source: "absent" },
      summary: {
        configs: 0,
        mappings: 0,
        truncated: false,
        diagnostics: {
          configLimit: 0,
          entryLimit: 0,
          depthLimit: 0,
          mappingLimit: 0,
          configTooLarge: 0,
          configUnreadable: 0,
          malformedConfig: 0,
          invalidMapping: 0,
          sourceUnreadable: 0,
        },
      },
    });
    renderImage();
    const image = await screen.findByRole("img", { name: "Player portrait" });
    expect(image).toHaveAttribute(
      "src",
      "http://graphics.localhost/7/personPortrait/42",
    );
    expect(image).toHaveAttribute("loading", "lazy");
    expect(image).toHaveAttribute("decoding", "async");
  });

  it("leaves the caller fallback after a native error", async () => {
    setGraphicsStatusIpcMock({
      generation: 7,
      selected: true,
      candidate: { available: false, source: "absent" },
      summary: {
        configs: 0,
        mappings: 0,
        truncated: false,
        diagnostics: {
          configLimit: 0,
          entryLimit: 0,
          depthLimit: 0,
          mappingLimit: 0,
          configTooLarge: 0,
          configUnreadable: 0,
          malformedConfig: 0,
          invalidMapping: 0,
          sourceUnreadable: 0,
        },
      },
    });
    renderImage();
    const image = await screen.findByRole("img", { name: "Player portrait" });
    fireEvent.error(image);
    expect(screen.getByText("Initials")).toBeVisible();
    expect(screen.queryByRole("img", { name: "Player portrait" })).toBeNull();
  });
});
