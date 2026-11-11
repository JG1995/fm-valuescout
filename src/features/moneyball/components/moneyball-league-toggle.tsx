import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { playerResultContextMutationKey } from "@/components/player-table/player-result-context";
import {
  leagueAdjustmentQueryOptions,
  setMoneyballLeagueAdjustment,
} from "../api/league-adjustment-query-options";

/**
 * Chooses whether Moneyball scores are moved for the strength of each player's
 * league. The choice belongs to the save and changes every Moneyball score the
 * app shows, so the host clears the results that hold the old scores.
 */
export function MoneyballLeagueToggle({
  onBeforeChange,
  onChanged,
}: {
  /** Runs before the choice is saved, to drop results scored the old way. */
  onBeforeChange: () => Promise<void>;
  onChanged: () => void;
}) {
  const queryClient = useQueryClient();
  const { data: adjustment } = useQuery(leagueAdjustmentQueryOptions);
  const change = useMutation({
    mutationKey: playerResultContextMutationKey,
    mutationFn: async (enabled: boolean) => {
      await onBeforeChange();
      return setMoneyballLeagueAdjustment(enabled);
    },
    onSuccess: (saved) => {
      queryClient.setQueryData(leagueAdjustmentQueryOptions.queryKey, saved);
      onChanged();
    },
  });
  if (!adjustment) return null;

  const reference = adjustment.referenceDivision;
  const hint =
    reference === null
      ? "League strength is measured by the next Load Data."
      : `League-adjusted scores show how a player would score in ${reference}.`;
  return (
    <>
      {/* One item, so the label wraps with its buttons. */}
      <span className="inline-flex items-center gap-2">
        <span
          aria-hidden
          className="text-label-md text-on-surface-variant"
          title={hint}
        >
          Scores
        </span>
        <fieldset
          aria-description={hint}
          disabled={reference === null || change.isPending}
          className="inline-flex rounded-md border border-outline bg-surface-container-high p-0.5 disabled:opacity-60"
        >
          <legend className="sr-only">Scores</legend>
          {[true, false].map((enabled) => (
            <button
              key={String(enabled)}
              type="button"
              aria-pressed={adjustment.enabled === enabled}
              className={
                adjustment.enabled === enabled
                  ? "h-8 rounded-sm bg-primary px-3 text-label-md text-on-primary"
                  : "h-8 rounded-sm px-3 text-label-md text-on-surface-variant hover:text-on-surface"
              }
              onClick={() => {
                if (adjustment.enabled !== enabled) change.mutate(enabled);
              }}
            >
              {enabled ? "League-adjusted" : "Raw"}
            </button>
          ))}
        </fieldset>
      </span>
      {change.isError ? (
        <span role="alert" className="text-body-sm text-error">
          The score setting could not be changed. Try again.
        </span>
      ) : null}
    </>
  );
}
