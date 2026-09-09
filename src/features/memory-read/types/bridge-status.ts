export type BridgeStatus = {
  protocolVersion: number;
  pluginVersion: string;
  state: string;
  updatedAtUtc: string;
  gamePluginModulePresent: boolean;
  gameAssemblyModulePresent: boolean;
  requestId?: string | null;
  playersFound?: number | null;
  error?: string | null;
};

export type DumpRequestResult = {
  requestId: string;
  state: string;
  playersFound?: number | null;
  dumpPresent: boolean;
  error?: string | null;
};
