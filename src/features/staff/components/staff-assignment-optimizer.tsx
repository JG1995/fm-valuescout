import { useMutation, useQuery } from "@tanstack/react-query";
import { ArrowLeftRight } from "lucide-react";
import { useCallback, useLayoutEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { Modal } from "@/components/ui/modal/modal";
import { optimizeStaffAssignments } from "../api/optimize-staff-assignments";
import { setStaffAssignmentSkip } from "../api/set-staff-assignment-skip";
import { staffAssignmentTargetsQueryOptions } from "../api/staff-assignment-targets-query-options";
import type {
  StaffAssignmentContext,
  StaffAssignmentOptimization,
} from "../types/staff-assignment";
import { StaffAssignmentResults } from "./staff-assignment-results";
import {
  type StaffAssignmentModalStatus,
  StaffAssignmentTargetModal,
} from "./staff-assignment-target-modal";
import { StaffFmChanges } from "./staff-fm-changes";

type StaffAssignmentOptimizerProps = {
  context: StaffAssignmentContext;
  contextKey: string;
  contextUnavailable: boolean;
  shortlistReady: boolean;
  onReviewShortlist?: () => void;
  onOpenStaff?: (uid: number) => void;
};

type OptimizeRequest = {
  contextKey: string;
  generation: number;
  saveContextToken: string;
  snapshotContextToken: string;
};

type SkipRequest = {
  contextKey: string;
  generation: number;
  saveContextToken: string;
  uid: number;
  skipped: boolean;
};

type PresentedResult = {
  contextKey: string;
  value: StaffAssignmentOptimization;
};

const NO_SHORTLIST_MESSAGE =
  "Shortlist staff in FM, save the game, then use Load Data.";

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

function canPresentResult(
  result: StaffAssignmentOptimization,
  context: StaffAssignmentContext,
) {
  switch (result.state) {
    case "ready":
    case "no_managed_club":
    case "no_shortlist":
      return (
        result.saveContextToken === context.saveContextToken &&
        result.snapshotContextToken === context.snapshotContextToken
      );
    case "no_current_snapshot":
      return (
        result.saveContextToken === context.saveContextToken &&
        result.snapshotContextToken === null
      );
    case "stale_context":
      return true;
  }
}

function setupMessage(result: StaffAssignmentOptimization) {
  switch (result.state) {
    case "stale_context":
      return "Your save changed while optimizing. Optimize again.";
    case "no_current_snapshot":
      return "This save has no data yet. Use Load Data first.";
    case "no_managed_club":
      return "Choose your club in My Club first.";
    case "no_shortlist":
      return NO_SHORTLIST_MESSAGE;
    case "ready":
      return null;
  }
}

export function StaffAssignmentOptimizer({
  context,
  contextKey,
  contextUnavailable,
  shortlistReady,
  onReviewShortlist = () => {},
  onOpenStaff = () => {},
}: StaffAssignmentOptimizerProps) {
  const targetsQuery = useQuery(
    staffAssignmentTargetsQueryOptions(context, contextKey),
  );
  const [modalOpen, setModalOpen] = useState(false);
  const [resultsOpen, setResultsOpen] = useState(false);
  const [fmChangesOpen, setFmChangesOpen] = useState(false);
  const [resultsReturnFocus, setResultsReturnFocus] =
    useState<HTMLElement | null>(null);
  // Saving staffing needs opened from Optimize or from the results optimizes again.
  const optimizeAfterSave = useRef(false);
  const currentContext = useRef(context);
  const currentContextKey = useRef(contextKey);
  const previousContextKey = useRef(contextKey);
  const previousContextUnavailable = useRef(contextUnavailable);
  const requestGeneration = useRef(0);
  const [targetSavePending, setTargetSavePending] = useState(false);
  const [result, setResult] = useState<PresentedResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [modalStatus, setModalStatus] = useState<StaffAssignmentModalStatus>({
    saved: false,
    error: null,
  });
  currentContext.current = context;
  currentContextKey.current = contextKey;

  const optimize = useMutation({
    mutationFn: (request: OptimizeRequest) =>
      optimizeStaffAssignments(
        request.saveContextToken,
        request.snapshotContextToken,
      ),
    onSuccess: (nextResult, request) => {
      if (
        request.contextKey !== currentContextKey.current ||
        request.generation !== requestGeneration.current ||
        !canPresentResult(nextResult, currentContext.current)
      ) {
        return;
      }
      setResult({ contextKey: request.contextKey, value: nextResult });
    },
    onError: (nextError, request) => {
      if (
        request.contextKey === currentContextKey.current &&
        request.generation === requestGeneration.current
      ) {
        setError(errorMessage(nextError));
      }
    },
  });

  // A skip or restore changes the pool, so the optimization runs again. The visible result
  // stays until the new one arrives; a context reset meanwhile drops both.
  const skip = useMutation({
    mutationFn: (request: SkipRequest) =>
      setStaffAssignmentSkip(
        request.saveContextToken,
        request.uid,
        request.skipped,
      ),
    onSuccess: (_, request) => {
      if (
        request.contextKey !== currentContextKey.current ||
        request.generation !== requestGeneration.current
      ) {
        return;
      }
      optimize.mutate({
        contextKey: request.contextKey,
        generation: request.generation,
        saveContextToken: currentContext.current.saveContextToken,
        snapshotContextToken: currentContext.current.snapshotContextToken,
      });
    },
    onError: (nextError, request) => {
      if (
        request.contextKey === currentContextKey.current &&
        request.generation === requestGeneration.current
      ) {
        setError(errorMessage(nextError));
      }
    },
  });

  const resetOptimize = optimize.reset;
  const resetOutcome = useCallback(() => {
    requestGeneration.current += 1;
    setResult(null);
    setError(null);
    setModalStatus({ saved: false, error: null });
    setResultsOpen(false);
    resetOptimize();
  }, [resetOptimize]);

  const runOptimize = () => {
    resetOutcome();
    setResultsOpen(true);
    optimize.mutate({
      contextKey: currentContextKey.current,
      generation: requestGeneration.current,
      saveContextToken: currentContext.current.saveContextToken,
      snapshotContextToken: currentContext.current.snapshotContextToken,
    });
  };

  const handleModalStatusChange = useCallback(
    (status: StaffAssignmentModalStatus) => {
      setModalStatus(status);
    },
    [],
  );

  useLayoutEffect(() => {
    const contextChanged = previousContextKey.current !== contextKey;
    const contextBecameUnavailable =
      contextUnavailable && !previousContextUnavailable.current;
    previousContextKey.current = contextKey;
    previousContextUnavailable.current = contextUnavailable;
    if (contextChanged || contextBecameUnavailable) {
      resetOutcome();
    }
  }, [contextKey, contextUnavailable, resetOutcome]);

  const configuredSlotCount = (targetsQuery.data?.targets ?? []).reduce(
    (total, target) => total + target.slotCount,
    0,
  );
  const optimizeDisabledReason = contextUnavailable
    ? "Updating your save data…"
    : targetsQuery.isPending
      ? "Loading staffing needs…"
      : targetsQuery.isError
        ? "Couldn't load staffing needs. Try again."
        : !shortlistReady
          ? NO_SHORTLIST_MESSAGE
          : null;
  const needsSummary = optimizeDisabledReason
    ? null
    : configuredSlotCount === 0
      ? "No staffing needs set yet."
      : `Staffing needs: ${configuredSlotCount} ${configuredSlotCount === 1 ? "slot" : "slots"}.`;
  const currentResult =
    !contextUnavailable &&
    result?.contextKey === contextKey &&
    canPresentResult(result.value, context)
      ? result.value
      : null;
  const message = currentResult ? setupMessage(currentResult) : null;
  const readyResult = currentResult?.state === "ready" ? currentResult : null;
  const resultsVisible = resultsOpen && readyResult !== null;
  // The results dialog reports its own skip errors.
  const statusError = modalStatus.error ?? (resultsVisible ? null : error);
  const statusSuccess = modalStatus.saved;

  return (
    <div className="flex w-full flex-wrap items-center justify-end gap-x-3 gap-y-1">
      <div
        data-testid="assignment-status-region"
        role={!statusError && (message || statusSuccess) ? "status" : undefined}
        className="min-w-0 text-right text-body-sm break-words"
      >
        {!contextUnavailable && statusError ? (
          <p role="alert" className="text-error">
            {statusError}
          </p>
        ) : null}
        {statusSuccess ? (
          <p className="text-success">Staffing needs saved.</p>
        ) : null}
        {message ? <p className="text-on-surface-variant">{message}</p> : null}
      </div>
      {optimizeDisabledReason || needsSummary ? (
        <p
          id="assignment-readiness"
          className="text-right text-body-sm text-on-surface-variant"
        >
          {optimizeDisabledReason ?? needsSummary}
        </p>
      ) : null}
      <div data-testid="assignment-action-row" className="flex gap-2">
        <StaffAssignmentTargetModal
          context={context}
          contextKey={contextKey}
          targets={targetsQuery.data}
          targetsError={
            targetsQuery.error instanceof Error ? targetsQuery.error : null
          }
          targetsPending={targetsQuery.isPending || targetsQuery.isFetching}
          open={modalOpen}
          onOpenChange={(open) => {
            // The Configure button opens the dialog through here; it only saves.
            if (open) {
              optimizeAfterSave.current = false;
            }
            setModalOpen(open);
          }}
          onSaved={(saved) => {
            const slotCount = saved.targets.reduce(
              (total, target) => total + target.slotCount,
              0,
            );
            if (optimizeAfterSave.current && slotCount > 0) {
              runOptimize();
            } else {
              resetOutcome();
            }
            optimizeAfterSave.current = false;
          }}
          onPendingChange={(pending) => {
            setTargetSavePending(pending);
            if (pending) {
              resetOutcome();
            }
          }}
          onStatusChange={handleModalStatusChange}
        />
        <Button
          aria-describedby={
            optimizeDisabledReason || needsSummary
              ? "assignment-readiness"
              : undefined
          }
          disabled={Boolean(optimizeDisabledReason) || targetSavePending}
          loading={optimize.isPending}
          loadingLabel="Optimizing…"
          onClick={(event) => {
            setResultsReturnFocus(event.currentTarget);
            if (configuredSlotCount === 0) {
              optimizeAfterSave.current = true;
              setModalOpen(true);
              return;
            }
            runOptimize();
          }}
        >
          Optimize assignments
        </Button>
      </div>
      <Modal
        open={resultsVisible}
        onClose={() => setResultsOpen(false)}
        title="Assignment recommendations"
        variant="informational"
        className="max-w-[64rem]!"
        returnFocusTo={resultsReturnFocus}
        footer={
          <Button
            variant="secondary"
            icon={ArrowLeftRight}
            onClick={() => {
              setResultsOpen(false);
              setFmChangesOpen(true);
            }}
          >
            Apply in FM
          </Button>
        }
      >
        {error ? (
          <p role="alert" className="pb-3 text-body-sm text-error">
            {error}
          </p>
        ) : null}
        {readyResult ? (
          <StaffAssignmentResults
            result={readyResult}
            onRequestConfiguration={() => {
              setResultsOpen(false);
              optimizeAfterSave.current = true;
              setModalOpen(true);
            }}
            onReviewShortlist={() => {
              setResultsOpen(false);
              onReviewShortlist();
            }}
            onOpenStaff={(uid) => {
              setResultsOpen(false);
              onOpenStaff(uid);
            }}
            onSetSkip={(uid, skipped) => {
              setError(null);
              skip.mutate({
                contextKey,
                generation: requestGeneration.current,
                saveContextToken: context.saveContextToken,
                uid,
                skipped,
              });
            }}
            skipDisabled={skip.isPending || optimize.isPending}
          />
        ) : null}
      </Modal>
      <StaffFmChanges
        context={context}
        contextKey={contextKey}
        open={fmChangesOpen}
        onClose={() => setFmChangesOpen(false)}
        onApplied={resetOutcome}
      />
    </div>
  );
}
