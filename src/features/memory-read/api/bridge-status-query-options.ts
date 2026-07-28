import { queryOptions } from "@tanstack/react-query";
import { bridgeStatusKeys } from "./bridge-status-keys";
import { fetchBridgeStatus } from "./fetch-bridge-status";

export const bridgeStatusQueryOptions = queryOptions({
  queryKey: bridgeStatusKeys.status(),
  queryFn: fetchBridgeStatus,
});
