import { useEffect, useRef } from "react";

const POPUP_GAP = 4;
const VIEWPORT_PADDING = 8;

type AnchoredPopoverOptions = {
  maxHeight?: number;
  minWidth?: number;
};

export function useAnchoredPopover<TAnchor extends HTMLElement>(
  open: boolean,
  { maxHeight = 256, minWidth = 0 }: AnchoredPopoverOptions = {},
) {
  const anchorRef = useRef<TAnchor>(null);
  const popoverRef = useRef<HTMLDivElement>(null);
  const supportsPopover =
    typeof HTMLElement !== "undefined" &&
    "showPopover" in HTMLElement.prototype;

  useEffect(() => {
    const anchor = anchorRef.current;
    const popover = popoverRef.current;
    if (!open || !anchor || !popover || !supportsPopover) {
      return;
    }

    const positionPopover = () => {
      const anchorBounds = anchor.getBoundingClientRect();
      const width = Math.min(
        Math.max(anchorBounds.width, minWidth),
        window.innerWidth - VIEWPORT_PADDING * 2,
      );
      const left = Math.min(
        Math.max(VIEWPORT_PADDING, anchorBounds.left),
        window.innerWidth - width - VIEWPORT_PADDING,
      );
      const spaceBelow =
        window.innerHeight - anchorBounds.bottom - POPUP_GAP - VIEWPORT_PADDING;
      const spaceAbove = anchorBounds.top - POPUP_GAP - VIEWPORT_PADDING;
      const placeBelow = spaceBelow >= Math.min(maxHeight, spaceAbove);

      popover.style.position = "fixed";
      popover.style.left = `${left}px`;
      popover.style.right = "auto";
      popover.style.width = `${width}px`;
      popover.style.maxHeight = `${Math.min(
        maxHeight,
        Math.max(spaceBelow, spaceAbove),
      )}px`;
      if (placeBelow) {
        popover.style.top = `${anchorBounds.bottom + POPUP_GAP}px`;
        popover.style.bottom = "auto";
      } else {
        popover.style.top = "auto";
        popover.style.bottom = `${window.innerHeight - anchorBounds.top + POPUP_GAP}px`;
      }
    };

    positionPopover();
    popover.showPopover();
    window.addEventListener("resize", positionPopover);
    window.addEventListener("scroll", positionPopover, true);
    return () => {
      window.removeEventListener("resize", positionPopover);
      window.removeEventListener("scroll", positionPopover, true);
      popover.hidePopover();
    };
  }, [maxHeight, minWidth, open, supportsPopover]);

  return {
    anchorRef,
    popoverRef,
    popover: supportsPopover ? ("manual" as const) : undefined,
  };
}
