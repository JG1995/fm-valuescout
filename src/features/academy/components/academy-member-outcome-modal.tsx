import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useEffect, useId, useMemo, useRef, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { TextField } from "@/components/ui/field/text-field";
import { Modal } from "@/components/ui/modal/modal";
import { academyKeys } from "../api/academy-keys";
import { setAcademyMemberOutcome } from "../api/set-academy-member-outcome";
import type { AcademyMember } from "../types/academy";

const CLUB_SUGGEST_LIMIT = 10;

export type AcademyMemberOutcomeMode = "sale" | "released" | "clear";

type AcademyMemberOutcomeModalProps = {
  academyClassId: number;
  target: AcademyMember | null;
  mode: AcademyMemberOutcomeMode | null;
  clubOptions: string[];
  returnFocusTo?: HTMLElement | null;
  onClose: () => void;
};

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

export function AcademyMemberOutcomeModal({
  academyClassId,
  target,
  mode,
  clubOptions,
  returnFocusTo,
  onClose,
}: AcademyMemberOutcomeModalProps) {
  const queryClient = useQueryClient();
  const listboxId = useId();
  const optionIdPrefix = useId();
  const activeOptionRef = useRef<HTMLButtonElement>(null);
  const [buyingClub, setBuyingClub] = useState("");
  const [saleFeeEur, setSaleFeeEur] = useState("");
  const [suggestionsOpen, setSuggestionsOpen] = useState(false);
  const [activeSuggestion, setActiveSuggestion] = useState(0);
  const [validationError, setValidationError] = useState<string | null>(null);
  const [visibleTarget, setVisibleTarget] = useState(target);
  const [visibleMode, setVisibleMode] = useState(mode);
  const [visibleReturnFocusTo, setVisibleReturnFocusTo] =
    useState(returnFocusTo);
  const isOpen = target !== null && mode !== null;
  const activeTarget = target ?? visibleTarget;
  const activeMode = mode ?? visibleMode;
  const name =
    activeTarget?.currentName ?? activeTarget?.lastKnownName ?? "player";
  const mutation = useMutation({
    mutationFn: (nextMode: AcademyMemberOutcomeMode) => {
      if (!activeTarget || !activeMode) {
        throw new Error("Select an academy player before recording an outcome");
      }
      if (nextMode === "sale") {
        return setAcademyMemberOutcome(academyClassId, activeTarget.playerUid, {
          status: "sold",
          buyingClub: buyingClub.trim(),
          saleFeeEur: Number(saleFeeEur),
        });
      }
      if (nextMode === "released") {
        return setAcademyMemberOutcome(academyClassId, activeTarget.playerUid, {
          status: "released",
          buyingClub: null,
          saleFeeEur: null,
        });
      }
      return setAcademyMemberOutcome(
        academyClassId,
        activeTarget.playerUid,
        null,
      );
    },
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: academyKeys.classes() }),
        queryClient.invalidateQueries({
          queryKey: academyKeys.academyClass(academyClassId),
        }),
      ]);
      onClose();
    },
  });
  const { reset } = mutation;

  useEffect(() => {
    if (!target || !mode) {
      return;
    }
    setVisibleTarget(target);
    setVisibleMode(mode);
    setVisibleReturnFocusTo(returnFocusTo);
    const existingSale =
      target.outcome?.status === "sold" ? target.outcome : null;
    setBuyingClub(existingSale?.buyingClub ?? "");
    setSaleFeeEur(existingSale?.saleFeeEur?.toString() ?? "");
    setSuggestionsOpen(false);
    setActiveSuggestion(0);
    setValidationError(null);
    reset();
  }, [mode, reset, returnFocusTo, target]);

  const matches = useMemo(() => {
    const query = buyingClub.trim().toLowerCase();
    if (activeMode !== "sale" || query.length === 0) {
      return [];
    }
    return Array.from(new Set(clubOptions))
      .filter((club) => club.toLowerCase().includes(query))
      .slice(0, CLUB_SUGGEST_LIMIT);
  }, [activeMode, buyingClub, clubOptions]);
  const activeClub = matches[activeSuggestion];
  const showSuggestions = suggestionsOpen && matches.length > 0;
  const error =
    validationError ?? (mutation.isError ? errorMessage(mutation.error) : null);
  const pendingMode = mutation.isPending ? mutation.variables : null;

  useEffect(() => {
    if (!activeClub) {
      return;
    }
    activeOptionRef.current?.scrollIntoView?.({ block: "nearest" });
  }, [activeClub]);

  if (!activeTarget || !activeMode) {
    return null;
  }

  const title =
    activeMode === "sale"
      ? `${activeTarget.outcome?.status === "sold" ? "Edit" : "Record"} sale for ${name}`
      : activeMode === "released"
        ? `Mark ${name} as released?`
        : `Restore ${name} to still at club?`;
  const submitLabel =
    activeMode === "sale"
      ? "Save sale"
      : activeMode === "released"
        ? "Mark released"
        : "Restore player";
  const pendingLabel =
    activeMode === "sale"
      ? "Saving…"
      : activeMode === "released"
        ? "Marking…"
        : "Restoring…";
  const restoreExistingSale =
    activeMode === "sale" && activeTarget.outcome?.status === "sold";

  const selectClub = (club: string) => {
    setBuyingClub(club);
    setSuggestionsOpen(false);
    setActiveSuggestion(0);
    setValidationError(null);
    reset();
  };

  return (
    <Modal
      open={isOpen}
      title={title}
      returnFocusTo={visibleReturnFocusTo}
      fallbackFocusTo={() =>
        document.querySelector<HTMLButtonElement>(
          `[data-academy-member-sell="${academyClassId}-${activeTarget.playerUid}"]`,
        )
      }
      onClose={() => {
        if (!mutation.isPending) {
          onClose();
        }
      }}
      footer={
        <>
          {restoreExistingSale ? (
            <Button
              disabled={mutation.isPending}
              loading={pendingMode === "clear"}
              loadingLabel="Restoring…"
              variant="secondary"
              onClick={() => mutation.mutate("clear")}
            >
              Restore to still at club
            </Button>
          ) : null}
          <Button
            disabled={mutation.isPending}
            variant="secondary"
            onClick={onClose}
          >
            Cancel
          </Button>
          <Button
            type="submit"
            form="academy-member-outcome-form"
            disabled={pendingMode === "clear"}
            loading={mutation.isPending && pendingMode !== "clear"}
            loadingLabel={pendingLabel}
          >
            {submitLabel}
          </Button>
        </>
      }
    >
      <form
        id="academy-member-outcome-form"
        className="space-y-4"
        onSubmit={(event) => {
          event.preventDefault();
          if (activeMode === "sale") {
            if (!buyingClub.trim()) {
              setValidationError("Enter the buying club");
              return;
            }
            const parsedFee = Number(saleFeeEur);
            if (
              !saleFeeEur.trim() ||
              !Number.isInteger(parsedFee) ||
              parsedFee < 0
            ) {
              setValidationError("Enter a non-negative whole-euro fee");
              return;
            }
          }
          setValidationError(null);
          mutation.mutate(activeMode);
        }}
      >
        {activeMode === "sale" ? (
          <>
            <p className="text-body-md text-on-surface-variant">
              Record the buying club and agreed transfer fee. This manual entry
              stays with the Academy membership.
            </p>
            <div className="relative">
              <TextField
                aria-activedescendant={
                  showSuggestions
                    ? `${optionIdPrefix}-${activeSuggestion}`
                    : undefined
                }
                aria-autocomplete="list"
                aria-controls={showSuggestions ? listboxId : undefined}
                aria-expanded={showSuggestions}
                aria-haspopup="listbox"
                autoComplete="off"
                disabled={mutation.isPending}
                label="Buying club"
                role="combobox"
                type="text"
                value={buyingClub}
                onBlur={() => {
                  window.setTimeout(() => setSuggestionsOpen(false), 150);
                }}
                onChange={(event) => {
                  setBuyingClub(event.target.value);
                  setSuggestionsOpen(true);
                  setActiveSuggestion(0);
                  setValidationError(null);
                  reset();
                }}
                onFocus={() => setSuggestionsOpen(true)}
                onKeyDown={(event) => {
                  if (event.key === "Escape") {
                    event.preventDefault();
                    event.stopPropagation();
                    setSuggestionsOpen(false);
                    return;
                  }
                  if (event.key === "Enter" && showSuggestions && activeClub) {
                    event.preventDefault();
                    selectClub(activeClub);
                    return;
                  }
                  if (!showSuggestions) {
                    return;
                  }
                  if (event.key === "ArrowDown") {
                    event.preventDefault();
                    setActiveSuggestion(
                      (index) => (index + 1) % matches.length,
                    );
                  } else if (event.key === "ArrowUp") {
                    event.preventDefault();
                    setActiveSuggestion(
                      (index) => (index - 1 + matches.length) % matches.length,
                    );
                  }
                }}
              />
              {showSuggestions ? (
                <div
                  aria-label="Buying club suggestions"
                  className="absolute z-20 mt-1 max-h-64 w-full overflow-auto rounded-lg border border-outline-variant bg-surface-container-highest py-1 shadow-overlay"
                  id={listboxId}
                  role="listbox"
                >
                  {matches.map((club, index) => (
                    <button
                      key={club}
                      ref={club === activeClub ? activeOptionRef : undefined}
                      aria-selected={index === activeSuggestion}
                      className={`flex w-full cursor-pointer px-3 py-2 text-left text-body-sm text-on-surface hover:bg-surface-container-high ${
                        index === activeSuggestion
                          ? "bg-surface-container-high"
                          : ""
                      }`}
                      id={`${optionIdPrefix}-${index}`}
                      role="option"
                      type="button"
                      onMouseDown={(event) => event.preventDefault()}
                      onMouseEnter={() => setActiveSuggestion(index)}
                      onClick={() => selectClub(club)}
                    >
                      {club}
                    </button>
                  ))}
                </div>
              ) : null}
            </div>
            <TextField
              disabled={mutation.isPending}
              label="Sale fee (€)"
              min={0}
              step={1}
              type="number"
              value={saleFeeEur}
              onChange={(event) => {
                setSaleFeeEur(event.target.value);
                setValidationError(null);
                reset();
              }}
            />
          </>
        ) : (
          <p className="text-body-md text-on-surface-variant">
            {activeMode === "released"
              ? "This keeps the player in their class but marks the outcome as released."
              : "This clears the manual outcome while keeping the player in their class."}
          </p>
        )}
        {error ? (
          <p className="text-body-sm text-error" role="alert">
            {error}
          </p>
        ) : null}
      </form>
    </Modal>
  );
}
