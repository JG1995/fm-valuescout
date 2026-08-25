import { useMutation, useQueryClient } from "@tanstack/react-query";
import {
  type ClearPlayerResultContext,
  playerResultContextMutationKey,
} from "@/components/player-table/player-result-context";
import { bridgeStatusQueryOptions } from "../api/bridge-status-query-options";
import { loadData } from "../api/load-data";

type UseLoadDataOptions = {
  /**
   * Invalidate caches this feature must not import. Snapshot query keys belong
   * to another feature, so the composing route or shell passes them in.
   */
  onSettled?: () => void;
  onBeforeContextChange: ClearPlayerResultContext;
};

export function useLoadData({
  onSettled,
  onBeforeContextChange,
}: UseLoadDataOptions) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: playerResultContextMutationKey,
    mutationFn: async (maxAccepted: number | null) => {
      await onBeforeContextChange();
      return loadData(maxAccepted);
    },
    onSettled: () => {
      void queryClient.invalidateQueries({
        queryKey: bridgeStatusQueryOptions.queryKey,
      });
      onSettled?.();
    },
  });
}
