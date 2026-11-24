import type { ReactNode } from "react";

/**
 * A pending sort, filter, or column change on a table that keeps its current
 * rows. The status floats over the first rows instead of taking a line of its
 * own, so the table does not move down and back up around every change. It
 * stays invisible for the first 150ms, which a cached order never reaches.
 */
export function TablePendingStatus({ children }: { children: ReactNode }) {
  return (
    // Zero height: an anchor between the toolbar and the table.
    <div className="relative z-20 h-0 shrink-0">
      <p
        role="status"
        className="pointer-events-none absolute top-19 left-1/2 -translate-x-1/2 animate-enter-late whitespace-nowrap rounded-full border border-outline-variant bg-surface-container-highest px-3 py-1 text-body-sm text-on-surface shadow-overlay"
      >
        {children}
      </p>
    </div>
  );
}
