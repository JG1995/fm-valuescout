import { useMutation } from "@tanstack/react-query";
import { useCallback, useLayoutEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { optimizeStaffAssignments } from "../api/optimize-staff-assignments";
import type {
  StaffAssignmentContext,
  StaffAssignmentOptimization,
} from "../types/staff-assignment";
import { StaffAssignmentResults } from "./staff-assignment-results";
import { StaffAssignmentTargetModal } from "./staff-assignment-target-modal";

type StaffAssignmentOptimizerProps = {
  context: StaffAssignmentContext;
  contextKey: string;
  contextUnavailable: boolean;
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
}: StaffAssignmentOptimizerProps) {
  const currentContext = useRef(context);
  const currentContextKey = useRef(contextKey);
  const previousContextKey = useRef(contextKey);
  const previousContextUnavailable = useRef(contextUnavailable);
  const requestGeneration = useRef(0);
  const [targetSavePending, setTargetSavePending] = useState(false);
  const [result, setResult] = useState<PresentedResult | null>(null);
  const [error, setError] = useState<string | null>(null);
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
    resetOptimize();
  }, [resetOptimize]);

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

  const currentResult =
    !contextUnavailable &&
    result?.contextKey === contextKey &&
    canPresentResult(result.value, context)
      ? result.value
      : null;
  const message = currentResult ? setupMessage(currentResult) : null;

  return (
    <div className="contents">
      <div className="flex flex-wrap items-center gap-2">
        <StaffAssignmentTargetModal
          context={context}
          contextKey={contextKey}
          onSaved={resetOutcome}
          onPendingChange={(pending) => {
            setTargetSavePending(pending);
            if (pending) {
              resetOutcome();
            }
          }}
        />
        <Button
          disabled={contextUnavailable || targetSavePending}
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
      {!contextUnavailable && error ? (
        <p role="alert" className="text-body-sm text-error">
          {error}
        </p>
      ) : null}
      {message ? (
        <p role="status" className="text-body-md text-on-surface-variant">
          {message}
        </p>
      ) : null}
      {currentResult?.state === "ready" ? (
        <StaffAssignmentResults result={currentResult} />
      ) : null}
    </div>
  );
}
