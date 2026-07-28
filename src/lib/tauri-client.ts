import { invoke } from "@tauri-apps/api/core";

export class TauriCommandError extends Error {
  readonly kind?: string;

  constructor(message: string, kind?: string) {
    super(message);
    this.name = "TauriCommandError";
    this.kind = kind;
  }
}

export async function invokeCommand<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    if (typeof error === "object" && error !== null && "kind" in error) {
      const structured = error as { kind: string; message?: string };
      throw new TauriCommandError(
        structured.message ?? structured.kind,
        structured.kind,
      );
    }
    throw new TauriCommandError(String(error));
  }
}
