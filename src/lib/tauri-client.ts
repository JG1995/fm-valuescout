import { invoke } from "@tauri-apps/api/core";

export type TauriCommandPhase = "scan" | "ingest";

export class TauriCommandError extends Error {
  readonly kind?: string;
  readonly phase?: TauriCommandPhase;

  constructor(
    message: string,
    options?: { kind?: string; phase?: TauriCommandPhase },
  ) {
    super(message);
    this.name = "TauriCommandError";
    this.kind = options?.kind;
    this.phase = options?.phase;
  }
}

function parseInvokeError(error: unknown): TauriCommandError {
  if (typeof error === "object" && error !== null) {
    const structured = error as {
      phase?: string;
      kind?: string;
      message?: string;
    };

    if (structured.phase === "scan" || structured.phase === "ingest") {
      return new TauriCommandError(structured.message ?? "Load data failed", {
        kind: structured.kind,
        phase: structured.phase,
      });
    }

    if (structured.kind) {
      return new TauriCommandError(structured.message ?? structured.kind, {
        kind: structured.kind,
      });
    }

    if (structured.message) {
      return new TauriCommandError(structured.message);
    }
  }

  return new TauriCommandError(String(error));
}

export async function invokeCommand<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw parseInvokeError(error);
  }
}
