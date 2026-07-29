import type { BridgeInstallStatus } from "../types/bridge-install";

export type BridgeInstallIpcMockMode =
  | "absent"
  | "installed"
  | "unsupportedPlatform"
  | "bepinexMissing";

const DEFAULT_PLUGINS_PATH =
  "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Football Manager 26\\BepInEx\\plugins";

const DEFAULT_INSTALL_STATUS: BridgeInstallStatus = {
  pluginsPath: DEFAULT_PLUGINS_PATH,
  pluginPresent: false,
  bepinexPresent: true,
  pluginsDirPresent: true,
};

let installMockMode: BridgeInstallIpcMockMode = "absent";
let currentInstallStatus: BridgeInstallStatus = { ...DEFAULT_INSTALL_STATUS };

function applyInstallMockMode(mode: BridgeInstallIpcMockMode) {
  if (mode === "installed") {
    currentInstallStatus = {
      ...DEFAULT_INSTALL_STATUS,
      pluginPresent: true,
    };
    return;
  }

  if (mode === "bepinexMissing") {
    currentInstallStatus = {
      ...DEFAULT_INSTALL_STATUS,
      bepinexPresent: false,
      pluginsDirPresent: false,
    };
    return;
  }

  currentInstallStatus = {
    ...DEFAULT_INSTALL_STATUS,
    pluginPresent: false,
  };
}

export function setBridgeInstallIpcMockMode(mode: BridgeInstallIpcMockMode) {
  installMockMode = mode;
  applyInstallMockMode(mode);
}

export function resetBridgeInstallIpcMock() {
  installMockMode = "absent";
  applyInstallMockMode("absent");
}

export function resolveBridgeInstallStatusIpcMock(): BridgeInstallStatus {
  if (installMockMode === "unsupportedPlatform") {
    throw {
      kind: "unsupportedPlatform",
      message: "FM26 bridge plugin install requires Windows",
    };
  }

  return { ...currentInstallStatus };
}

export function resolveInstallBridgePluginIpcMock(): BridgeInstallStatus {
  if (installMockMode === "unsupportedPlatform") {
    throw {
      kind: "unsupportedPlatform",
      message: "FM26 bridge plugin install requires Windows",
    };
  }

  if (installMockMode === "bepinexMissing") {
    throw {
      kind: "bepinexMissing",
      message: "BepInEx not found at C:\\Steam\\Football Manager 26\\BepInEx",
    };
  }

  currentInstallStatus = {
    ...currentInstallStatus,
    pluginPresent: true,
  };
  installMockMode = "installed";
  return { ...currentInstallStatus };
}

export function resolveRemoveBridgePluginIpcMock(): BridgeInstallStatus {
  if (installMockMode === "unsupportedPlatform") {
    throw {
      kind: "unsupportedPlatform",
      message: "FM26 bridge plugin install requires Windows",
    };
  }

  currentInstallStatus = {
    ...currentInstallStatus,
    pluginPresent: false,
  };
  installMockMode = "absent";
  return { ...currentInstallStatus };
}
