import type { ReactNode } from "react";
import { cn } from "@/utils/cn";

/** One placeholder block. Size it with `className` to the content it stands for. */
export function Skeleton({ className }: { className?: string }) {
  return (
    <div
      aria-hidden="true"
      className={cn(
        "animate-skeleton rounded-sm bg-surface-container-high",
        className,
      )}
    />
  );
}

type LoadingRegionProps = {
  /** Names what is loading, in the user's terms. */
  label: string;
  /** Placeholder blocks shaped like the content that will replace them. */
  children: ReactNode;
  className?: string;
};

/**
 * The Loading state of `.wiki/DESIGN.md`: placeholders in the final layout, so
 * nothing moves when the data arrives. The region stays invisible for the
 * first 150ms, which is longer than most local reads take. The label is always
 * announced and becomes visible when motion is reduced, where the blocks hold
 * still.
 */
export function LoadingRegion({
  label,
  children,
  className,
}: LoadingRegionProps) {
  return (
    <div
      role="status"
      aria-busy="true"
      className={cn("animate-enter-late", className)}
    >
      <p className="text-body-md text-on-surface-variant motion-safe:sr-only">
        {label}
      </p>
      {children}
    </div>
  );
}

const TABLE_ROWS = Array.from({ length: 24 }, (_, index) => `row-${index}`);

type TableSkeletonProps = {
  /** Names what is loading, e.g. "Loading players". */
  label: string;
  /** Rows to draw. The default overfills a full-height table, which clips it. */
  rows?: number;
  className?: string;
};

/**
 * Stands in for a data table while its first page loads: a header band and
 * rows at the real two-line row height. In a full-height panel it fills the
 * height the table will take.
 */
export function TableSkeleton({
  label,
  rows = TABLE_ROWS.length,
  className,
}: TableSkeletonProps) {
  return (
    <LoadingRegion
      label={label}
      className={cn("flex min-h-40 flex-1 flex-col overflow-hidden", className)}
    >
      <div
        aria-hidden="true"
        className="h-table-header-height shrink-0 border-b border-outline-variant"
      />
      {TABLE_ROWS.slice(0, rows).map((row, index) => (
        <div
          key={row}
          aria-hidden="true"
          className="flex h-table-row-height-two-line shrink-0 items-center gap-4 border-b border-outline-variant px-4"
        >
          <Skeleton className="size-6 shrink-0" />
          <Skeleton className={index % 3 === 0 ? "h-3 w-44" : "h-3 w-36"} />
          <Skeleton className="ml-6 h-3 w-10" />
          <Skeleton className="h-3 w-10" />
          <Skeleton className="h-3 w-16" />
        </div>
      ))}
    </LoadingRegion>
  );
}
