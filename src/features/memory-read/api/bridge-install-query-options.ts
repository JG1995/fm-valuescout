import { queryOptions } from "@tanstack/react-query";
import { bridgeInstallKeys } from "./bridge-install-keys";
import { fetchBridgeInstallStatus } from "./fetch-bridge-install-status";

export const bridgeInstallQueryOptions = queryOptions({
  queryKey: bridgeInstallKeys.status(),
  queryFn: fetchBridgeInstallStatus,
});
