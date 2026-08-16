import { queryOptions } from "@tanstack/react-query";
import { fetchStaffDetail } from "./fetch-staff";
import { staffKeys } from "./staff-keys";

export function getStaffQueryOptions(uid: number) {
  return queryOptions({
    queryKey: staffKeys.detail(uid),
    queryFn: () => fetchStaffDetail(uid),
  });
}
