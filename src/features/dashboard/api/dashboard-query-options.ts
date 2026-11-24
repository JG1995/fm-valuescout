import { queryOptions } from "@tanstack/react-query";
import { invokeCommand } from "@/lib/tauri-client";
import type {
  DashboardContext,
  DashboardExpiringContracts,
  DashboardProspects,
  DashboardSignings,
  DashboardSigningsRequest,
} from "../types/dashboard";
import { dashboardKeys } from "./dashboard-keys";

function sameContext(left: unknown, right: DashboardContext) {
  const context = left as Partial<DashboardContext> | undefined;
  return (
    context?.saveId === right.saveId &&
    context.contextToken === right.contextToken &&
    context.snapshotId === right.snapshotId &&
    context.clubName === right.clubName
  );
}

// Shortlists and season stats change on other pages without invalidating
// Planner, so every visit to the dashboard reads again.
const staleTime = 0;

export function dashboardSigningsQueryOptions(
  context: DashboardContext,
  request: DashboardSigningsRequest,
) {
  return queryOptions({
    queryKey: dashboardKeys.signings(context, request),
    queryFn: () =>
      invokeCommand<DashboardSignings | null>("get_dashboard_signings", {
        mode: request.mode,
        shortlistId: request.shortlistId,
        laneId: request.laneId,
        seasonStats: request.seasonStats,
      }),
    staleTime,
    // A mode, shortlist, slot, or season-stats change keeps the current rows until the
    // replacement arrives. Another save, snapshot, or club never shows them.
    placeholderData: (previousData, previousQuery) =>
      sameContext(previousQuery?.queryKey[3], context)
        ? previousData
        : undefined,
  });
}

/**
 * The signings read before a shortlist or slot is chosen: what the panel
 * requests first, so the route loader warms exactly that read.
 */
export function dashboardFirstSigningsQueryOptions(
  context: DashboardContext,
  mode: DashboardSigningsRequest["mode"],
  seasonStats: boolean,
) {
  return dashboardSigningsQueryOptions(context, {
    mode,
    shortlistId: null,
    laneId: null,
    seasonStats,
  });
}

export function dashboardExpiringContractsQueryOptions(
  context: DashboardContext,
) {
  return queryOptions({
    queryKey: dashboardKeys.expiringContracts(context),
    queryFn: () =>
      invokeCommand<DashboardExpiringContracts | null>(
        "get_dashboard_expiring_contracts",
      ),
    staleTime,
  });
}

export function dashboardProspectsQueryOptions(context: DashboardContext) {
  return queryOptions({
    queryKey: dashboardKeys.prospects(context),
    queryFn: () =>
      invokeCommand<DashboardProspects | null>("get_dashboard_prospects"),
    staleTime,
  });
}
