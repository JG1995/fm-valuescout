import { Info } from "lucide-react";
import type { ReactNode } from "react";

export type DashboardSetupNote = {
  id: string;
  text: string;
  /** Where the user fixes it, when that is a page of this app. */
  action?: ReactNode;
};

/** What is still missing before the dashboard can show everything it offers. */
export function DashboardSetupNotes({
  notes,
}: {
  notes: readonly DashboardSetupNote[];
}) {
  if (notes.length === 0) return null;
  return (
    <section
      aria-label="Setup"
      className="shrink-0 rounded-lg border border-outline-variant bg-surface-container px-4 py-2"
    >
      <ul className="divide-y divide-outline-variant">
        {notes.map((note) => (
          <li
            key={note.id}
            className="flex min-h-10 flex-wrap items-center gap-x-3 gap-y-1 py-1"
          >
            <Info
              aria-hidden="true"
              size={16}
              strokeWidth={1.5}
              className="shrink-0 text-info"
            />
            <span className="min-w-0 flex-1 text-body-sm text-on-surface">
              {note.text}
            </span>
            {note.action}
          </li>
        ))}
      </ul>
    </section>
  );
}
