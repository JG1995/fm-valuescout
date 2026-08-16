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
  defaultPlayerTableLayouts,
  usePlayerTableStore,
} from "@/stores/use-player-table-store";
import {
  resetAcademyIpcMock,
  resolveAssignAcademyMemberIpcMock,
  resolveCreateAcademyClassIpcMock,
  resolveDeleteAcademyClassIpcMock,
  resolveGetAcademyClassIpcMock,
  resolveListAcademyCandidatesIpcMock,
  resolveListAcademyClassesIpcMock,
  resolveRemoveAcademyMemberIpcMock,
  resolveSetAcademyMemberOutcomeIpcMock,
} from "@/testing/academy-ipc-mock";
import {
  resetCsvImportIpcMock,
  resolveBusyCsvImportRequest,
  resolveCsvImportIpcMock,
} from "@/testing/csv-import-ipc-mock";
import {
  resetPlannerIpcMock,
  resolveAddPlannerStringIpcMock,
  resolveAssignPlannerPlayerIpcMock,
  resolveClearPlannerAssignmentIpcMock,
  resolveClearPlannerDepthIpcMock,
  resolveMovePlannerPlayerIpcMock,
  resolveOptimizePlannerDepthIpcMock,
  resolvePlannerClubFamilyIpcMock,
  resolvePlannerClubsIpcMock,
  resolvePlannerDepthIpcMock,
  resolvePlannerSlotCandidatesIpcMock,
  resolvePlannerTacticIpcMock,
  resolvePlannerTacticOptionsIpcMock,
  resolveRemovePlannerStringIpcMock,
  resolveSavePlannerClubFamilyIpcMock,
  resolveSavePlannerTacticIpcMock,
} from "@/testing/planner-ipc-mock";
import {
  resetGetPlayerOverride,
  resolveBoostCurrentAbilityIpcMock,
  resolveBoostWonderkidMentalityIpcMock,
  resolveGetPlayerIpcMock,
  resolveSetPlayerHiddenInformationRevealedIpcMock,
} from "@/testing/player-ipc-mock";
import {
  resetSearchPlayersOverride,
  resolveSearchPlayersIpcMock,
  resolveSuggestPlayersIpcMock,
} from "@/testing/search-ipc-mock";
import {
  resetSnapshotIpcMock,
  resolveBusyLoadDataRequest,
  resolveBusySnapshotDeleteRequest,
  resolveCreateSaveIpcMock,
  resolveDeleteSaveIpcMock,
  resolveDeleteSnapshotIpcMock,
  resolveGetCurrentSnapshotIpcMock,
  resolveListSanityPlayersIpcMock,
  resolveListSavesIpcMock,
  resolveListSnapshotsIpcMock,
  resolveLoadDataIpcMock,
  resolveRenameSaveIpcMock,
  resolveRenameSnapshotIpcMock,
  resolveSetActiveSaveIpcMock,
  setLoadDataIpcMockMode,
} from "@/testing/snapshot-ipc-mock";
import {
  resetSquadPlayersOverride,
  resolveSquadCurrentAbilityBoostIpcMock,
  resolveSquadPlayersIpcMock,
  resolveSquadWonderkidMentalityBoostIpcMock,
} from "@/testing/squad-ipc-mock";
import {
  resetStaffIpcMock,
  resolveListMyStaffIpcMock,
  resolveSearchStaffIpcMock,
} from "@/testing/staff-ipc-mock";

function registerIpcMocks() {
  mockIPC((cmd, args) => {
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

    if (cmd === "list_snapshots") {
      return resolveListSnapshotsIpcMock(args);
    }

    if (cmd === "rename_snapshot") {
      return resolveRenameSnapshotIpcMock(args);
    }

    if (cmd === "delete_snapshot") {
      return resolveDeleteSnapshotIpcMock(args);
    }

    if (cmd === "delete_save") {
      return resolveDeleteSaveIpcMock(args);
    }

    if (cmd === "get_current_snapshot") {
      return resolveGetCurrentSnapshotIpcMock();
    }

    if (cmd === "list_sanity_players") {
      return resolveListSanityPlayersIpcMock();
    }

    if (cmd === "import_csv") {
      return resolveCsvImportIpcMock(args);
    }

    if (cmd === "search_players") {
      return resolveSearchPlayersIpcMock(args);
    }

    if (cmd === "search_staff") {
      return resolveSearchStaffIpcMock(args);
    }

    if (cmd === "list_my_staff") {
      return resolveListMyStaffIpcMock(args);
    }

    if (cmd === "suggest_players") {
      return resolveSuggestPlayersIpcMock(args);
    }

    if (cmd === "get_player") {
      return resolveGetPlayerIpcMock(args);
    }

    if (cmd === "set_hidden_information_revealed") {
      return resolveSetPlayerHiddenInformationRevealedIpcMock(args);
    }

    if (cmd === "boost_current_ability") {
      return resolveBoostCurrentAbilityIpcMock(args);
    }

    if (cmd === "boost_wonderkid_mentality") {
      return resolveBoostWonderkidMentalityIpcMock(args);
    }

    if (cmd === "list_academy_classes") {
      return resolveListAcademyClassesIpcMock();
    }

    if (cmd === "create_academy_class") {
      return resolveCreateAcademyClassIpcMock(args);
    }

    if (cmd === "delete_academy_class") {
      return resolveDeleteAcademyClassIpcMock(args);
    }

    if (cmd === "get_academy_class") {
      return resolveGetAcademyClassIpcMock(args);
    }

    if (cmd === "list_academy_candidates") {
      return resolveListAcademyCandidatesIpcMock(args);
    }

    if (cmd === "assign_academy_member") {
      return resolveAssignAcademyMemberIpcMock(args);
    }

    if (cmd === "remove_academy_member") {
      return resolveRemoveAcademyMemberIpcMock(args);
    }

    if (cmd === "set_academy_member_outcome") {
      return resolveSetAcademyMemberOutcomeIpcMock(args);
    }

    if (cmd === "get_planner_club_family") {
      return resolvePlannerClubFamilyIpcMock();
    }

    if (cmd === "list_squad_players") {
      return resolveSquadPlayersIpcMock(args);
    }

    if (cmd === "boost_squad_current_ability") {
      return resolveSquadCurrentAbilityBoostIpcMock(args);
    }

    if (cmd === "boost_squad_wonderkid_mentality") {
      return resolveSquadWonderkidMentalityBoostIpcMock(args);
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

    if (cmd === "add_planner_string") {
      return resolveAddPlannerStringIpcMock(args);
    }

    if (cmd === "remove_planner_string") {
      return resolveRemovePlannerStringIpcMock(args);
    }

    if (cmd === "clear_planner_depth") {
      return resolveClearPlannerDepthIpcMock(args);
    }

    if (cmd === "optimize_planner_depth") {
      return resolveOptimizePlannerDepthIpcMock(args);
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
  resolveBusySnapshotDeleteRequest();
  resolveBusyCsvImportRequest();
  cleanup();
  clearMocks();
  setBridgeStatusIpcMockMode("ready");
  setDumpRequestIpcMockMode("success");
  setLoadDataIpcMockMode("success");
  resetBridgeInstallIpcMock();
  resetSnapshotIpcMock();
  resetCsvImportIpcMock();
  resetSearchPlayersOverride();
  resetStaffIpcMock();
  resetSquadPlayersOverride();
  resetGetPlayerOverride();
  resetPlannerIpcMock();
  resetAcademyIpcMock();
  usePlayerTableStore.setState({ layouts: defaultPlayerTableLayouts() });
  usePlayerTableStore.persist.clearStorage();
  registerIpcMocks();
});
