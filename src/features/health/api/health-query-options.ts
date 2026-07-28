import { queryOptions } from "@tanstack/react-query";
import { fetchHealthStatus } from "./fetch-health-status";
import { healthKeys } from "./health-keys";

export const healthQueryOptions = queryOptions({
  queryKey: healthKeys.status(),
  queryFn: fetchHealthStatus,
});
