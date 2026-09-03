import { useQuery } from "@tanstack/react-query";
import type { PlannerContext } from "../api/planner-keys";
import { plannerTacticOptionsQueryOptions } from "../api/planner-tactic-options-query-options";
import { plannerTacticQueryOptions } from "../api/planner-tactic-query-options";
import type { PlannerTactic, TacticOptions } from "../types/tactic";

export type TacticContextBoundaryState = {
  tactic: PlannerTactic | undefined;
  options: TacticOptions | undefined;
  isPending: boolean;
  initialError: Error | null;
  isRefetchError: boolean;
  refreshError: Error | null;
};

type TacticContextBoundaryProps = {
  context: PlannerContext;
  children: (state: TacticContextBoundaryState) => React.ReactNode;
};

export function TacticContextBoundary({
  context,
  children,
}: TacticContextBoundaryProps) {
  const tacticQuery = useQuery(plannerTacticQueryOptions(context));
  const optionsQuery = useQuery(plannerTacticOptionsQueryOptions(context));

  const isPending =
    (tacticQuery.isPending && tacticQuery.data === undefined) ||
    (optionsQuery.isPending && optionsQuery.data === undefined);

  const hasTacticData = tacticQuery.data !== undefined;
  const hasOptionsData = optionsQuery.data !== undefined;
  const hasData = hasTacticData && hasOptionsData;

  const initialError =
    !hasTacticData && tacticQuery.isError
      ? tacticQuery.error
      : !hasOptionsData && optionsQuery.isError
        ? optionsQuery.error
        : null;

  const isRefetchError =
    (hasData && tacticQuery.isError) || (hasData && optionsQuery.isError);

  const refreshError = isRefetchError
    ? ((tacticQuery.error as Error | null) ??
      (optionsQuery.error as Error | null) ??
      null)
    : null;

  return (
    <>
      {children({
        tactic: tacticQuery.data,
        options: optionsQuery.data,
        isPending: Boolean(isPending),
        initialError,
        isRefetchError: Boolean(isRefetchError),
        refreshError,
      })}
    </>
  );
}
