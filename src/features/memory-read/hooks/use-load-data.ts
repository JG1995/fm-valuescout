import { useMutation, useQueryClient } from "@tanstack/react-query";
import { bridgeStatusQueryOptions } from "../api/bridge-status-query-options";
import { loadData } from "../api/load-data";

type UseLoadDataOptions = {
  /**
   * Invalidate caches this feature must not import. Snapshot query keys belong
   * to another feature, so the composing route or shell passes them in.
   */
  onSettled?: () => void;
};

export function useLoadData({ onSettled }: UseLoadDataOptions = {}) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: loadData,
    onSettled: () => {
      void queryClient.invalidateQueries({
        queryKey: bridgeStatusQueryOptions.queryKey,
      });
      onSettled?.();
    },
  });
}
