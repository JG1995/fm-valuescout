import type { QueryClient } from "@tanstack/react-query";
import { searchKeys } from "@/features/search/api/search-keys";
import { squadKeys } from "@/features/squad/api/squad-keys";

export async function clearPlayerResultContext(
  queryClient: QueryClient,
  guard?: () => boolean,
) {
  await Promise.all([
    queryClient.cancelQueries({ queryKey: searchKeys.playerPages() }),
    queryClient.cancelQueries({ queryKey: squadKeys.playerPages() }),
  ]);
  if (guard && !guard()) return;
  queryClient.removeQueries({ queryKey: searchKeys.playerPages() });
  queryClient.removeQueries({ queryKey: squadKeys.playerPages() });
}
