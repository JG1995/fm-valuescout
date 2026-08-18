import type { Page } from "@playwright/test";

type SmokeStubOptions = {
  csvImportFormat?: "youthTracker" | "moneyball";
  plannerSnapshot?: boolean;
  plannerPotentialScores?: boolean;
  playerTableRowCount?: number;
  squadPageFailure?: boolean;
  squadOverview?: boolean;
  playerProfile?: boolean;
  staffWorkspace?: boolean;
  staffShortlist?: boolean;
  staffFamily?: "configured" | "none";
  snapshotHistory?: boolean;
};

export async function stubTauriIpc(page: Page, options: SmokeStubOptions = {}) {
  const csvImportFormat = options.csvImportFormat ?? null;
  const plannerSnapshot = options.plannerSnapshot ?? false;
  const plannerPotentialScores = options.plannerPotentialScores ?? false;
  const playerTableRowCount = options.playerTableRowCount ?? null;
  const squadPageFailure = options.squadPageFailure ?? false;
  const squadOverview = options.squadOverview ?? false;
  const playerProfile = options.playerProfile ?? false;
  const staffWorkspace = options.staffWorkspace ?? false;
  const staffShortlist = options.staffShortlist ?? false;
  const staffFamilyConfigured = options.staffFamily !== "none";
  const snapshotHistory = options.snapshotHistory ?? false;
  await page.addInitScript({
    content: `
      let playerProfileMentalityUpdated = false;
      let playerProfileHiddenInformationRevealed =
        window.localStorage.getItem("player-profile-hidden-information") !==
        "false";
      const csvImportFormat = ${JSON.stringify(csvImportFormat)};
      const plannerSnapshot = ${plannerSnapshot ? "true" : "false"};
      const plannerPotentialScores = ${plannerPotentialScores ? "true" : "false"};
      const playerTableRowCount = ${JSON.stringify(playerTableRowCount)};
      const squadPageFailure = ${squadPageFailure ? "true" : "false"};
      const squadOverview = ${squadOverview ? "true" : "false"};
      const playerProfile = ${playerProfile ? "true" : "false"};
      const staffWorkspace = ${staffWorkspace ? "true" : "false"};
      const staffShortlist = ${staffShortlist ? "true" : "false"};
      const staffFamilyConfigured = ${staffFamilyConfigured ? "true" : "false"};
      const snapshotHistoryEnabled = ${snapshotHistory ? "true" : "false"};
      let nextSaveId = 2;
      let saves = [{
        id: 1,
        contextToken: "save-token-1",
        name: "Default save",
        isActive: true,
        createdAtUtc: "2026-07-28T12:00:00.000Z",
        updatedAtUtc: "2026-07-28T12:00:00.000Z",
      }];
      let snapshots = snapshotHistoryEnabled ? [
        {
          id: 1,
          contextToken: "snapshot-token-1",
          saveId: 1,
          customName: null,
          gameDate: "2026-06-01",
          gameDateSource: "inGame",
          playerCount: 21,
          loadedAtUtc: "2026-07-28T13:00:00.000Z",
          isCurrent: false,
        },
        {
          id: 2,
          contextToken: "snapshot-token-2",
          saveId: 1,
          customName: null,
          gameDate: "2026-08-01",
          gameDateSource: "inGame",
          playerCount: 24,
          loadedAtUtc: "2026-07-28T15:00:00.000Z",
          isCurrent: true,
        },
      ] : [];
      const activeSave = () => saves.find((save) => save.isActive) || saves[0];
      const snapshotOrder = (left, right) => {
        if (left.gameDate === null && right.gameDate !== null) return 1;
        if (left.gameDate !== null && right.gameDate === null) return -1;
        if (left.gameDate !== right.gameDate) {
          return (right.gameDate || "").localeCompare(left.gameDate || "");
        }
        if (left.loadedAtUtc !== right.loadedAtUtc) {
          return right.loadedAtUtc.localeCompare(left.loadedAtUtc);
        }
        return right.id - left.id;
      };
      const snapshotsForSave = (saveId) => snapshots
        .filter((snapshot) => snapshot.saveId === saveId)
        .sort(snapshotOrder);
      const promoteCurrentSnapshot = (saveId) => {
        const next = snapshotsForSave(saveId)[0] || null;
        snapshots = snapshots.map((snapshot) => snapshot.saveId === saveId
          ? { ...snapshot, isCurrent: snapshot.id === next?.id }
          : snapshot);
        return next?.id || null;
      };
      const currentSnapshot = () => snapshotsForSave(activeSave().id)
        .find((snapshot) => snapshot.isCurrent) || null;
      const snapshotSummary = (snapshot) => ({
        id: snapshot.id,
        saveId: snapshot.saveId,
        schemaVersion: 6,
        generatedAtUtc: snapshot.loadedAtUtc,
        gameVersion: "26.0.0",
        supportedGameVersion: "26.0.0",
        bridgeVersion: "0.1.0",
        protocolVersion: 1,
        gameDate: snapshot.gameDate,
        gameDateSource: snapshot.gameDateSource,
        scanTruncated: false,
        maxAccepted: null,
        playerCount: snapshot.playerCount,
        loadedAtUtc: snapshot.loadedAtUtc,
      });
      const snapshotSanityPlayers = (snapshot) => [{
        name: "Snapshot " + snapshot.id + " player",
        ca: snapshot.playerCount,
        club: "Snapshot " + snapshot.id + " FC",
        proofRoleScore: snapshot.playerCount,
      }];
      const plannerTactic = {
        lanes: [
          ["goalkeeper", "GK", "goalkeeper_ip", "GK", "line_holding_keeper_oop"],
          ["left_back", "DL", "full_back_ip", "DL", "holding_full_back_oop"],
          ["left_centre_back", "DCR", "centre_back_ip", "DCR", "covering_centre_back_oop"],
          ["right_centre_back", "DCL", "centre_back_ip", "DCL", "covering_centre_back_oop"],
          ["right_back", "DR", "full_back_ip", "DR", "holding_full_back_oop"],
          ["defensive_midfielder", "DM", "defensive_midfielder_ip", "DM", "screening_defensive_midfielder_oop"],
          ["left_central_midfielder", "MCR", "central_midfielder_ip", "MCR", "pressing_central_midfielder_oop"],
          ["right_central_midfielder", "MCL", "central_midfielder_ip", "MCL", "pressing_central_midfielder_oop"],
          ["left_winger", "AML", "winger_ip", "ML", "tracking_wide_midfielder_oop"],
          ["right_winger", "AMR", "winger_ip", "MR", "tracking_wide_midfielder_oop"],
          ["centre_forward", "STC", "centre_forward_ip", "STC", "central_outlet_centre_forward_oop"],
        ].map(([laneId, ipPosition, ipRoleId, oopPosition, oopRoleId]) => ({
          laneId,
          ipWeight: 0.5,
          importanceRank: null,
          preferredFoot: "any",
          footPreference: "preferred",
          ipPosition,
          ipRoleId,
          oopPosition,
          oopRoleId,
        })),
      };
      const plannerTeamNames = {
        senior: "Senior",
        reserves: "Reserves",
        youth: "Youth",
      };
      const plannerDepth = {
        tactic: plannerTactic,
        teams: ["senior", "reserves", "youth"].map((team, index) => ({
          team,
          displayName: plannerTeamNames[team],
          strings: [{
            id: index + 1,
            stringOrder: 0,
            assignments: plannerPotentialScores && team === "senior" ? [{
              id: 77,
              laneId: "goalkeeper",
              playerUid: 77,
              lastKnownName: "Potential Keeper",
              currentName: "Potential Keeper",
              state: "resolved",
              combinedScore: 82,
              potentialCombinedScore: 91,
            }] : [],
          }],
        })),
      };
      let squadPlayers = squadOverview ? [
        {
          uid: 42,
          name: "Alex Scout",
          age: 25,
          birthYear: 2001,
          birthDayOfYear: 80,
          nationalities: ["ENG"],
          club: "Barcelona",
          division: "La Liga",
          ca: 160,
          pa: 170,
          marketValueGbp: 16000000,
        },
        {
          uid: 77,
          name: "Zara Keeper",
          age: 22,
          birthYear: 2004,
          birthDayOfYear: 124,
          nationalities: ["ESP"],
          club: "Barcelona",
          division: "La Liga",
          ca: 150,
          pa: 165,
          marketValueGbp: 12000000,
        },
      ] : [];
      const staffRoleIds = [
        "assistant_manager",
        "manager",
        "coach_attacking_technical",
        "coach_attacking_tactical",
        "coach_defending_technical",
        "coach_defending_tactical",
        "coach_possession_technical",
        "coach_possession_tactical",
        "coach_fitness",
        "coach_goalkeeping",
        "set_piece_coach",
        "loan_manager",
        "head_of_youth_development",
        "scout",
        "director_of_football",
        "technical_director",
        "recruitment_analyst",
        "head_performance_analyst",
        "performance_analyst",
        "physio",
        "sports_scientist",
      ];
      let staffRows = staffWorkspace ? [{
        uid: 101,
        name: "Alex Coach",
        age: 44,
        birthYear: 1982,
        birthDayOfYear: 120,
        nationalities: ["Denmark"],
        nationUid: null,
        gender: "male",
        club: "Barcelona",
        division: "La Liga",
        ca: 145,
        pa: 160,
        jobId: 1,
        weeklyWageGbp: 15000,
        contractExpiryYear: 2028,
        contractExpiryDayOfYear: 220,
        dynamicValues: Object.fromEntries(staffRoleIds.map((roleId) => ["role." + roleId, 72])),
      }] : [];
      if (staffWorkspace && playerTableRowCount !== null) {
        staffRows = Array.from({ length: playerTableRowCount }, (_, index) => ({
          uid: index + 101,
          name: "Staff member " + String(index + 1).padStart(3, "0"),
          age: 44,
          birthYear: 1982,
          birthDayOfYear: 120,
          nationalities: ["Denmark"],
          nationUid: null,
          gender: "male",
          club: index % 2 === 0 ? "Barcelona" : "Barcelona B",
          division: "La Liga",
          ca: 145,
          pa: 160,
          jobId: 1,
          weeklyWageGbp: 15000,
          contractExpiryYear: 2028,
          contractExpiryDayOfYear: 220,
          dynamicValues: Object.fromEntries(staffRoleIds.map((roleId) => ["role." + roleId, 72])),
        }));
      }
      const shortlistStaffRows = staffWorkspace && staffShortlist ? [
        {
          ...staffRows[0],
          shortlist: {
            preferredJob: "Technical Director",
            clubJob: "Technical Director",
            coachingQualifications: "Continental Pro",
          },
          dynamicValues: {
            ...staffRows[0].dynamicValues,
            "role.technical_director": 95,
          },
        },
        {
          ...staffRows[0],
          uid: 102,
          name: "Coach Casey",
          shortlist: {
            preferredJob: "Coach",
            clubJob: "-",
            coachingQualifications: "Continental A",
          },
        },
        {
          ...staffRows[0],
          uid: 103,
          name: "Manager Morgan",
          shortlist: {
            preferredJob: "Manager",
            clubJob: "",
            coachingQualifications: "Continental Pro",
          },
          dynamicValues: {
            ...staffRows[0].dynamicValues,
            "role.manager": 90,
          },
        },
        {
          ...staffRows[0],
          uid: 104,
          name: "Manager Taylor",
          ca: 150,
          shortlist: {
            preferredJob: "Manager",
            clubJob: "Manager",
            coachingQualifications: "Continental Pro",
          },
          dynamicValues: {
            ...staffRows[0].dynamicValues,
            "role.manager": 80,
          },
        },
      ] : [];
      if (squadOverview && playerTableRowCount !== null) {
        squadPlayers = Array.from({ length: playerTableRowCount }, (_, index) => ({
          uid: index + 1,
          name: "Player " + String(index + 1).padStart(3, "0"),
          age: 25,
          birthYear: 2001,
          birthDayOfYear: 80,
          nationalities: ["ENG"],
          club: "Barcelona",
          division: "La Liga",
          ca: 200 - index,
          pa: 210 - index,
          marketValueGbp: 16000000,
        }));
      }
      const sendSquadBoostProgress = (args, progress) => {
        args?.onProgress?.onmessage?.(progress);
      };
      const resolveSquadBoost = async (args) => {
        sendSquadBoostProgress(args, {
          processed: 0,
          total: 2,
          updated: 0,
          skipped: 0,
          failed: 0,
        });
        await new Promise((resolve) => setTimeout(resolve, 50));
        sendSquadBoostProgress(args, {
          processed: 1,
          total: 2,
          updated: 1,
          skipped: 0,
          failed: 0,
        });
        await new Promise((resolve) => setTimeout(resolve, 50));
        sendSquadBoostProgress(args, {
          processed: 2,
          total: 2,
          updated: 2,
          skipped: 0,
          failed: 0,
        });
      };
      const resolveMyStaffBoost = async (args) => {
        const total = staffRows.length;
        sendSquadBoostProgress(args, {
          processed: 0,
          total,
          updated: 0,
          skipped: 0,
          failed: 0,
        });
        await new Promise((resolve) => setTimeout(resolve, 50));
        staffRows = staffRows.map((staff) => ({
          ...staff,
          ca: Math.min(staff.ca + 10, staff.pa, 200),
        }));
        sendSquadBoostProgress(args, {
          processed: total,
          total,
          updated: total,
          skipped: 0,
          failed: 0,
        });
      };
      if (squadPageFailure) {
        squadPlayers = Array.from({ length: 51 }, (_, index) => ({
          uid: index + 1,
          name: "Squad player " + String(index + 1).padStart(3, "0"),
          age: 25,
          birthYear: 2001,
          birthDayOfYear: 80,
          nationalities: ["ENG"],
          club: "Barcelona",
          division: "La Liga",
          ca: 200 - index,
          pa: 210 - index,
          marketValueGbp: 16000000,
        }));
      }
      let squadPageFailureTriggered = false;

      window.__TAURI_INTERNALS__ = {
        invoke: async (cmd, args) => {
          if (cmd === "plugin:dialog|open") {
            return csvImportFormat ? "/tmp/smoke-import.csv" : null;
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
            return saves.map((save) => ({ ...save }));
          }

          if (cmd === "create_save") {
            const created = {
              id: nextSaveId,
              contextToken: "save-token-" + nextSaveId,
              name: args?.name ?? "New save",
              isActive: false,
              createdAtUtc: "2026-07-28T16:00:00.000Z",
              updatedAtUtc: "2026-07-28T16:00:00.000Z",
            };
            nextSaveId += 1;
            saves.push(created);
            return created;
          }

          if (cmd === "rename_save") {
            const index = saves.findIndex((save) => save.id === args?.saveId);
            if (index < 0) throw new Error("Save not found");
            const updated = {
              ...saves[index],
              name: args?.name ?? "Renamed save",
              updatedAtUtc: "2026-07-28T16:05:00.000Z",
            };
            saves[index] = updated;
            return updated;
          }

          if (cmd === "set_active_save") {
            const target = saves.find((save) => save.id === args?.saveId);
            if (!target) throw new Error("Save not found");
            saves = saves.map((save) => ({
              ...save,
              isActive: save.id === target.id,
              updatedAtUtc: save.id === target.id
                ? "2026-07-28T16:10:00.000Z"
                : save.updatedAtUtc,
            }));
            return saves.find((save) => save.id === target.id);
          }

          if (cmd === "list_snapshots") {
            return snapshotsForSave(args?.saveId ?? activeSave().id)
              .map((snapshot) => ({ ...snapshot }));
          }

          if (cmd === "rename_snapshot") {
            const index = snapshots.findIndex((snapshot) =>
              snapshot.id === args?.snapshotId &&
              snapshot.contextToken === args?.contextToken,
            );
            if (index < 0) throw new Error("Snapshot changed or no longer exists");
            const updated = {
              ...snapshots[index],
              customName: typeof args?.customName === "string" && args.customName.trim()
                ? args.customName.trim()
                : null,
            };
            snapshots[index] = updated;
            return updated;
          }

          if (cmd === "delete_snapshot") {
            const target = snapshots.find((snapshot) =>
              snapshot.id === args?.snapshotId &&
              snapshot.contextToken === args?.contextToken,
            );
            if (!target) throw new Error("Snapshot changed or no longer exists");
            snapshots = snapshots.filter((snapshot) => snapshot.id !== target.id);
            const currentSnapshotId = target.isCurrent
              ? promoteCurrentSnapshot(target.saveId)
              : snapshotsForSave(target.saveId).find((snapshot) => snapshot.isCurrent)?.id || null;
            return {
              deletedSnapshotId: target.id,
              saveId: target.saveId,
              currentSnapshotId,
            };
          }

          if (cmd === "delete_save") {
            const target = saves.find((save) =>
              save.id === args?.saveId && save.contextToken === args?.contextToken,
            );
            if (!target) throw new Error("Save changed or no longer exists");
            saves = saves.filter((save) => save.id !== target.id);
            snapshots = snapshots.filter((snapshot) => snapshot.saveId !== target.id);
            if (target.isActive) {
              if (saves.length === 0) {
                saves = [{
                  id: nextSaveId,
                  contextToken: "save-token-" + nextSaveId,
                  name: "Default save",
                  isActive: true,
                  createdAtUtc: "2026-07-28T16:20:00.000Z",
                  updatedAtUtc: "2026-07-28T16:20:00.000Z",
                }];
                nextSaveId += 1;
              } else {
                saves = saves.map((save, index) => ({
                  ...save,
                  isActive: index === 0,
                }));
              }
            }
            return {
              deletedSaveId: target.id,
              deletedWasActive: target.isActive,
              activeSave: { ...activeSave() },
            };
          }

          if (cmd === "get_current_snapshot") {
            const snapshot = currentSnapshot();
            if (snapshot) return snapshotSummary(snapshot);
            return plannerSnapshot || playerProfile || staffWorkspace
              ? {
                  id: 1,
                  saveId: 1,
                  schemaVersion: 6,
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
            const snapshot = currentSnapshot();
            return snapshot ? snapshotSanityPlayers(snapshot) : [];
          }

          if (cmd === "import_csv") {
            return {
              format: csvImportFormat,
              totalPlayers: csvImportFormat === "moneyball" ? 75 : 3,
              storedPlayers: csvImportFormat === "moneyball" ? 74 : 3,
              skippedPlayers: csvImportFormat === "moneyball" ? 1 : 0,
            };
          }

          if (cmd === "search_players") {
            const offset = Number.isInteger(args?.offset)
              ? Math.max(0, args.offset)
              : 0;
            const limit = Number.isInteger(args?.limit)
              ? Math.min(200, Math.max(1, args.limit))
              : 50;
            return squadOverview
              ? {
                  players: squadPlayers.slice(offset, offset + limit),
                  total: squadPlayers.length,
                }
              : { players: [], total: 0 };
          }

          if (cmd === "search_staff") {
            const offset = Number.isInteger(args?.offset)
              ? Math.max(0, args.offset)
              : 0;
            const limit = Number.isInteger(args?.limit)
              ? Math.min(200, Math.max(1, args.limit))
              : 50;
            return {
              state: staffWorkspace ? "ready" : "no_current_snapshot",
              staff: staffRows.slice(offset, offset + limit),
              total: staffRows.length,
            };
          }

          if (cmd === "list_my_staff") {
            const offset = Number.isInteger(args?.offset)
              ? Math.max(0, args.offset)
              : 0;
            const limit = Number.isInteger(args?.limit)
              ? Math.min(200, Math.max(1, args.limit))
              : 50;
            return {
              state: !staffWorkspace
                ? "no_current_snapshot"
                : staffFamilyConfigured
                  ? "ready"
                  : "no_club_family",
              staff: staffFamilyConfigured
                ? staffRows.slice(offset, offset + limit)
                : [],
              total: staffFamilyConfigured ? staffRows.length : 0,
            };
          }

          if (cmd === "list_staff_shortlist") {
            if (staffWorkspace && staffShortlist) {
              const preferredJob = typeof args?.preferredJob === "string"
                ? args.preferredJob
                : "";
              const unemployedOnly = args?.unemployedOnly === true;
              const matching = shortlistStaffRows.filter((staff) =>
                (!preferredJob || staff.shortlist.preferredJob === preferredJob) &&
                (!unemployedOnly || !staff.shortlist.clubJob || staff.shortlist.clubJob === "-"),
              );
              if (args?.sortBy === "ca" && args?.sortDir === "desc") {
                matching.sort((left, right) => right.ca - left.ca);
              } else if (
                typeof args?.sortBy === "string" &&
                args.sortBy.startsWith("role.") &&
                args?.sortDir === "desc"
              ) {
                matching.sort(
                  (left, right) =>
                    (right.dynamicValues?.[args.sortBy] ?? -1) -
                    (left.dynamicValues?.[args.sortBy] ?? -1),
                );
              }
              const offset = Number.isInteger(args?.offset)
                ? Math.max(0, args.offset)
                : 0;
              const limit = Number.isInteger(args?.limit)
                ? Math.min(200, Math.max(1, args.limit))
                : 50;
              return {
                state: "ready",
                staff: matching.slice(offset, offset + limit),
                total: matching.length,
                preferredJobOptions: ["Coach", "Manager", "Technical Director"],
              };
            }
            return {
              state: staffWorkspace ? "no_shortlist" : "no_current_snapshot",
              staff: [],
              total: 0,
              preferredJobOptions: [],
            };
          }

          if (cmd === "boost_staff_current_ability") {
            const uid = Number.isInteger(args?.uid) ? args.uid : 0;
            const staff = staffRows.find((row) => row.uid === uid);
            if (!staff) {
              throw new Error("Staff member not found");
            }
            const previousCurrentAbility = staff.ca;
            const currentAbility = Math.min(staff.ca + 10, staff.pa, 200);
            staffRows = staffRows.map((row) =>
              row.uid === uid ? { ...row, ca: currentAbility } : row,
            );
            return {
              snapshotId: 1,
              operation: "boost-staff-current-ability",
              previousCurrentAbility,
              currentAbility,
              potentialAbility: staff.pa,
            };
          }

          if (cmd === "boost_my_staff_current_ability") {
            await resolveMyStaffBoost(args);
            return {
              updated: staffRows.length,
              skipped: 0,
              failed: 0,
              recoveryRequired: false,
              recoveryMessage: null,
            };
          }

          if (cmd === "suggest_players") {
            return [];
          }

          if (cmd === "get_player") {
            const profileUid = Number.isInteger(args?.uid) ? args.uid : 42;
            return playerProfile ? {
              uid: profileUid,
              name: profileUid === 99 ? "Other Scout" : "Potential Scout",
              age: 22,
              birthYear: 2004,
              birthDayOfYear: 80,
              nationalities: ["ENG"],
              heightCm: 182,
              preferredFoot: "right",
              positions: {
                GK: 14,
                SW: 18,
                DL: null,
                DC: null,
                DR: null,
                DM: null,
                ML: null,
                MC: 20,
                MR: 17,
                AML: null,
                AMC: 14,
                AMR: null,
                ST: 15,
                WBL: 0,
                WBR: null,
              },
              attributes: {
                Passing: 14,
                Determination: playerProfileMentalityUpdated ? 18 : 8,
              },
              potentialAttributes: { Passing: 16 },
              hiddenAttributes: { Consistency: 12 },
              personality: {
                Ambition: playerProfileMentalityUpdated ? 20 : 10,
                Professionalism: 15,
              },
              weeklyWageGbp: 50000,
              contractExpiryYear: 2028,
              contractExpiryDayOfYear: 1,
              transferListed: false,
              loanListed: null,
              notForSale: null,
              setForRelease: null,
              marketValueGbp: 12500000,
              reputationCurrent: 5000,
              reputationWorld: 4000,
              club: "Test FC",
              parentClub: null,
              onLoan: false,
              division: "Premier Division",
              teamLevel: "First",
              ca: 140,
              pa: 160,
              hiddenInformationRevealed: playerProfileHiddenInformationRevealed,
              roleScores: [
                {
                  roleId: "current-specialist",
                  displayName: "Current Specialist",
                  phase: "in_possession",
                  positionTags: ["MC"],
                  score: 82,
                  potentialScore: 88,
                },
                {
                  roleId: "advanced-playmaker",
                  displayName: "Advanced Playmaker",
                  phase: "in_possession",
                  positionTags: ["MC"],
                  score: 77,
                  potentialScore: 91,
                },
                {
                  roleId: "central-midfielder",
                  displayName: "Central Midfielder",
                  phase: "in_possession",
                  positionTags: ["MC"],
                  score: 73,
                  potentialScore: 83,
                },
                {
                  roleId: "box-to-box-midfielder",
                  displayName: "Box-to-Box Midfielder",
                  phase: "in_possession",
                  positionTags: ["MC"],
                  score: 68,
                  potentialScore: 79,
                },
                {
                  roleId: "deep-lying-playmaker",
                  displayName: "Deep-Lying Playmaker",
                  phase: "in_possession",
                  positionTags: ["MC"],
                  score: 62,
                  potentialScore: 76,
                },
                {
                  roleId: "ball-winning-midfielder",
                  displayName: "Ball-Winning Midfielder",
                  phase: "out_of_possession",
                  positionTags: ["MC"],
                  score: 56,
                  potentialScore: 68,
                },
                {
                  roleId: "pressing-central-midfielder",
                  displayName: "Pressing Central Midfielder",
                  phase: "out_of_possession",
                  positionTags: ["MC"],
                  score: 49,
                  potentialScore: 60,
                },
                {
                  roleId: "potential-specialist",
                  displayName: "Potential Specialist",
                  phase: "in_possession",
                  positionTags: ["ST"],
                  score: 70,
                  potentialScore: 94,
                },
                {
                  roleId: "advanced-forward",
                  displayName: "Advanced Forward",
                  phase: "in_possession",
                  positionTags: ["ST"],
                  score: 78,
                  potentialScore: 87,
                },
                {
                  roleId: "deep-lying-forward",
                  displayName: "Deep-Lying Forward",
                  phase: "in_possession",
                  positionTags: ["ST"],
                  score: 64,
                  potentialScore: 82,
                },
                {
                  roleId: "pressing-forward",
                  displayName: "Pressing Forward",
                  phase: "out_of_possession",
                  positionTags: ["ST"],
                  score: 60,
                  potentialScore: 77,
                },
              ],
            } : null;
          }

          if (cmd === "get_staff") {
            const staffUid = Number.isInteger(args?.uid) ? args.uid : 0;
            if (!staffWorkspace || staffUid !== 101) return null;
            const attributes = Object.fromEntries([
              "Attacking",
              "Defending",
              "Fitness",
              "GoalkeepingDistribution",
              "GoalkeepingHandling",
              "GoalkeepingReflexes",
              "Possession",
              "SetPieces",
              "Tactical",
              "Technical",
              "Adaptability",
              "Authority",
              "Determination",
              "ManManagement",
              "Motivating",
              "WorkingWithYoungsters",
              "DataAnalysis",
              "JudgingPlayerAbility",
              "JudgingPlayerPotential",
              "JudgingStaffAbility",
              "Negotiating",
              "Physiotherapy",
              "SportsScience",
              "TacticalKnowledge",
            ].map((key) => [key, key === "Adaptability" ? 16 : 15]));
            return {
              uid: 101,
              name: "Alex Coach",
              age: 44,
              birthYear: 1982,
              birthDayOfYear: 120,
              nationalities: ["Denmark"],
              nationUid: null,
              gender: "male",
              club: "Barcelona",
              division: "La Liga",
              ca: 145,
              pa: 160,
              jobId: 1,
              weeklyWageGbp: 15000,
              contractExpiryYear: 2028,
              contractExpiryDayOfYear: 220,
              attributes,
              hiddenInformationRevealed: playerProfileHiddenInformationRevealed,
              roleScores: staffRoleIds.map((roleId, index) => ({
                roleId,
                displayName: roleId
                  .split("_")
                  .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
                  .join(" "),
                score: 100 - index,
              })),
            };
          }

          if (cmd === "set_hidden_information_revealed") {
            if (typeof args?.revealed !== "boolean") {
              throw new Error("Missing revealed state");
            }
            playerProfileHiddenInformationRevealed = args.revealed;
            window.localStorage.setItem(
              "player-profile-hidden-information",
              String(playerProfileHiddenInformationRevealed),
            );
            return playerProfileHiddenInformationRevealed;
          }

          if (cmd === "boost_wonderkid_mentality") {
            playerProfileMentalityUpdated = true;
            return {
              snapshotId: 1,
              operation: "wonderkid-mentality",
              previousCurrentAbility: null,
              currentAbility: null,
              potentialAbility: null,
              previousAmbition: 10,
              ambition: 20,
              previousProfessionalism: 15,
              professionalism: 15,
              previousDetermination: 8,
              determination: 18,
            };
          }

          if (cmd === "boost_squad_current_ability") {
            await resolveSquadBoost(args);
            return {
              updated: 2,
              skipped: 0,
              failed: 0,
              recoveryRequired: false,
              recoveryMessage: null,
            };
          }

          if (cmd === "boost_squad_wonderkid_mentality") {
            await resolveSquadBoost(args);
            return {
              updated: 2,
              skipped: 0,
              failed: 0,
              recoveryRequired: false,
              recoveryMessage: null,
            };
          }

          if (cmd === "get_planner_club_family") {
            return squadOverview
              ? { primaryClub: "Barcelona", sources: [] }
              : { primaryClub: null, sources: [] };
          }

          if (cmd === "list_squad_players") {
            const sortBy = args?.sortBy || "ca";
            const sortDir = args?.sortDir || "desc";
            const offset = Number.isInteger(args?.offset)
              ? Math.max(0, args.offset)
              : 0;
            const limit = Number.isInteger(args?.limit)
              ? Math.min(200, Math.max(1, args.limit))
              : 50;
            if (
              squadPageFailure &&
              offset >= 50 &&
              !squadPageFailureTriggered
            ) {
              squadPageFailureTriggered = true;
              throw new Error("Could not load the next squad page.");
            }
            const sorted = [...squadPlayers].sort((left, right) => {
              const values = {
                name: [left.name, right.name],
                age: [left.age, right.age],
                nationality: [left.nationalities.join(", "), right.nationalities.join(", ")],
                club: [left.club || "", right.club || ""],
                division: [left.division || "", right.division || ""],
                ca: [left.ca, right.ca],
                pa: [left.pa, right.pa],
                value: [left.marketValueGbp || -1, right.marketValueGbp || -1],
              }[sortBy] || [left.ca, right.ca];
              const comparison = typeof values[0] === "string"
                ? values[0].localeCompare(values[1])
                : values[0] - values[1];
              if (comparison === 0) return left.uid - right.uid;
              return sortDir === "asc" ? comparison : -comparison;
            });
            return {
              players: sorted.slice(offset, offset + limit),
              total: sorted.length,
            };
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

          if (cmd === "save_planner_teams") {
            if (!Array.isArray(args?.teams) || args.teams.length < 1 || args.teams.length > 3) {
              throw new Error("Planner configuration must contain one to three teams");
            }
            const inputs = args.teams.map((input) => {
              if (
                !["senior", "reserves", "youth"].includes(input?.team) ||
                typeof input?.displayName !== "string"
              ) {
                throw new Error("Invalid planner team settings");
              }
              return {
                team: input.team,
                displayName: input.displayName.trim(),
              };
            });
            const removedPopulated = plannerDepth.teams.some(
              (team) =>
                !inputs.some((input) => input.team === team.team) &&
                team.strings.some((plannerString) => plannerString.assignments.length > 0),
            );
            if (removedPopulated && args?.confirmPopulatedRemoval !== true) {
              throw new Error("Removing populated planner teams requires confirmation");
            }
            let nextStringId =
              Math.max(
                0,
                ...plannerDepth.teams.flatMap((team) =>
                  team.strings.map((plannerString) => plannerString.id),
                ),
              ) + 1;
            plannerDepth.teams = ["senior", "reserves", "youth"]
              .filter((team) => inputs.some((input) => input.team === team))
              .map((team) => {
                const existing = plannerDepth.teams.find(
                  (candidate) => candidate.team === team,
                );
                const input = inputs.find((candidate) => candidate.team === team);
                return existing
                  ? { ...existing, displayName: input.displayName }
                  : {
                      team,
                      displayName: input.displayName,
                      strings: [{ id: nextStringId++, stringOrder: 0, assignments: [] }],
                    };
              });
            return plannerDepth;
          }

          if (cmd === "optimize_planner_depth") {
            if (
              args?.scoreBasis !== "current" &&
              args?.scoreBasis !== "potential"
            ) {
              throw new Error("Optimizer requires a valid score basis");
            }
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
                potentialCombinedScore: 91,
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

          if (cmd === "clear_planner_depth") {
            if (args?.confirmed !== true) {
              throw new Error("Clearing all squads requires confirmation");
            }
            for (const team of plannerDepth.teams) {
              for (const plannerString of team.strings) {
                plannerString.assignments = [];
              }
            }
            return plannerDepth;
          }

          if (cmd === "get_planner_tactic_options") {
            return {
              placements: ["GK", "DL", "DCR", "DC", "DCL", "DR", "DMCR", "DM", "DMCL", "MCR", "MC", "MCL", "ML", "MR", "AML", "AMCR", "AMC", "AMCL", "AMR", "STCR", "STC", "STCL"],
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
            const loadedSnapshot = {
              id: 1,
              saveId: 1,
              schemaVersion: 6,
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
            };
            return {
              requestId: "req-smoke",
              playersFound: 0,
              scanTruncated: false,
              maxAccepted: null,
              timings: { scanMs: 0, ingestMs: 0, totalMs: 0 },
              storedSnapshot: loadedSnapshot,
              effectiveSnapshot: loadedSnapshot,
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
