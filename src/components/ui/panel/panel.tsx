import type { ReactNode } from "react";
import { cn } from "@/utils/cn";

type PanelProps = {
  title?: ReactNode;
  headingLevel?: 2 | 3;
  /** Rendered on the header row, including when there is no title. */
  actions?: ReactNode;
  /** Drop the content padding when the only child is a full-bleed table. */
  flush?: boolean;
  className?: string;
  /** Adds layout constraints to the content wrapper without changing its padding. */
  contentClassName?: string;
  children: ReactNode;
};

export function Panel({
  title,
  headingLevel = 2,
  actions,
  flush = false,
  className,
  contentClassName,
  children,
}: PanelProps) {
  const Heading = headingLevel === 3 ? "h3" : "h2";
  return (
    <section
      className={cn(
        "overflow-hidden rounded-lg border border-outline-variant bg-surface-container",
        className,
      )}
    >
      {title || actions ? (
        <div className="flex shrink-0 flex-wrap items-center justify-between gap-2 px-4 pt-4">
          {typeof title === "string" ? (
            <Heading
              className={cn(
                "text-on-surface",
                headingLevel === 3
                  ? "text-body-md font-semibold"
                  : "text-headline-sm",
              )}
            >
              {title}
            </Heading>
          ) : (
            title
          )}
          {actions ? (
            <div className="ml-auto flex flex-wrap items-center gap-2">
              {actions}
            </div>
          ) : null}
        </div>
      ) : null}
      <div
        className={cn(
          flush ? (title ? "mt-4" : actions ? "mt-1" : undefined) : "p-4",
          contentClassName,
        )}
      >
        {children}
      </div>
    </section>
  );
}
