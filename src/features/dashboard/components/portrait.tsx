import type { ReactNode } from "react";
import { cn } from "@/utils/cn";

/**
 * A fixed slot for a player's portrait. `className` sets its size and radius;
 * the slot keeps its place whether or not an image arrives.
 */
export function Portrait({
  className,
  children,
}: {
  className: string;
  /** The image or its fallback, from the dashboard's graphics. */
  children: ReactNode;
}) {
  return (
    <span
      className={cn(
        "flex shrink-0 items-center justify-center overflow-hidden bg-surface-container-high [container-type:size]",
        className,
      )}
    >
      {children}
    </span>
  );
}

/** The player's initials, sized to the portrait slot they stand in for. */
export function PortraitInitials({ name }: { name: string }) {
  return (
    <span
      aria-hidden="true"
      className="font-mono text-[36cqh] leading-none font-semibold text-on-surface-variant"
    >
      {name
        .split(" ")
        .map((part) => part[0])
        .join("")
        .slice(0, 2)
        .toUpperCase()}
    </span>
  );
}
