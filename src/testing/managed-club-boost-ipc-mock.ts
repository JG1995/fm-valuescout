import type { InvokeArgs } from "@tauri-apps/api/core";

type ManagedClubBoostIpcArgs = {
  onProgress?: {
    onmessage?: (progress: Record<string, unknown>) => void;
  };
};

let calls: Array<InvokeArgs | undefined> = [];
let errorMessage: string | null = null;
let recoveryRequired = false;

export function resetManagedClubBoostIpcMock() {
  calls = [];
  errorMessage = null;
  recoveryRequired = false;
}

export function setManagedClubBoostError(message: string) {
  errorMessage = message;
}

export function setManagedClubBoostRecoveryRequired(value: boolean) {
  recoveryRequired = value;
}

export function getManagedClubBoostIpcMockCalls() {
  return calls;
}

export function resolveManagedClubBoostIpcMock(args: InvokeArgs | undefined) {
  if (!args || !("onProgress" in args)) {
    throw new Error("Managed club boost progress channel is required");
  }
  calls = [...calls, args];
  if (errorMessage) {
    return Promise.reject({
      phase: "eligibility",
      kind: "noSnapshot",
      message: errorMessage,
    });
  }

  const { onProgress } = args as ManagedClubBoostIpcArgs;
  if (recoveryRequired) {
    onProgress?.onmessage?.({
      phase: "wonderkids",
      processed: 1,
      total: 3,
      updated: 1,
      skipped: 0,
      failed: 0,
    });
    return Promise.resolve({
      wonderkids: {
        updated: 1,
        skipped: 0,
        failed: 0,
        recoveryRequired: true,
        recoveryMessage: "Load Data again",
      },
      playerCurrentAbility: null,
      staffCurrentAbility: null,
    });
  }

  onProgress?.onmessage?.({
    phase: "staffCurrentAbility",
    processed: 3,
    total: 3,
    updated: 3,
    skipped: 0,
    failed: 0,
  });
  return Promise.resolve({
    wonderkids: {
      updated: 2,
      skipped: 0,
      failed: 0,
      recoveryRequired: false,
      recoveryMessage: null,
    },
    playerCurrentAbility: {
      updated: 2,
      skipped: 1,
      failed: 0,
      recoveryRequired: false,
      recoveryMessage: null,
    },
    staffCurrentAbility: {
      updated: 3,
      skipped: 1,
      failed: 0,
      recoveryRequired: false,
      recoveryMessage: null,
    },
  });
}
