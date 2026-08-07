import type { ReactNode } from "react";
import { cn } from "@/utils/cn";

type PanelProps = {
  title?: ReactNode;
  /** Rendered on the header row, right-aligned beside the title. */
  actions?: ReactNode;
  /** Drop the content padding when the only child is a full-bleed table. */
  flush?: boolean;
  className?: string;
  children: ReactNode;
};

export function Panel({
  title,
  actions,
  flush = false,
  className,
  children,
}: PanelProps) {
  return (
    <section
      className={cn(
        "overflow-hidden rounded-lg border border-outline-variant bg-surface-container",
        className,
      )}
    >
      {title ? (
        <div className="flex items-center justify-between gap-4 px-4 pt-4">
          {typeof title === "string" ? (
            <h2 className="text-headline-sm text-on-surface">{title}</h2>
          ) : (
            title
          )}
          {actions}
        </div>
      ) : null}
      <div className={cn(flush ? (title ? "mt-4" : undefined) : "p-4")}>
        {children}
      </div>
    </section>
  );
}
