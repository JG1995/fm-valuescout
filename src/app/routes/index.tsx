import { createFileRoute, Link } from "@tanstack/react-router";
import { LayoutDashboard } from "lucide-react";
import { EmptyState } from "@/components/ui/empty-state/empty-state";

export const Route = createFileRoute("/")({
  component: IndexPage,
});

// A plain path, like the nav's destinations: Search's validateSearch fills the
// sort, filters, and preferred analysis view instead of this link fixing them.
const playerSearchPath: string = "/search";

const destinationLinkClass =
  "inline-flex h-8 items-center rounded-md border border-outline px-4 text-label-lg text-on-surface transition-colors duration-150 ease-out hover:bg-surface-container-high active:bg-surface-container-highest";

// TODO: replace once Dashboard's purpose is accepted (.wiki/REDESIGN.md §12).
function IndexPage() {
  return (
    <div className="space-y-gutter">
      <h1 className="text-headline-md text-on-surface">Dashboard</h1>
      <EmptyState
        icon={LayoutDashboard}
        title="Dashboard is not built yet"
        action={
          <div className="flex gap-2 pt-2">
            <Link to={playerSearchPath} className={destinationLinkClass}>
              Open Player Search
            </Link>
            <Link to="/my-club" className={destinationLinkClass}>
              Open My Club
            </Link>
          </div>
        }
      >
        Scout players in Player Search, or manage your squad in My Club.
      </EmptyState>
    </div>
  );
}
