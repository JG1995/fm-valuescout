import { queryOptions } from "@tanstack/react-query";
import { academyKeys } from "./academy-keys";
import { fetchAcademyCandidates } from "./fetch-academy-candidates";

export function academyCandidatesQueryOptions(search: string) {
  return queryOptions({
    queryKey: academyKeys.candidate(search),
    queryFn: () => fetchAcademyCandidates(search),
  });
}
