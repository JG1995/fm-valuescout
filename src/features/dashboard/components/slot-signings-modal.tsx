import { Modal } from "@/components/ui/modal/modal";
import type {
  DashboardGraphics,
  DashboardScoreMode,
  DashboardSignings,
} from "../types/dashboard";
import { orderedSlots, slotContext, slotLabel } from "../utils/slot-label";
import {
  DashboardTable,
  Gain,
  PlayerIdentity,
  rowClass,
  Score,
  SlotCell,
} from "./dashboard-table";
import { Portrait } from "./portrait";
import { ScoreModeToggle } from "./score-mode-toggle";
import { signingClubLine, signingValueLine } from "./signing-card";

/**
 * Every tactic slot with its current player and the compared player who would
 * improve it most. Slots answer independently, so one player can appear for
 * several.
 */
export function SlotSigningsModal({
  open,
  onClose,
  signings,
  mode,
  onModeChange,
  poolLabel,
  candidateNoun,
  refreshing,
  graphics,
}: {
  open: boolean;
  onClose: () => void;
  signings: DashboardSignings;
  mode: DashboardScoreMode;
  onModeChange: (mode: DashboardScoreMode) => void;
  /** The compared shortlist's name, or "Realistic signings". */
  poolLabel: string;
  /** What one compared player is called, for example "realistic signing". */
  candidateNoun: string;
  /** Whether the rows still belong to the previous mode or pool. */
  refreshing: boolean;
  graphics: DashboardGraphics;
}) {
  return (
    <Modal
      open={open}
      onClose={onClose}
      title="Best signing for each slot"
      variant="informational"
      className="max-w-[64rem]!"
    >
      <div className="flex flex-wrap items-center justify-between gap-2 pb-3">
        <p className="text-body-md text-on-surface-variant" role="status">
          {poolLabel}: the player who would improve each slot most.
          {refreshing ? " Updating…" : null}
        </p>
        <ScoreModeToggle mode={mode} onChange={onModeChange} />
      </div>
      {signings.scoresAvailable ? (
        <div
          className={
            refreshing
              ? "overflow-hidden rounded-lg bg-surface-container opacity-60"
              : "overflow-hidden rounded-lg bg-surface-container"
          }
        >
          <DashboardTable
            caption="Each tactic slot's current player and best signing"
            headings={[
              { label: "Slot", width: "w-44" },
              { label: "Current player", width: "w-60" },
              { label: "Best signing" },
              { label: "Score", numeric: true, width: "w-16" },
              { label: "Gain", numeric: true, width: "w-16" },
            ]}
          >
            {orderedSlots(signings.slots).map((slot) => {
              const label = slotLabel(slot);
              const best = slot.bestSigning;
              return (
                <tr key={slot.laneId} className={rowClass}>
                  <SlotCell label={label} context={slotContext(slot)} />
                  <td className="min-w-0 px-2">
                    {slot.incumbent ? (
                      <span className="flex min-w-0 items-center gap-2">
                        <Portrait className="size-7 rounded-sm">
                          {graphics.portrait(
                            slot.incumbent.uid,
                            slot.incumbent.name,
                          )}
                        </Portrait>
                        <span
                          className="min-w-0 flex-1 truncate text-body-sm text-on-surface"
                          title={slot.incumbent.name}
                        >
                          {slot.incumbent.name}
                        </span>
                        <Score
                          score={slot.incumbent.score}
                          name={`${slot.incumbent.name} at ${label.compact}`}
                        />
                      </span>
                    ) : (
                      <span className="text-body-sm text-on-surface-variant">
                        Empty slot
                      </span>
                    )}
                  </td>
                  {best ? (
                    <>
                      <th
                        scope="row"
                        className="min-w-0 px-2 text-left font-normal"
                      >
                        <PlayerIdentity
                          uid={best.uid}
                          name={best.name}
                          detail={`${signingClubLine(best)} · ${signingValueLine(best, mode)}`}
                          view={mode}
                          portrait={graphics.portrait(best.uid, best.name)}
                        />
                      </th>
                      <td className="px-2 text-right">
                        <Score
                          score={best.score}
                          name={`${best.name} at ${label.compact}`}
                        />
                      </td>
                      <td className="px-2 text-right">
                        <Gain points={best.improvement} />
                      </td>
                    </>
                  ) : (
                    <td
                      colSpan={3}
                      className="px-2 text-body-sm text-on-surface-variant"
                    >
                      No {candidateNoun} improves this slot
                    </td>
                  )}
                </tr>
              );
            })}
          </DashboardTable>
        </div>
      ) : (
        <p className="text-body-md text-on-surface">
          Moneyball scores need season stats from your linked FM save. Switch to
          General, or link the save in Settings and use Load Data.
        </p>
      )}
    </Modal>
  );
}
