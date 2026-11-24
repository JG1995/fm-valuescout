import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import {
  DEFAULT_GRAPHICS_STATUS,
  setGraphicsStatusIpcMock,
} from "../api/graphics-ipc-mock";
import { graphicsKeys } from "../api/graphics-keys";
import type { GraphicsStatus } from "../types/graphics";
import { GraphicsImage } from "./graphics-image";

function renderImage(initialStatus?: GraphicsStatus, uid = 42) {
  const queryClient = new QueryClient();
  if (initialStatus) {
    queryClient.setQueryData(graphicsKeys.status(), initialStatus);
  }
  return {
    ...render(
      <QueryClientProvider client={queryClient}>
        <GraphicsImage
          kind="personPortrait"
          uid={uid}
          slot={{
            alt: "Player portrait",
            className: "portrait",
            fallback: <span>Initials</span>,
          }}
        />
      </QueryClientProvider>,
    ),
    queryClient,
  };
}

const LOADED_STATUS = {
  ...DEFAULT_GRAPHICS_STATUS,
  generation: 7,
  selected: true,
};

function loadImage(uid: number) {
  setGraphicsStatusIpcMock(LOADED_STATUS);
  const view = renderImage(LOADED_STATUS, uid);
  fireEvent.load(view.getByRole("img", { name: "Player portrait" }));
  view.unmount();
}

describe("GraphicsImage", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("holds a loaded image so a remounted row reuses it without a new request", () => {
    const image = vi.spyOn(window, "Image");

    loadImage(9001);
    loadImage(9001);

    expect(image).toHaveBeenCalledTimes(1);
    expect(image.mock.results[0]?.value).toHaveProperty(
      "src",
      "http://graphics.localhost/7/personPortrait/9001",
    );
  });

  it("releases the least recently loaded image beyond its bound", () => {
    loadImage(9101);
    for (let uid = 9200; uid < 9200 + 256; uid += 1) {
      loadImage(uid);
    }
    const image = vi.spyOn(window, "Image");

    loadImage(9101);

    expect(image).toHaveBeenCalledTimes(1);
  });

  it("keeps the fallback while the background index rebuild is active", async () => {
    const rebuildingStatus = {
      ...DEFAULT_GRAPHICS_STATUS,
      generation: 7,
      selected: true,
      rebuilding: true,
    };
    setGraphicsStatusIpcMock(rebuildingStatus);
    renderImage(rebuildingStatus);

    expect(await screen.findByText("Initials")).toBeVisible();
    expect(screen.queryByRole("img", { name: "Player portrait" })).toBeNull();
  });

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

  it("retries a failed image after the graphics generation changes", async () => {
    const initialStatus = {
      ...DEFAULT_GRAPHICS_STATUS,
      generation: 7,
      selected: true,
    };
    setGraphicsStatusIpcMock(initialStatus);
    const { queryClient } = renderImage(initialStatus);
    fireEvent.error(
      await screen.findByRole("img", { name: "Player portrait" }),
    );

    const nextStatus = { ...initialStatus, generation: 8 };
    setGraphicsStatusIpcMock(nextStatus);
    queryClient.setQueryData(graphicsKeys.status(), nextStatus);

    expect(
      await screen.findByRole("img", { name: "Player portrait" }),
    ).toHaveAttribute("src", "http://graphics.localhost/8/personPortrait/42");
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
