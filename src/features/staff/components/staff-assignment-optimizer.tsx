import { useMutation, useQuery } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { useCallback, useLayoutEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { optimizeStaffAssignments } from "../api/optimize-staff-assignments";
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

type StaffAssignmentOptimizerProps = {
  context: StaffAssignmentContext;
  contextKey: string;
  contextUnavailable: boolean;
  shortlistReady: boolean;
  uploadAction?: ReactNode;
  onReviewShortlist?: () => void;
};

type OptimizeRequest = {
  contextKey: string;
  generation: number;
  saveContextToken: string;
  snapshotContextToken: string;
};

type PresentedResult = {
  contextKey: string;
  value: StaffAssignmentOptimization;
};

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
      return "Assignment context changed. Refresh the current save context before optimizing again.";
    case "no_current_snapshot":
      return "No current snapshot is available for this save. Use Load Data before optimizing assignments.";
    case "no_managed_club":
      return "Choose a managed club before optimizing assignments.";
    case "no_shortlist":
      return "Upload a Staff Shortlist before optimizing assignments.";
    case "ready":
      return null;
  }
}

export function StaffAssignmentOptimizer({
  context,
  contextKey,
  contextUnavailable,
  shortlistReady,
  uploadAction,
  onReviewShortlist = () => {},
}: StaffAssignmentOptimizerProps) {
  const targetsQuery = useQuery(
    staffAssignmentTargetsQueryOptions(context, contextKey),
  );
  const [modalOpen, setModalOpen] = useState(false);
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

  const resetOptimize = optimize.reset;
  const resetOutcome = useCallback(() => {
    requestGeneration.current += 1;
    setResult(null);
    setError(null);
    setModalStatus({ saved: false, error: null });
    resetOptimize();
  }, [resetOptimize]);

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
    ? "Assignment context is refreshing."
    : targetsQuery.isPending
      ? "Loading staffing needs."
      : targetsQuery.isError
        ? "Staffing needs could not be loaded. Try again."
        : configuredSlotCount === 0
          ? "Configure staffing needs before optimizing assignments."
          : !shortlistReady
            ? "Upload a Staff Shortlist before optimizing assignments."
            : null;
  const currentResult =
    !contextUnavailable &&
    result?.contextKey === contextKey &&
    canPresentResult(result.value, context)
      ? result.value
      : null;
  const message = currentResult ? setupMessage(currentResult) : null;
  const statusError = modalStatus.error ?? error;
  const statusSuccess = modalStatus.saved;

  return (
    <div className="w-full">
      <div
        data-testid="assignment-action-row"
        className="flex w-full flex-wrap items-center justify-end gap-2"
      >
        {uploadAction}
        <StaffAssignmentTargetModal
          context={context}
          contextKey={contextKey}
          targets={targetsQuery.data}
          targetsError={
            targetsQuery.error instanceof Error ? targetsQuery.error : null
          }
          targetsPending={targetsQuery.isPending || targetsQuery.isFetching}
          open={modalOpen}
          onOpenChange={setModalOpen}
          onSaved={resetOutcome}
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
            optimizeDisabledReason ? "assignment-readiness" : undefined
          }
          disabled={Boolean(optimizeDisabledReason) || targetSavePending}
          loading={optimize.isPending}
          loadingLabel="Optimizing…"
          onClick={() => {
            resetOutcome();
            optimize.mutate({
              contextKey,
              generation: requestGeneration.current,
              saveContextToken: context.saveContextToken,
              snapshotContextToken: context.snapshotContextToken,
            });
          }}
        >
          Optimize assignments
        </Button>
      </div>
      {optimizeDisabledReason ? (
        <p
          id="assignment-readiness"
          className="mt-2 w-full text-body-sm text-on-surface-variant"
        >
          {optimizeDisabledReason}
        </p>
      ) : null}
      <div
        data-testid="assignment-status-region"
        role={statusError || message || statusSuccess ? "status" : undefined}
        className="mt-2 w-full min-h-6 text-body-sm"
      >
        {!contextUnavailable && statusError ? (
          <p
            role="alert"
            className="w-full shrink-0 basis-full text-body-sm text-error"
          >
            {statusError}
          </p>
        ) : null}
        {statusSuccess ? (
          <p className="w-full shrink-0 basis-full text-body-sm text-success">
            Slot counts saved.
          </p>
        ) : null}
        {message ? (
          <p className="w-full shrink-0 basis-full text-body-md text-on-surface-variant">
            {message}
          </p>
        ) : null}
      </div>
      {currentResult?.state === "ready" ? (
        <StaffAssignmentResults
          result={currentResult}
          onRequestConfiguration={() => setModalOpen(true)}
          onReviewShortlist={onReviewShortlist}
        />
      ) : null}
    </div>
  );
}
