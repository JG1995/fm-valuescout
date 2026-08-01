import "@testing-library/jest-dom/vitest";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { cleanup } from "@testing-library/react";
import { afterEach } from "vitest";
import {
  resetBridgeInstallIpcMock,
  resolveBridgeInstallStatusIpcMock,
  resolveInstallBridgePluginIpcMock,
  resolveRemoveBridgePluginIpcMock,
} from "@/features/memory-read/api/bridge-install-ipc-mock";
import {
  resolveBridgeStatusIpcMock,
  resolveBusyDumpRequest,
  resolveDumpRequestIpcMock,
  setBridgeStatusIpcMockMode,
  setDumpRequestIpcMockMode,
} from "@/features/memory-read/api/bridge-status-ipc-mock";
import {
  resetPlannerIpcMock,
  resolveAssignPlannerPlayerIpcMock,
  resolveClearPlannerAssignmentIpcMock,
  resolveMovePlannerPlayerIpcMock,
  resolvePlannerClubFamilyIpcMock,
  resolvePlannerClubsIpcMock,
  resolvePlannerDepthIpcMock,
  resolvePlannerSlotCandidatesIpcMock,
  resolvePlannerTacticIpcMock,
  resolvePlannerTacticOptionsIpcMock,
  resolveSavePlannerClubFamilyIpcMock,
  resolveSavePlannerTacticIpcMock,
} from "@/testing/planner-ipc-mock";
import {
  resetGetPlayerOverride,
  resolveGetPlayerIpcMock,
} from "@/testing/player-ipc-mock";
import {
  resetSearchPlayersOverride,
  resolveSearchPlayersIpcMock,
  resolveSuggestPlayersIpcMock,
} from "@/testing/search-ipc-mock";
import {
  resetSnapshotIpcMock,
  resolveBusyLoadDataRequest,
  resolveCreateSaveIpcMock,
  resolveGetCurrentSnapshotIpcMock,
  resolveListSanityPlayersIpcMock,
  resolveListSavesIpcMock,
  resolveLoadDataIpcMock,
  resolveRenameSaveIpcMock,
  resolveSetActiveSaveIpcMock,
  setLoadDataIpcMockMode,
} from "@/testing/snapshot-ipc-mock";

let demoValue = "";

function registerIpcMocks() {
  mockIPC((cmd, args) => {
    if (cmd === "get_status") {
      return { status: "ok" };
    }

    if (cmd === "get_demo_value") {
      return { value: demoValue };
    }

    if (cmd === "set_demo_value") {
      const nextValue =
        typeof args === "object" &&
        args !== null &&
        "value" in args &&
        typeof args.value === "string"
          ? args.value
          : "";
      demoValue = nextValue;
      return { value: demoValue };
    }

    if (cmd === "get_bridge_status") {
      return resolveBridgeStatusIpcMock();
    }

    if (cmd === "request_player_dump") {
      return resolveDumpRequestIpcMock();
    }

    if (cmd === "list_saves") {
      return resolveListSavesIpcMock();
    }

    if (cmd === "create_save") {
      return resolveCreateSaveIpcMock(args);
    }

    if (cmd === "rename_save") {
      return resolveRenameSaveIpcMock(args);
    }

    if (cmd === "set_active_save") {
      return resolveSetActiveSaveIpcMock(args);
    }

    if (cmd === "get_current_snapshot") {
      return resolveGetCurrentSnapshotIpcMock();
    }

    if (cmd === "list_sanity_players") {
      return resolveListSanityPlayersIpcMock();
    }

    if (cmd === "search_players") {
      return resolveSearchPlayersIpcMock(args);
    }

    if (cmd === "suggest_players") {
      return resolveSuggestPlayersIpcMock(args);
    }

    if (cmd === "get_player") {
      return resolveGetPlayerIpcMock(args);
    }

    if (cmd === "get_planner_club_family") {
      return resolvePlannerClubFamilyIpcMock();
    }

    if (cmd === "list_planner_clubs") {
      return resolvePlannerClubsIpcMock();
    }

    if (cmd === "save_planner_club_family") {
      return resolveSavePlannerClubFamilyIpcMock(args);
    }

    if (cmd === "get_planner_tactic") {
      return resolvePlannerTacticIpcMock();
    }

    if (cmd === "get_planner_tactic_options") {
      return resolvePlannerTacticOptionsIpcMock();
    }

    if (cmd === "get_planner_depth") {
      return resolvePlannerDepthIpcMock();
    }

    if (cmd === "get_planner_slot_candidates") {
      return resolvePlannerSlotCandidatesIpcMock(args);
    }

    if (cmd === "save_planner_tactic") {
      return resolveSavePlannerTacticIpcMock(args);
    }

    if (cmd === "assign_planner_player") {
      return resolveAssignPlannerPlayerIpcMock(args);
    }

    if (cmd === "clear_planner_assignment") {
      return resolveClearPlannerAssignmentIpcMock(args);
    }

    if (cmd === "move_planner_player") {
      return resolveMovePlannerPlayerIpcMock(args);
    }

    if (cmd === "load_data") {
      return resolveLoadDataIpcMock(args);
    }

    if (cmd === "get_bridge_install_status") {
      return resolveBridgeInstallStatusIpcMock();
    }

    if (cmd === "install_bridge_plugin") {
      return resolveInstallBridgePluginIpcMock();
    }

    if (cmd === "remove_bridge_plugin") {
      return resolveRemoveBridgePluginIpcMock();
    }

    throw new Error(`Unhandled IPC command: ${cmd}`);
  });
}

registerIpcMocks();

afterEach(() => {
  resolveBusyDumpRequest();
  resolveBusyLoadDataRequest();
  cleanup();
  clearMocks();
  demoValue = "";
  setBridgeStatusIpcMockMode("ready");
  setDumpRequestIpcMockMode("success");
  setLoadDataIpcMockMode("success");
  resetBridgeInstallIpcMock();
  resetSnapshotIpcMock();
  resetSearchPlayersOverride();
  resetGetPlayerOverride();
  resetPlannerIpcMock();
  registerIpcMocks();
});
