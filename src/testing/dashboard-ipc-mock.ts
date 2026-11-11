import type {
  DashboardExpiringContracts,
  DashboardProspect,
  DashboardSignings,
  DashboardSigningsRequest,
} from "@/features/dashboard/types/dashboard";

type SigningsResolver = (
  request: DashboardSigningsRequest,
) => DashboardSignings | null;

const NO_CONTRACTS: DashboardExpiringContracts = {
  windowMonths: 12,
  total: 0,
  players: [],
};

let signingsResolver: SigningsResolver = () => null;
let signingsRequests: DashboardSigningsRequest[] = [];
let signingsError: string | null = null;
let expiringContracts: DashboardExpiringContracts | null = NO_CONTRACTS;
let prospects: DashboardProspect[] | null = [];

export function resetDashboardIpcMock() {
  signingsResolver = () => null;
  signingsRequests = [];
  signingsError = null;
  expiringContracts = NO_CONTRACTS;
  prospects = [];
}

/** Answers every later `get_dashboard_signings` from the request it receives. */
export function setDashboardSigningsIpcMock(resolver: SigningsResolver) {
  signingsResolver = resolver;
}

export function setDashboardSigningsIpcMockError(message: string | null) {
  signingsError = message;
}

export function getDashboardSigningsIpcRequests() {
  return signingsRequests;
}

export function setDashboardExpiringContractsIpcMock(
  contracts: DashboardExpiringContracts | null,
) {
  expiringContracts = contracts;
}

export function setDashboardProspectsIpcMock(next: DashboardProspect[] | null) {
  prospects = next;
}

export function resolveDashboardSigningsIpcMock(args: unknown) {
  const request = args as DashboardSigningsRequest;
  signingsRequests.push({
    mode: request.mode,
    shortlistId: request.shortlistId ?? null,
    laneId: request.laneId ?? null,
  });
  if (signingsError !== null) {
    throw new Error(signingsError);
  }
  return signingsResolver(request);
}

export function resolveDashboardExpiringContractsIpcMock() {
  return expiringContracts;
}

export function resolveDashboardProspectsIpcMock() {
  return prospects;
}
