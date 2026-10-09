import { expect, type Locator } from "@playwright/test";

export async function expectReadableText(locator: Locator) {
  const ratio = await locator.evaluate((element) => {
    type Node = { parentElement: Node | null };
    type Context = {
      fillStyle: string;
      fillRect: (x: number, y: number, width: number, height: number) => void;
      getImageData: (
        x: number,
        y: number,
        width: number,
        height: number,
      ) => { data: Uint8ClampedArray };
    };
    // SAFETY: Playwright runs this callback in Chromium; the Node test config omits DOM types.
    const browser = globalThis as unknown as {
      document: {
        createElement: (tag: string) => {
          width: number;
          height: number;
          getContext: (
            type: string,
            options: { willReadFrequently: boolean },
          ) => Context;
        };
      };
      getComputedStyle: (node: unknown) => {
        backgroundColor: string;
        color: string;
      };
    };
    const canvas = browser.document.createElement("canvas");
    canvas.width = canvas.height = 1;
    const context = canvas.getContext("2d", { willReadFrequently: true });
    context.fillStyle = "#fff";
    context.fillRect(0, 0, 1, 1);
    const ancestors: Node[] = [];
    // SAFETY: The locator resolves a DOM element whose ancestors expose parentElement.
    const start = element as unknown as Node;
    for (let node: Node | null = start; node; node = node.parentElement)
      ancestors.push(node);
    for (const node of ancestors.reverse()) {
      context.fillStyle = browser.getComputedStyle(node).backgroundColor;
      context.fillRect(0, 0, 1, 1);
    }
    const background = Array.from(context.getImageData(0, 0, 1, 1).data);
    context.fillStyle = browser.getComputedStyle(element).color;
    context.fillRect(0, 0, 1, 1);
    const foreground = Array.from(context.getImageData(0, 0, 1, 1).data);
    const luminance = (rgb: number[]) =>
      rgb
        .slice(0, 3)
        .map((value) => {
          const channel = value / 255;
          return channel <= 0.04045
            ? channel / 12.92
            : ((channel + 0.055) / 1.055) ** 2.4;
        })
        .reduce(
          (sum, value, index) => sum + value * [0.2126, 0.7152, 0.0722][index],
          0,
        );
    const levels = [luminance(background), luminance(foreground)].sort(
      (a, b) => b - a,
    );
    return (levels[0] + 0.05) / (levels[1] + 0.05);
  });
  expect(ratio).toBeGreaterThanOrEqual(4.5);
}
