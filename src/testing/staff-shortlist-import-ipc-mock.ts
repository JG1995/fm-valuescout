import type { StaffShortlistImportSummary } from "@/features/staff/components/staff-shortlist-import-modal";

export type StaffShortlistImportIpcMockMode = "success" | "error";

const DEFAULT_SUMMARY: StaffShortlistImportSummary = {
  totalStaff: 2,
  storedStaff: 2,
  skippedStaff: 0,
};

let mode: StaffShortlistImportIpcMockMode = "success";
let summary = DEFAULT_SUMMARY;
let error: unknown = new Error(
  "CSV does not contain staff in the current snapshot",
);
let lastArgs: unknown;

export function resetStaffShortlistImportIpcMock() {
  mode = "success";
  summary = DEFAULT_SUMMARY;
  error = new Error("CSV does not contain staff in the current snapshot");
  lastArgs = undefined;
}

export function setStaffShortlistImportIpcMockResult(
  nextSummary: StaffShortlistImportSummary,
) {
  mode = "success";
  summary = nextSummary;
}

export function setStaffShortlistImportIpcMockError(nextError: unknown) {
  mode = "error";
  error = nextError;
}

export function getLastStaffShortlistImportIpcArgs() {
  return lastArgs;
}

export function resolveStaffShortlistImportIpcMock(args: unknown) {
  lastArgs = args;

  if (mode === "error") {
    return Promise.reject(error);
  }

  return Promise.resolve(summary);
}
