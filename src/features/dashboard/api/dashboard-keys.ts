import type {
  DashboardContext,
  DashboardSigningsRequest,
} from "../types/dashboard";

export const dashboardKeys = {
  // Under Planner's root, like Squad: the tactic, the first string, the
  // managed club, and Load Data all change these reads, and each of them
  // already invalidates Planner.
  all: ["planner", "dashboard"] as const,
  signings: (context: DashboardContext, request: DashboardSigningsRequest) =>
    [...dashboardKeys.all, "signings", context, request] as const,
  expiringContracts: (context: DashboardContext) =>
    [...dashboardKeys.all, "expiring-contracts", context] as const,
  prospects: (context: DashboardContext) =>
    [...dashboardKeys.all, "prospects", context] as const,
};
