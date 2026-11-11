import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Pin, PinOff } from "lucide-react";
import { useEffect, useId, useRef, useState } from "react";
import { Button } from "@/components/ui/button/button";
import {
  fieldClasses,
  fieldLabelClasses,
} from "@/components/ui/field/field-styles";
import { Modal } from "@/components/ui/modal/modal";
import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import { assignPlannerPlayer } from "../api/assign-planner-player";
import { clearPlannerAssignment } from "../api/clear-planner-assignment";
import { movePlannerPlayer } from "../api/move-planner-player";
import { plannerKeys } from "../api/planner-keys";
import { plannerSlotCandidatesQueryOptions } from "../api/planner-slot-candidates-query-options";
import { setPlannerAssignmentPinned } from "../api/set-planner-assignment-pinned";
import type { PlannerSlotCandidate } from "../types/depth";
import type { PlannerTactic, TacticOptions } from "../types/tactic";
import type { PlannerTeam } from "../types/team";
import { ordinalStringLabel } from "../utils/string-label";
import { linkedPositionDescriptionForId } from "../utils/tactic-editor";

const SEARCH_DEBOUNCE_MS = 200;

export type PlannerSlotTarget = {
  team: PlannerTeam;
  stringId: number;
  stringOrder: number;
  laneId: string;
  laneName: string;
  /** The player in the cell, whom a pick replaces. */
  occupant: { playerUid: number; name: string; pinned: boolean } | null;
};

type PlannerSlotFitPickerProps = {
  activeSaveId: number;
  open: boolean;
  target: PlannerSlotTarget;
  tactic: PlannerTactic;
  options: TacticOptions;
  teamLabels: Partial<Record<PlannerTeam, string>>;
  onClose: () => void;
  onMutationError: (message: string) => void;
};

function scoreEvidence(score: number | null) {
  return score === null ? "—" : score;
}

/** What Optimize holds against a candidate for the slot, in the picker's words. */
function candidateNotes(candidate: PlannerSlotCandidate) {
  return [
    candidate.positionFit === "unfamiliar" ? "Unfamiliar position" : null,
    candidate.positionFit === "unsuited" ? "Cannot play this position" : null,
    candidate.footFit === "mismatch" ? "Wrong foot" : null,
    candidate.footFit === "excluded" ? "Wrong foot: slot is strict" : null,
    candidate.overAge ? "Over age limit" : null,
  ].filter((note) => note !== null);
}

function CandidateScore({
  label,
  score,
}: {
  label: string;
  score: number | null;
}) {
  return score === null ? (
    <span className="font-mono text-mono-sm text-on-surface-variant">—</span>
  ) : (
    <ScoreBadge roleName={label} score={score} />
  );
}

function assignmentLocation(
  candidate: PlannerSlotCandidate,
  tactic: PlannerTactic,
  options: TacticOptions,
  teamLabels: Partial<Record<PlannerTeam, string>>,
) {
  const location = candidate.assignmentLocation;
  if (!location) {
    return "Unassigned";
  }
  return `Assigned: ${slotLocation(
    location.team,
    location.stringOrder,
    linkedPositionDescriptionForId(location.laneId, tactic.lanes, options),
    teamLabels,
  )}`;
}

function slotLocation(
  team: PlannerTeam,
  stringOrder: number,
  laneName: string,
  teamLabels: Partial<Record<PlannerTeam, string>>,
) {
  return `${teamLabels[team] ?? team} · ${ordinalStringLabel(stringOrder)} · ${laneName}`;
}

function targetLocation(
  target: PlannerSlotTarget,
  teamLabels: Partial<Record<PlannerTeam, string>>,
) {
  return slotLocation(
    target.team,
    target.stringOrder,
    target.laneName,
    teamLabels,
  );
}

function moveConfirmation(
  candidate: PlannerSlotCandidate,
  target: PlannerSlotTarget,
  tactic: PlannerTactic,
  options: TacticOptions,
  teamLabels: Partial<Record<PlannerTeam, string>>,
) {
  const location = candidate.assignmentLocation;
  if (!location) {
    return null;
  }
  return `Move ${candidate.name} from ${slotLocation(
    location.team,
    location.stringOrder,
    linkedPositionDescriptionForId(location.laneId, tactic.lanes, options),
    teamLabels,
  )} to ${targetLocation(target, teamLabels)}?`;
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

export function PlannerSlotFitPicker({
  activeSaveId,
  open,
  target,
  tactic,
  options,
  teamLabels,
  onClose,
  onMutationError,
}: PlannerSlotFitPickerProps) {
  const queryClient = useQueryClient();
  const inputRef = useRef<HTMLInputElement>(null);
  const activeOptionRef = useRef<HTMLButtonElement>(null);
  const returningToSearchRef = useRef(false);
  const searchInputId = useId();
  const listboxId = useId();
  const optionIdPrefix = useId();
  const [search, setSearch] = useState("");
  const [debouncedSearch, setDebouncedSearch] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);
  const [moveCandidate, setMoveCandidate] =
    useState<PlannerSlotCandidate | null>(null);
  const occupant = target.occupant;

  useEffect(() => {
    setActiveIndex(0);
    const timer = window.setTimeout(() => {
      setDebouncedSearch(search.trim());
    }, SEARCH_DEBOUNCE_MS);
    return () => {
      window.clearTimeout(timer);
    };
  }, [search]);

  useEffect(() => {
    if (moveCandidate) {
      returningToSearchRef.current = true;
    } else if (returningToSearchRef.current) {
      inputRef.current?.focus();
      returningToSearchRef.current = false;
    }
  }, [moveCandidate]);

  const candidatesQuery = useQuery(
    plannerSlotCandidatesQueryOptions(activeSaveId, {
      team: target.team,
      laneId: target.laneId,
      search: debouncedSearch,
    }),
  );
  const candidates = candidatesQuery.data ?? [];

  const closeAfterSuccess = async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: plannerKeys.depth() }),
      queryClient.invalidateQueries({ queryKey: plannerKeys.slotCandidates() }),
    ]);
    onClose();
  };
  const closeAfterError = (error: unknown) => {
    onMutationError(errorMessage(error));
    onClose();
  };
  const assign = useMutation({
    mutationFn: (candidate: PlannerSlotCandidate) =>
      assignPlannerPlayer(target.stringId, target.laneId, candidate.playerUid),
    onSuccess: closeAfterSuccess,
    onError: closeAfterError,
  });
  const move = useMutation({
    mutationFn: (candidate: PlannerSlotCandidate) =>
      movePlannerPlayer(target.stringId, target.laneId, candidate.playerUid),
    onSuccess: closeAfterSuccess,
    onError: closeAfterError,
  });
  const clear = useMutation({
    mutationFn: () => clearPlannerAssignment(target.stringId, target.laneId),
    onSuccess: closeAfterSuccess,
    onError: closeAfterError,
  });
  const pin = useMutation({
    mutationFn: (pinned: boolean) =>
      setPlannerAssignmentPinned(target.stringId, target.laneId, pinned),
    onSuccess: closeAfterSuccess,
    onError: closeAfterError,
  });
  const isMutating =
    assign.isPending || move.isPending || clear.isPending || pin.isPending;

  const selectCandidate = (candidate: PlannerSlotCandidate) => {
    if (candidate.playerUid === occupant?.playerUid) {
      return;
    }
    if (candidate.assignmentLocation) {
      setMoveCandidate(candidate);
      return;
    }
    assign.mutate(candidate);
  };

  const activeCandidate = candidates[activeIndex];
  const isMoveConfirmation = moveCandidate !== null;
  const showCandidates = candidates.length > 0;

  useEffect(() => {
    if (!activeCandidate) {
      return;
    }
    activeOptionRef.current?.scrollIntoView?.({ block: "nearest" });
  }, [activeCandidate]);

  return (
    <Modal
      open={open}
      title={
        isMoveConfirmation
          ? `Move ${moveCandidate.name}?`
          : occupant
            ? `Replace ${occupant.name}`
            : `Find a player for ${target.laneName}`
      }
      onClose={() => {
        if (!isMutating) onClose();
      }}
      footer={
        isMoveConfirmation ? (
          <>
            <Button
              disabled={isMutating}
              variant="secondary"
              onClick={() => setMoveCandidate(null)}
            >
              Back
            </Button>
            <Button
              autoFocus
              disabled={isMutating}
              onClick={() => {
                if (moveCandidate) {
                  move.mutate(moveCandidate);
                }
              }}
            >
              {isMutating ? "Moving…" : "Confirm move"}
            </Button>
          </>
        ) : (
          <Button disabled={isMutating} variant="secondary" onClick={onClose}>
            Cancel
          </Button>
        )
      }
    >
      {isMoveConfirmation ? (
        <p className="text-body-md text-on-surface-variant">
          {moveConfirmation(moveCandidate, target, tactic, options, teamLabels)}
          {occupant && occupant.playerUid !== moveCandidate.playerUid
            ? ` ${occupant.name} is left unassigned.`
            : null}
        </p>
      ) : (
        <div className="space-y-3">
          {occupant ? (
            <section
              aria-label="Current player"
              className="flex flex-wrap items-center justify-between gap-2 rounded-md border border-outline-variant px-3 py-2"
            >
              <p className="min-w-0 text-body-sm text-on-surface-variant">
                <span className="block truncate text-on-surface">
                  {occupant.name}
                </span>
                {targetLocation(target, teamLabels)}
                <span className="block">
                  {occupant.pinned
                    ? "Pinned: Optimize keeps him here."
                    : "Placed by Optimize, which may replace him."}
                </span>
              </p>
              <span className="flex shrink-0 gap-2">
                <Button
                  disabled={isMutating}
                  variant="secondary"
                  icon={occupant.pinned ? PinOff : Pin}
                  onClick={() => pin.mutate(!occupant.pinned)}
                >
                  {occupant.pinned ? "Unpin" : "Pin"}
                </Button>
                <Button
                  disabled={isMutating}
                  variant="secondary"
                  onClick={() => clear.mutate()}
                >
                  {clear.isPending ? "Clearing…" : "Clear slot"}
                </Button>
              </span>
            </section>
          ) : null}
          <div className="space-y-1">
            <label className={fieldLabelClasses} htmlFor={searchInputId}>
              {occupant
                ? "Search for a replacement"
                : "Search squad candidates"}
            </label>
            <input
              ref={inputRef}
              aria-activedescendant={
                showCandidates && activeCandidate
                  ? `${optionIdPrefix}-${activeCandidate.playerUid}`
                  : undefined
              }
              aria-autocomplete="list"
              aria-controls={showCandidates ? listboxId : undefined}
              aria-expanded={showCandidates}
              aria-haspopup="listbox"
              className={`${fieldClasses} w-full`}
              id={searchInputId}
              disabled={isMutating}
              role="combobox"
              type="text"
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              onKeyDown={(event) => {
                if (candidates.length === 0) {
                  return;
                }
                if (event.key === "ArrowDown") {
                  event.preventDefault();
                  setActiveIndex((index) => (index + 1) % candidates.length);
                } else if (event.key === "ArrowUp") {
                  event.preventDefault();
                  setActiveIndex(
                    (index) =>
                      (index - 1 + candidates.length) % candidates.length,
                  );
                } else if (event.key === "Enter" && activeCandidate) {
                  event.preventDefault();
                  selectCandidate(activeCandidate);
                }
              }}
            />
          </div>
          {candidatesQuery.isError ? (
            <p className="text-body-sm text-error" role="alert">
              {errorMessage(candidatesQuery.error)}
            </p>
          ) : null}
          {candidatesQuery.isPending ? (
            <p className="text-body-sm text-on-surface-variant">
              Finding candidates…
            </p>
          ) : null}
          {!candidatesQuery.isPending && candidates.length === 0 ? (
            <p className="text-body-sm text-on-surface-variant">
              No managed-club players match this search.
            </p>
          ) : null}
          {candidates.length > 0 ? (
            <div
              aria-label="Slot-fit candidates"
              className="max-h-80 overflow-y-auto rounded-md border border-outline-variant"
              id={listboxId}
              role="listbox"
            >
              {candidates.map((candidate, index) => {
                const inThisSlot = candidate.playerUid === occupant?.playerUid;
                const notes = candidateNotes(candidate);
                return (
                  <button
                    key={candidate.playerUid}
                    ref={
                      candidate.playerUid === activeCandidate?.playerUid
                        ? activeOptionRef
                        : undefined
                    }
                    aria-selected={index === activeIndex}
                    className={`flex w-full items-center justify-between gap-3 border-b border-outline-variant px-3 py-2 text-left text-body-sm text-on-surface last:border-b-0 hover:bg-surface-container-high focus-visible:outline-2 focus-visible:outline-offset-[-2px] focus-visible:outline-primary disabled:opacity-45 ${
                      index === activeIndex ? "bg-surface-container-high" : ""
                    }`}
                    disabled={isMutating || inThisSlot}
                    id={`${optionIdPrefix}-${candidate.playerUid}`}
                    role="option"
                    type="button"
                    onMouseDown={(event) => event.preventDefault()}
                    onMouseEnter={() => setActiveIndex(index)}
                    onClick={() => selectCandidate(candidate)}
                  >
                    <span className="min-w-0">
                      <span className="block truncate">
                        {candidate.name}
                        {candidate.age === null ? null : (
                          <span className="text-on-surface-variant">
                            {" "}
                            · Age {candidate.age}
                          </span>
                        )}
                      </span>
                      <span className="block text-body-sm text-on-surface-variant">
                        {candidate.currentClub} ·{" "}
                        {inThisSlot
                          ? "In this slot"
                          : assignmentLocation(
                              candidate,
                              tactic,
                              options,
                              teamLabels,
                            )}
                      </span>
                      <span className="block font-mono text-mono-sm text-on-surface-variant">
                        IP {scoreEvidence(candidate.ipScore)} · OOP{" "}
                        {scoreEvidence(candidate.oopScore)}
                      </span>
                      {notes.length > 0 ? (
                        <span className="block text-body-sm text-warning">
                          {notes.join(" · ")}
                        </span>
                      ) : null}
                    </span>
                    <span
                      className="flex shrink-0 items-center gap-1"
                      title="Current to expected combined role score"
                    >
                      <CandidateScore
                        label="Combined role score"
                        score={candidate.combinedScore}
                      />
                      <span
                        aria-hidden="true"
                        className="text-label-sm text-on-surface-variant"
                      >
                        →
                      </span>
                      <CandidateScore
                        label="Expected combined role score"
                        score={candidate.expectedCombinedScore}
                      />
                    </span>
                  </button>
                );
              })}
            </div>
          ) : null}
        </div>
      )}
    </Modal>
  );
}
