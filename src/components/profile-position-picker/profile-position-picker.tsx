import { attributeValueTier } from "@/components/ui/attribute-value/attribute-value";
import {
  type PositionFamiliarityMap,
  PROFILE_POSITION_ROWS,
} from "@/utils/profile-position-roles";

export function ProfilePositionPicker({
  positions,
  selectedPosition,
  onSelectPosition,
}: {
  positions: PositionFamiliarityMap;
  selectedPosition: string;
  onSelectPosition: (position: string) => void;
}) {
  return (
    <fieldset className="relative isolate overflow-hidden rounded-lg border border-outline-variant bg-surface-container-lowest p-3">
      <legend className="sr-only">Select a pitch position</legend>
      <div
        aria-hidden="true"
        className="pointer-events-none absolute inset-3 rounded-sm border border-outline/45"
      >
        <span className="absolute top-1/2 right-0 left-0 border-t border-outline/45" />
        <span className="absolute top-1/2 left-1/2 size-14 -translate-x-1/2 -translate-y-1/2 rounded-full border border-outline/45" />
        <span className="absolute top-0 left-1/2 h-10 w-1/2 -translate-x-1/2 border-x border-b border-outline/45" />
        <span className="absolute bottom-0 left-1/2 h-10 w-1/2 -translate-x-1/2 border-x border-t border-outline/45" />
      </div>
      <div className="relative z-10 grid h-full grid-cols-3 content-between gap-1.5">
        {PROFILE_POSITION_ROWS.flatMap((row, rowIndex) =>
          row.map((position, columnIndex) => {
            const key = `${rowIndex}:${columnIndex}`;
            if (position === null) {
              return <span aria-hidden="true" key={key} className="min-h-11" />;
            }

            const familiarity = positions[position];
            const knownFamiliarity =
              typeof familiarity === "number" && familiarity > 0;
            const selected = position === selectedPosition;
            const accessibleName = knownFamiliarity
              ? `${position}, familiarity ${familiarity}`
              : `${position}, no recorded familiarity`;

            return (
              <button
                key={key}
                type="button"
                aria-label={accessibleName}
                aria-pressed={selected}
                data-tier={
                  knownFamiliarity ? attributeValueTier(familiarity) : undefined
                }
                className={`min-h-11 rounded-md border px-1 py-1 text-center transition-colors duration-150 ease-out focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary ${
                  selected
                    ? "border-primary bg-primary-container text-on-primary-container ring-2 ring-primary/50"
                    : familiarity === 1
                      ? "border-outline-variant bg-surface-container/50 text-score-1 hover:bg-surface-container"
                      : knownFamiliarity
                        ? "border-outline bg-surface-container-high hover:bg-surface-container-highest data-[tier=1]:text-score-1 data-[tier=2]:text-score-2 data-[tier=3]:text-score-3 data-[tier=4]:text-score-4"
                        : "border-outline-variant bg-surface-container/85 text-on-surface-variant hover:bg-surface-container-high"
                }`}
                onClick={() => onSelectPosition(position)}
              >
                <span className="block text-label-md">{position}</span>
                <span className="block font-mono text-[10px] tabular-nums">
                  {knownFamiliarity ? familiarity : "—"}
                </span>
              </button>
            );
          }),
        )}
      </div>
    </fieldset>
  );
}
