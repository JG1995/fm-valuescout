import { Maximize2 } from "lucide-react";
import type { ReactNode } from "react";
import { Button } from "@/components/ui/button/button";

/**
 * For a panel that holds a `PanelExpandButton`. The panel anchors the button's
 * hit area and takes a stronger border while the pointer is over it.
 */
export const expandablePanelClass =
  "relative has-[[data-panel-expand]:hover]:border-outline";

/** Lifts a panel's other controls and links above the expand hit area. */
export const aboveExpandClass = "relative z-10";

/**
 * Opens a panel's fuller view in a modal. Its hit area covers the whole panel,
 * so a click anywhere on the summary opens it; the button itself is what the
 * keyboard and assistive technology reach.
 */
export function PanelExpandButton({
  onClick,
  children,
}: {
  onClick: () => void;
  children: ReactNode;
}) {
  return (
    <Button
      variant="secondary"
      icon={Maximize2}
      data-panel-expand=""
      className="after:absolute after:inset-0"
      onClick={onClick}
    >
      {children}
    </Button>
  );
}
