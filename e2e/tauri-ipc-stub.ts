import type { Page } from "@playwright/test";

type SmokeStubOptions = {
  plannerSnapshot?: boolean;
};

export async function stubTauriIpc(page: Page, options: SmokeStubOptions = {}) {
  const plannerSnapshot = options.plannerSnapshot ?? false;
  await page.addInitScript({
    content: `
      let demoValue = "";
      const plannerSnapshot = ${plannerSnapshot ? "true" : "false"};
      const plannerTactic = {
        ipWeight: 0.5,
        lanes: [
          ["goalkeeper", "GK", "goalkeeper_ip", "GK", "line_holding_keeper_oop"],
          ["left_back", "DL", "full_back_ip", "DL", "holding_full_back_oop"],
          ["left_centre_back", "DC", "centre_back_ip", "DC", "covering_centre_back_oop"],
          ["right_centre_back", "DC", "centre_back_ip", "DC", "covering_centre_back_oop"],
          ["right_back", "DR", "full_back_ip", "DR", "holding_full_back_oop"],
          ["defensive_midfielder", "DM", "defensive_midfielder_ip", "DM", "screening_defensive_midfielder_oop"],
          ["left_central_midfielder", "MC", "central_midfielder_ip", "MC", "pressing_central_midfielder_oop"],
          ["right_central_midfielder", "MC", "central_midfielder_ip", "MC", "pressing_central_midfielder_oop"],
          ["left_winger", "AML", "winger_ip", "ML", "tracking_wide_midfielder_oop"],
          ["right_winger", "AMR", "winger_ip", "MR", "tracking_wide_midfielder_oop"],
          ["centre_forward", "ST", "centre_forward_ip", "ST", "central_outlet_centre_forward_oop"],
        ].map(([laneId, ipPosition, ipRoleId, oopPosition, oopRoleId]) => ({
          laneId,
          ipPosition,
          ipRoleId,
          oopPosition,
          oopRoleId,
        })),
      };
      const plannerDepth = {
        tactic: plannerTactic,
        teams: ["senior", "reserves", "youth"].map((team, index) => ({
          team,
          strings: [{ id: index + 1, stringOrder: 0, assignments: [] }],
        })),
      };

      window.__TAURI_INTERNALS__ = {
        invoke: async (cmd, args) => {
          if (cmd === "get_status") {
            return { status: "ok" };
          }

          if (cmd === "get_demo_value") {
            return { value: demoValue };
          }

          if (cmd === "set_demo_value") {
            demoValue = args?.value ?? "";
            return { value: demoValue };
          }

          if (cmd === "get_bridge_status") {
            return {
              protocolVersion: 1,
              pluginVersion: "0.1.0",
              state: "idle",
              updatedAtUtc: "2026-07-28T15:00:00+00:00",
              gamePluginModulePresent: true,
              gameAssemblyModulePresent: true,
            };
          }

          if (cmd === "request_player_dump") {
            return {
              requestId: "req-smoke",
              state: "ready",
              playersFound: 0,
              dumpPresent: true,
              error: null,
            };
          }

          if (cmd === "list_saves") {
            return [
              {
                id: 1,
                name: "Default save",
                isActive: true,
                createdAtUtc: "2026-07-28T12:00:00.000Z",
                updatedAtUtc: "2026-07-28T12:00:00.000Z",
              },
            ];
          }

          if (cmd === "create_save") {
            return {
              id: 2,
              name: args?.name ?? "New save",
              isActive: false,
              createdAtUtc: "2026-07-28T16:00:00.000Z",
              updatedAtUtc: "2026-07-28T16:00:00.000Z",
            };
          }

          if (cmd === "rename_save") {
            return {
              id: args?.saveId ?? 1,
              name: args?.name ?? "Renamed save",
              isActive: true,
              createdAtUtc: "2026-07-28T12:00:00.000Z",
              updatedAtUtc: "2026-07-28T16:05:00.000Z",
            };
          }

          if (cmd === "set_active_save") {
            return {
              id: args?.saveId ?? 1,
              name: "Default save",
              isActive: true,
              createdAtUtc: "2026-07-28T12:00:00.000Z",
              updatedAtUtc: "2026-07-28T16:10:00.000Z",
            };
          }

          if (cmd === "get_current_snapshot") {
            return plannerSnapshot
              ? {
                  id: 1,
                  saveId: 1,
                  schemaVersion: 5,
                  generatedAtUtc: "2026-07-28T15:00:00.000Z",
                  gameVersion: "26.0.0",
                  supportedGameVersion: "26.0.0",
                  bridgeVersion: "0.1.0",
                  protocolVersion: 1,
                  gameDate: null,
                  gameDateSource: "unknown",
                  scanTruncated: false,
                  maxAccepted: null,
                  playerCount: 3,
                  loadedAtUtc: "2026-07-28T15:05:00.000Z",
                }
              : null;
          }

          if (cmd === "list_sanity_players") {
            return [];
          }

          if (cmd === "search_players") {
            return { players: [], total: 0 };
          }

          if (cmd === "suggest_players") {
            return [];
          }

          if (cmd === "get_player") {
            return null;
          }

          if (cmd === "get_planner_club_family") {
            return { primaryClub: null, sources: [] };
          }

          if (cmd === "list_planner_clubs") {
            return plannerSnapshot
              ? ["Barcelona", "Barca Athletic", "Barcelona U19"]
              : [];
          }

          if (cmd === "save_planner_club_family") {
            return { primaryClub: args?.primaryClub ?? null, sources: [] };
          }

          if (cmd === "get_planner_tactic") {
            return plannerTactic;
          }

          if (cmd === "get_planner_depth") {
            return plannerDepth;
          }

          if (cmd === "optimize_planner_depth") {
            const reserves = plannerDepth.teams.find(
              (team) => team.team === "reserves",
            );
            if (!reserves) {
              throw new Error("Planner team not found");
            }
            reserves.strings[0].assignments = [
              {
                id: 77,
                laneId: "goalkeeper",
                playerUid: 77,
                lastKnownName: "Optimized Keeper",
                currentName: "Optimized Keeper",
                state: "resolved",
                combinedScore: 82,
              },
            ];
            return plannerDepth;
          }

          if (cmd === "add_planner_string") {
            const team = plannerDepth.teams.find(
              (candidate) => candidate.team === args?.team,
            );
            if (!team) {
              throw new Error("Planner team not found");
            }
            const id = Math.max(
              ...plannerDepth.teams.flatMap((candidate) =>
                candidate.strings.map((plannerString) => plannerString.id),
              ),
            ) + 1;
            team.strings.push({ id, stringOrder: team.strings.length, assignments: [] });
            return plannerDepth;
          }

          if (cmd === "remove_planner_string") {
            const team = plannerDepth.teams.find((candidate) =>
              candidate.strings.some(
                (plannerString) => plannerString.id === args?.stringId,
              ),
            );
            const plannerString = team?.strings.find(
              (candidate) => candidate.id === args?.stringId,
            );
            if (!team || !plannerString) {
              throw new Error("Planner string not found");
            }
            if (team.strings.length <= 1) {
              throw new Error("The " + team.team + " team must keep at least one string");
            }
            if (plannerString.assignments.length > 0 && !args?.confirmPopulated) {
              throw new Error("Removing a populated string requires confirmation");
            }
            team.strings = team.strings
              .filter((candidate) => candidate.id !== plannerString.id)
              .map((candidate, index) => ({ ...candidate, stringOrder: index }));
            return plannerDepth;
          }

          if (cmd === "get_planner_tactic_options") {
            return {
              placements: ["GK", "DL", "DC", "DR", "DM", "MC", "ML", "MR", "AML", "AMR", "ST"],
              roles: [
                { roleId: "goalkeeper_ip", displayName: "Goalkeeper", phase: "in_possession", positionTags: ["GK"] },
                { roleId: "line_holding_keeper_oop", displayName: "Line-Holding Keeper", phase: "out_of_possession", positionTags: ["GK"] },
                { roleId: "full_back_ip", displayName: "Full-Back", phase: "in_possession", positionTags: ["DL", "DR"] },
                { roleId: "holding_full_back_oop", displayName: "Holding Full-Back", phase: "out_of_possession", positionTags: ["DL", "DR"] },
                { roleId: "centre_back_ip", displayName: "Centre-Back", phase: "in_possession", positionTags: ["DC"] },
                { roleId: "covering_centre_back_oop", displayName: "Covering Centre-Back", phase: "out_of_possession", positionTags: ["DC"] },
                { roleId: "defensive_midfielder_ip", displayName: "Defensive Midfielder", phase: "in_possession", positionTags: ["DM"] },
                { roleId: "screening_defensive_midfielder_oop", displayName: "Screening Defensive Midfielder", phase: "out_of_possession", positionTags: ["DM"] },
                { roleId: "central_midfielder_ip", displayName: "Central Midfielder", phase: "in_possession", positionTags: ["MC"] },
                { roleId: "pressing_central_midfielder_oop", displayName: "Pressing Central Midfielder", phase: "out_of_possession", positionTags: ["MC"] },
                { roleId: "winger_ip", displayName: "Winger", phase: "in_possession", positionTags: ["ML", "MR", "AML", "AMR"] },
                { roleId: "tracking_wide_midfielder_oop", displayName: "Tracking Wide Midfielder", phase: "out_of_possession", positionTags: ["ML", "MR"] },
                { roleId: "centre_forward_ip", displayName: "Centre Forward", phase: "in_possession", positionTags: ["ST"] },
                { roleId: "central_outlet_centre_forward_oop", displayName: "Central Outlet Centre Forward", phase: "out_of_possession", positionTags: ["ST"] },
              ],
            };
          }

          if (cmd === "save_planner_tactic") {
            return args?.tactic;
          }

          if (cmd === "load_data") {
            return {
              requestId: "req-smoke",
              playersFound: 0,
              scanTruncated: false,
              maxAccepted: null,
              timings: { scanMs: 0, ingestMs: 0, totalMs: 0 },
              snapshot: {
                id: 1,
                saveId: 1,
                schemaVersion: 5,
                generatedAtUtc: "2026-07-28T15:00:00.000Z",
                gameVersion: "26.0.0",
                supportedGameVersion: "26.0.0",
                bridgeVersion: "0.1.0",
                protocolVersion: 1,
                gameDate: null,
                gameDateSource: "unknown",
                scanTruncated: false,
                maxAccepted: null,
                playerCount: 0,
                loadedAtUtc: "2026-07-28T15:05:00.000Z",
              },
            };
          }

          if (cmd === "get_bridge_install_status") {
            return {
              pluginsPath:
                "C:\\\\Program Files (x86)\\\\Steam\\\\steamapps\\\\common\\\\Football Manager 26\\\\BepInEx\\\\plugins",
              pluginPresent: false,
              bepinexPresent: true,
              pluginsDirPresent: true,
            };
          }

          if (cmd === "install_bridge_plugin") {
            return {
              pluginsPath:
                "C:\\\\Program Files (x86)\\\\Steam\\\\steamapps\\\\common\\\\Football Manager 26\\\\BepInEx\\\\plugins",
              pluginPresent: true,
              bepinexPresent: true,
              pluginsDirPresent: true,
            };
          }

          if (cmd === "remove_bridge_plugin") {
            return {
              pluginsPath:
                "C:\\\\Program Files (x86)\\\\Steam\\\\steamapps\\\\common\\\\Football Manager 26\\\\BepInEx\\\\plugins",
              pluginPresent: false,
              bepinexPresent: true,
              pluginsDirPresent: true,
            };
          }

          throw new Error("Unhandled IPC: " + cmd);
        },
        transformCallback: (callback) => callback,
        convertFileSrc: (filePath) => filePath,
        metadata: {
          currentWindow: { label: "main" },
          currentWebview: { windowLabel: "main", label: "main" },
        },
      };
    `,
  });
}
