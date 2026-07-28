export type BridgeStatus = {
  protocolVersion: number;
  pluginVersion: string;
  state: string;
  updatedAtUtc: string;
  gamePluginModulePresent: boolean;
  gameAssemblyModulePresent: boolean;
};
