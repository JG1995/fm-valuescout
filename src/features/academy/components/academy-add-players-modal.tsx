import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useId, useRef, useState } from "react";
import { Button } from "@/components/ui/button/button";
import {
  fieldClasses,
  fieldLabelClasses,
} from "@/components/ui/field/field-styles";
import { Modal } from "@/components/ui/modal/modal";
import { academyCandidatesQueryOptions } from "../api/academy-candidates-query-options";
import { academyKeys } from "../api/academy-keys";
import { assignAcademyMember } from "../api/assign-academy-member";
import type { AcademyCandidate } from "../types/academy";
import { playableAcademyPositions } from "../utils/academy-positions";

type AcademyAddPlayersModalProps = {
  open: boolean;
  academyClassId: number;
  academyClassYear: number;
  onClose: () => void;
};

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

export function AcademyAddPlayersModal({
  open,
  academyClassId,
  academyClassYear,
  onClose,
}: AcademyAddPlayersModalProps) {
  const queryClient = useQueryClient();
  const searchInputId = useId();
  const listboxId = useId();
  const optionIdPrefix = useId();
  const activeOptionRef = useRef<HTMLButtonElement>(null);
  const [search, setSearch] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);
  const candidatesQuery = useQuery({
    ...academyCandidatesQueryOptions(search.trim()),
    enabled: open,
  });
  const candidates = candidatesQuery.data ?? [];
  const assign = useMutation({
    mutationFn: (candidate: AcademyCandidate) =>
      assignAcademyMember(academyClassId, candidate.playerUid),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: academyKeys.classes() }),
        queryClient.invalidateQueries({
          queryKey: academyKeys.academyClass(academyClassId),
        }),
        queryClient.invalidateQueries({ queryKey: academyKeys.candidates() }),
      ]);
      onClose();
    },
  });
  const { reset } = assign;

  useEffect(() => {
    if (open) {
      setSearch("");
      setActiveIndex(0);
      reset();
    }
  }, [open, reset]);

  const activeCandidate = candidates[activeIndex];

  useEffect(() => {
    if (!activeCandidate) {
      return;
    }
    activeOptionRef.current?.scrollIntoView?.({ block: "nearest" });
  }, [activeCandidate]);

  const selectCandidate = (candidate: AcademyCandidate) => {
    if (!assign.isPending) {
      assign.mutate(candidate);
    }
  };

  return (
    <Modal
      open={open}
      title={`Add players to Class of ${academyClassYear}`}
      onClose={() => {
        if (!assign.isPending) {
          onClose();
        }
      }}
      footer={
        <Button
          disabled={assign.isPending}
          variant="secondary"
          onClick={onClose}
        >
          Cancel
        </Button>
      }
    >
      <div className="space-y-3">
        <div className="space-y-1">
          <label className={fieldLabelClasses} htmlFor={searchInputId}>
            Search managed-club players
          </label>
          <input
            aria-activedescendant={
              activeCandidate
                ? `${optionIdPrefix}-${activeCandidate.playerUid}`
                : undefined
            }
            aria-autocomplete="list"
            aria-controls={candidates.length > 0 ? listboxId : undefined}
            aria-expanded={candidates.length > 0}
            aria-haspopup="listbox"
            className={`${fieldClasses} w-full`}
            disabled={assign.isPending}
            id={searchInputId}
            role="combobox"
            type="search"
            value={search}
            onChange={(event) => {
              if (assign.isPending) {
                return;
              }
              setSearch(event.target.value);
              setActiveIndex(0);
              reset();
            }}
            onKeyDown={(event) => {
              if (assign.isPending || candidates.length === 0) {
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
        {assign.isError ? (
          <p className="text-body-sm text-error" role="alert">
            {errorMessage(assign.error)}
          </p>
        ) : null}
        {candidatesQuery.isPending ? (
          <p className="text-body-sm text-on-surface-variant">
            Finding managed-club players…
          </p>
        ) : null}
        {!candidatesQuery.isPending && candidates.length === 0 ? (
          <p className="text-body-sm text-on-surface-variant">
            No unclassified managed-club players match this search.
          </p>
        ) : null}
        {candidates.length > 0 ? (
          <div
            aria-label="Academy candidates"
            className="max-h-80 overflow-y-auto rounded-md border border-outline-variant"
            id={listboxId}
            role="listbox"
          >
            {candidates.map((candidate, index) => (
              <button
                key={candidate.playerUid}
                ref={
                  candidate.playerUid === activeCandidate?.playerUid
                    ? activeOptionRef
                    : undefined
                }
                aria-selected={index === activeIndex}
                className={`flex w-full items-center justify-between gap-3 border-b border-outline-variant px-3 py-2 text-left text-body-sm text-on-surface last:border-b-0 hover:bg-surface-container-high focus-visible:outline-2 focus-visible:outline-offset-[-2px] focus-visible:outline-primary ${
                  index === activeIndex ? "bg-surface-container-high" : ""
                }`}
                disabled={assign.isPending}
                id={`${optionIdPrefix}-${candidate.playerUid}`}
                role="option"
                type="button"
                onMouseDown={(event) => event.preventDefault()}
                onMouseEnter={() => setActiveIndex(index)}
                onClick={() => selectCandidate(candidate)}
              >
                <span className="min-w-0">
                  <span className="block truncate">{candidate.name}</span>
                  <span className="block text-body-sm text-on-surface-variant">
                    {candidate.currentClub} ·{" "}
                    {candidate.age === null ? "—" : `${candidate.age} years`}
                  </span>
                </span>
                <span className="font-mono text-mono-sm text-on-surface-variant">
                  {playableAcademyPositions(candidate.positions).join(", ") ||
                    "—"}
                </span>
              </button>
            ))}
          </div>
        ) : null}
      </div>
    </Modal>
  );
}
