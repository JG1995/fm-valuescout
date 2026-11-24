import { LoadingRegion, Skeleton } from "@/components/ui/skeleton/skeleton";
import { cn } from "@/utils/cn";
import { cardPanelSummaryClass } from "./dashboard-grid";
import { cardGridMinHeightClass } from "./player-card";

// The card grid's layout before it is measured: three cards in one row.
const CARDS = ["first", "second", "third"];
// A tactic has eleven slots.
const SLOTS = Array.from({ length: 11 }, (_, index) => `slot-${index}`);

/**
 * Stands in for a card panel while its read loads. The label takes the
 * summary line's place and stays visible: the signings read can run for
 * seconds, and a pulse alone would not say what it is waiting for.
 */
export function CardPanelSkeleton({ label }: { label: string }) {
  return (
    <div
      role="status"
      aria-busy="true"
      className="flex flex-1 animate-enter-late flex-col"
    >
      <p className={cardPanelSummaryClass}>{label}</p>
      <div
        aria-hidden="true"
        className={cn(
          "grid flex-1 grid-cols-3 gap-px border-t border-outline-variant bg-outline-variant",
          cardGridMinHeightClass,
        )}
      >
        {CARDS.map((card) => (
          <div
            key={card}
            className="flex flex-col gap-3 bg-surface-container p-4"
          >
            <Skeleton className="size-12" />
            <Skeleton className="h-3 w-3/4" />
            <Skeleton className="h-3 w-1/2" />
          </div>
        ))}
      </div>
    </div>
  );
}

/** Stands in for the starting XI's slot rows and its note while they load. */
export function StartingXiSkeleton({ label }: { label: string }) {
  return (
    <LoadingRegion label={label} className="flex flex-1 flex-col">
      <div aria-hidden="true" className="flex flex-1 flex-col">
        {SLOTS.map((slot) => (
          <div
            key={slot}
            className="flex min-h-table-row-height-two-line flex-1 basis-0 items-center gap-3 border-t border-outline-variant px-4"
          >
            <Skeleton className="h-3 w-20" />
            <Skeleton className="size-7" />
            <Skeleton className="h-3 w-32" />
          </div>
        ))}
      </div>
      <div
        aria-hidden="true"
        className="shrink-0 border-t border-outline-variant px-4 py-2 text-body-sm"
      >
        <div className="flex h-[2lh] flex-col justify-around">
          <Skeleton className="h-3 w-full" />
          <Skeleton className="h-3 w-2/3" />
        </div>
      </div>
    </LoadingRegion>
  );
}
