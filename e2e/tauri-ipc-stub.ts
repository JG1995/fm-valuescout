import type { Page } from "@playwright/test";
import { VISIBLE_ATTRIBUTE_KEYS } from "../src/utils/player-attributes";

export type SmokeStubOptions = {
  academyWorkspace?: boolean;
  academyLargeCohort?: boolean;
  csvImportFormat?: "youthTracker";
  plannerSnapshot?: boolean;
  plannerPotentialScores?: boolean;
  playerTableRowCount?: number;
  squadPageFailure?: boolean;
  squadOverview?: boolean;
  playerProfile?: boolean;
  playerProfileLayout?: boolean;
  moneyballSearch?: boolean;
  shortlistSearch?: boolean;
  staffWorkspace?: boolean;
  staffShortlist?: boolean;
  staffAssignment?: boolean;
  staffAssignmentSenior?: boolean;
  staffFamily?: "configured" | "none";
  snapshotHistory?: boolean;
};

export async function stubTauriIpc(page: Page, options: SmokeStubOptions = {}) {
  const academyWorkspace = options.academyWorkspace ?? false;
  const csvImportFormat = options.csvImportFormat ?? null;
  const plannerSnapshot = options.plannerSnapshot ?? false;
  const plannerPotentialScores = options.plannerPotentialScores ?? false;
  const playerTableRowCount = options.playerTableRowCount ?? null;
  const squadPageFailure = options.squadPageFailure ?? false;
  const squadOverview = options.squadOverview ?? false;
  const playerProfile = options.playerProfile ?? false;
  const profileAttributes = options.playerProfileLayout
    ? Object.fromEntries(
        VISIBLE_ATTRIBUTE_KEYS.map((key, index) => [key, 10 + (index % 11)]),
      )
    : {};
  const moneyballSearch = options.moneyballSearch ?? false;
  const shortlistSearch = options.shortlistSearch ?? false;
  const staffWorkspace = options.staffWorkspace ?? false;
  const staffShortlist = options.staffShortlist ?? false;
  const staffAssignment = options.staffAssignment ?? false;
  const staffAssignmentSenior = options.staffAssignmentSenior ?? true;
  const staffFamilyConfigured = options.staffFamily !== "none";
  const snapshotHistory = options.snapshotHistory ?? false;
  await page.addInitScript({
    content: `
      const academyWorkspace = ${academyWorkspace ? "true" : "false"};
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
      const playerProfileLayout = ${options.playerProfileLayout ? "true" : "false"};
      const profileAttributes = ${JSON.stringify(profileAttributes)};
      const moneyballSearch = ${moneyballSearch ? "true" : "false"};
      const staffWorkspace = ${staffWorkspace ? "true" : "false"};
      const staffShortlist = ${staffShortlist ? "true" : "false"};
      const staffAssignment = ${staffAssignment ? "true" : "false"};
      const staffAssignmentSenior = ${staffAssignmentSenior ? "true" : "false"};
      const staffFamilyConfigured = ${staffFamilyConfigured ? "true" : "false"};
      const snapshotHistoryEnabled = ${snapshotHistory ? "true" : "false"};
      const shortlistSearchEnabled = ${shortlistSearch ? "true" : "false"};
      let nextSaveId = 2;
      let clubDnaDefinition = null;
      let moneyballStatsGameDate = moneyballSearch ? "2026-07-30" : null;
      let moneyballLeagueAdjusted = true;
      let saves = [{
        id: 1,
        contextToken: "save-token-1",
        name: "Default save",
        isActive: true,
        createdAtUtc: "2026-07-28T12:00:00.000Z",
        updatedAtUtc: "2026-07-28T12:00:00.000Z",
        linkedSaveFileName: null,
        // Loans and Targets restrict Search to different players.
        playerShortlists: shortlistSearchEnabled ? [
          { id: 2, name: "Loans", fileName: "Loans.fmf" },
          { id: 1, name: "Targets", fileName: "Targets.fmf" },
        ] : [],
      }];
      let nextShortlistId = 3;
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
      const academyClasses = academyWorkspace
        ? [{ id: 7, classYear: 2026, isAutomatic: false, memberCount: 2 }]
        : [];
      const academyMembers = academyWorkspace
        ? [
            {
              playerUid: 42,
              lastKnownName: "Alex Scout",
              currentName: "Alex Scout",
              state: "resolved",
              age: 18,
              nationalities: ["ENG"],
              positions: { ST: 20 },
              currentClub: "Barcelona U19",
              parentClub: null,
              teamLevel: "youth",
              pa: 170,
              determination: 16,
              heightCm: 183,
              preferredFoot: "right",
              reportedCareerAppearances: 8,
              goals: 3,
              assists: 2,
              internationalCaps: 1,
              outcome: null,
              isGraduate: true,
            },
            {
              playerUid: 99,
              lastKnownName: "Jamie Prospect",
              currentName: "Jamie Prospect",
              state: "resolved",
              age: 17,
              nationalities: ["ESP"],
              positions: { MC: 18 },
              currentClub: "Barcelona U19",
              parentClub: null,
              teamLevel: "youth",
              pa: 155,
              determination: 14,
              heightCm: 177,
              preferredFoot: "left",
              reportedCareerAppearances: 0,
              goals: 0,
              assists: 0,
              internationalCaps: 0,
              outcome: null,
              isGraduate: false,
            },
          ]
        : [];
      if (academyWorkspace && ${options.academyLargeCohort ? "true" : "false"}) {
        academyMembers.push(...Array.from({ length: 48 }, (_, index) => {
          const name = index === 0
            ? "Alexandros Papadopoulos — Кириллица"
            : "Academy prospect " + String(index + 3).padStart(3, "0");
          const unresolved = index === 47;
          return {
            ...academyMembers[1],
            playerUid: index + 100,
            lastKnownName: name,
            currentName: unresolved ? null : name,
            state: unresolved ? "unresolved" : index === 0 ? "departed" : "resolved",
            currentClub: unresolved ? null : index === 0
              ? "Club Deportivo Internacional de Desarrollo Juvenil Barcelona"
              : "Barcelona U19",
            reportedCareerAppearances: unresolved ? null : 0,
            goals: unresolved ? null : 0,
            assists: unresolved ? null : 0,
            internationalCaps: unresolved ? null : 0,
            isGraduate: unresolved ? null : false,
          };
        }));
        academyClasses[0].memberCount = academyMembers.length;
      }
      let staffAssignmentSnapshotToken = "snapshot-token-1";
      const staffAssignmentSnapshot = () => ({
        id: 1,
        contextToken: staffAssignmentSnapshotToken,
        saveId: 1,
        customName: null,
        gameDate: "2026-08-01",
        gameDateSource: "inGame",
        playerCount: 4,
        loadedAtUtc: "2026-07-28T15:00:00.000Z",
        isCurrent: true,
      });
      const staffAssignmentTeams = [
        ...(staffAssignmentSenior
          ? [{ team: "senior", displayName: "First Team" }]
          : []),
        { team: "reserves", displayName: "Reserves" },
        { team: "youth", displayName: "Youth" },
      ];
      const staffAssignmentTeamJobs = [
        { jobId: "manager", jobLabel: "Manager", section: "coaching" },
        { jobId: "assistant_manager", jobLabel: "Assistant Manager", section: "coaching" },
        { jobId: "coaches", jobLabel: "Coaches", section: "coaching" },
        { jobId: "performance_analyst", jobLabel: "Performance Analyst", section: "coaching" },
        { jobId: "physio", jobLabel: "Physio", section: "medical" },
        { jobId: "sports_scientist", jobLabel: "Sports Scientist", section: "medical" },
      ];
      const staffAssignmentClubJobs = [
        { jobId: "head_of_youth_development", jobLabel: "Head of Youth Development", section: "coaching", maxSlotCount: 1 },
        { jobId: "head_performance_analyst", jobLabel: "Head Performance Analyst", section: "coaching", maxSlotCount: 1 },
        { jobId: "set_piece_coach", jobLabel: "Set Piece Coach", section: "coaching", maxSlotCount: 1 },
        { jobId: "director_of_football", jobLabel: "Director of Football", section: "recruitment", maxSlotCount: 1 },
        { jobId: "chief_scout", jobLabel: "Chief Scout", section: "recruitment", maxSlotCount: 1 },
        { jobId: "technical_director", jobLabel: "Technical Director", section: "recruitment", maxSlotCount: 1 },
        { jobId: "scout", jobLabel: "Scout", section: "recruitment", maxSlotCount: 50 },
        { jobId: "recruitment_analyst", jobLabel: "Recruitment Analyst", section: "recruitment", maxSlotCount: 50 },
        { jobId: "loan_manager", jobLabel: "Loan Manager", section: "recruitment", maxSlotCount: 1 },
        { jobId: "head_physio", jobLabel: "Head Physio", section: "medical", maxSlotCount: 1 },
        { jobId: "head_sports_science", jobLabel: "Head of Sports Science", section: "medical", maxSlotCount: 1 },
      ];
      const staffAssignmentJobDisplayOrder = [
        "manager",
        "assistant_manager",
        "coaches",
        "set_piece_coach",
        "head_of_youth_development",
        "head_performance_analyst",
        "performance_analyst",
        "director_of_football",
        "chief_scout",
        "technical_director",
        "scout",
        "recruitment_analyst",
        "loan_manager",
        "head_physio",
        "physio",
        "head_sports_science",
        "sports_scientist",
      ];
      let staffAssignmentTargets = [
        ...staffAssignmentTeams.flatMap(({ team }) =>
          staffAssignmentTeamJobs
            .filter(({ jobId }) => team !== "senior" || jobId !== "manager")
            .map((job) => ({
              scope: team,
              ...job,
              maxSlotCount: 50,
              slotCount: 0,
            })),
        ),
        ...staffAssignmentClubJobs.map((job) => ({
          scope: "club",
          ...job,
          slotCount: 0,
        })),
      ];
      const staffAssignmentScopeOrder = {
        senior: 0,
        club: staffAssignmentSenior ? 0 : 3,
        reserves: 1,
        youth: 2,
      };
      staffAssignmentTargets.sort((left, right) =>
        staffAssignmentScopeOrder[left.scope] - staffAssignmentScopeOrder[right.scope]
        || staffAssignmentJobDisplayOrder.indexOf(left.jobId) - staffAssignmentJobDisplayOrder.indexOf(right.jobId));
      const staffAssignmentTargetResponse = () => ({
        teams: staffAssignmentTeams.map((team) => ({ ...team })),
        targets: staffAssignmentTargets.map((target) => ({ ...target })),
      });
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
        contextToken: snapshot.contextToken,
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
        moneyballStatsGameDate,
      });
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
          customMetricKeys:
            laneId === "goalkeeper" ? ["save_ratio", "clean_sheets_ratio"] : [],
        })),
      };
      const plannerTacticStore = new Map([
        ["1:save-token-1", JSON.parse(JSON.stringify(plannerTactic))],
      ]);
      function plannerContextKey(saveId, contextToken) {
        return saveId + ":" + contextToken;
      }
      function validatePlannerTacticContext(args) {
        if (!args || typeof args.saveId !== "number" || typeof args.contextToken !== "string") {
          throw new Error("Invalid planner tactic request");
        }
        return { saveId: args.saveId, contextToken: args.contextToken };
      }
      function ensurePlannerSaveContext(saveId, contextToken) {
        const save = saves.find((candidate) => candidate.id === saveId);
        if (!save) {
          throw "Save " + saveId + " not found";
        }
        if (save.contextToken !== contextToken) {
          throw "Save changed or no longer exists";
        }
      }
      const plannerTeamNames = {
        senior: "Senior",
        reserves: "Reserves",
        youth: "Youth",
      };
      const ordinalStringLabel = (stringOrder) => {
        const number = stringOrder + 1;
        const suffix =
          number % 100 >= 11 && number % 100 <= 13
            ? "th"
            : number % 10 === 1
              ? "st"
              : number % 10 === 2
                ? "nd"
                : number % 10 === 3
                  ? "rd"
                  : "th";
        return number + suffix + " string";
      };
      const plannerDepth = {
        tactic: plannerTactic,
        teams: ["senior", "reserves", "youth"].map((team, index) => ({
          team,
          displayName: plannerTeamNames[team],
          maxAge: { senior: null, reserves: 23, youth: 18 }[team],
          strings: [{
            id: index + 1,
            stringOrder: 0,
            displayName: ordinalStringLabel(0),
            assignments: plannerPotentialScores && team === "senior" ? [{
              id: 77,
              laneId: "goalkeeper",
              playerUid: 77,
              lastKnownName: "Potential Keeper",
              currentName: "Potential Keeper",
              state: "resolved",
              age: 19,
              pinned: false,
              overAge: false,
              combinedScore: 82,
              expectedCombinedScore: 88,
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
          currentClubUid: 9001,
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
          currentClubUid: 9001,
          division: "La Liga",
          ca: 150,
          pa: 165,
          marketValueGbp: 12000000,
        },
      ] : [];
      const moneyballPlayers = moneyballSearch ? Array.from(
        { length: 101 },
        (_, index) => ({
          uid: index + 1,
          name: "Moneyball player " + String(index + 1).padStart(3, "0"),
          age: 20 + (index % 12),
          birthYear: 2000,
          birthDayOfYear: index + 1,
          nationalities: ["ENG"],
          club: "Moneyball FC",
          currentClubUid: 9002,
          division: "Premier Division",
          ca: 150 - (index % 40),
          pa: 170 - (index % 30),
          marketValueGbp: 1000000 + index * 10000,
          dynamicValues: {
            "moneyball.minutes": 900 + index * 10,
            "moneyball.average_rating": 6.5 + (index % 20) / 10,
            "moneyball.goals_per_90": 0.1 + (index % 10) / 10,
            "moneyball.assists_per_90": 0.05 + (index % 8) / 10,
            "moneyball.xg_per_90": 0.08 + (index % 9) / 10,
            "moneyball.xa_per_90": 0.04 + (index % 7) / 10,
            "moneyball_role.wbl_wbr_wing_back_ip":
              index === 0 ? 0 : index === 1 ? null : 45 + (index % 55),
            "moneyball_role.dl_dr_wing_back_ip":
              index === 2 ? null : 55 + (index % 45),
          },
          moneyballPercentiles: {
            "moneyball.average_rating": 50 + (index % 50),
            "moneyball.goals_per_90": 50 + (index % 50),
            "moneyball.assists_per_90": 50 + (index % 50),
            "moneyball.xg_per_90": 50 + (index % 50),
            "moneyball.xa_per_90": 50 + (index % 50),
          },
        }),
      ) : [];
      const shortlistPlayers = shortlistSearchEnabled ? Array.from(
        { length: 24 },
        (_, index) => ({
          uid: 1001 + index,
          name: "Shortlist player " + String(index + 1).padStart(3, "0"),
          age: 22 + (index % 10),
          birthYear: 2000,
          birthDayOfYear: index + 1,
          nationalities: ["ENG"],
          club: "Shortlist FC",
          currentClubUid: 9003,
          division: "Premier Division",
          ca: 165 - index,
          pa: 185 - index,
          marketValueGbp: 5000000 + index * 50000,
          dynamicValues: {
            "role.deep_lying_playmaker_ip": 85 - index,
            "attr.Acceleration": 12 + (index % 8),
            club_dna: 75 - index,
          },
        }),
      ) : [];
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
          currentClubUid: 9001,
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
        await new Promise((resolve) => setTimeout(resolve, 200));
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
          currentClubUid: 9001,
          division: "La Liga",
          ca: 200 - index,
          pa: 210 - index,
          marketValueGbp: 16000000,
        }));
      }
      const managedClubOptions = [
        { clubName: "Barcelona", clubUid: 100 },
        { clubName: "Barca Athletic", clubUid: 101 },
        { clubName: "Barcelona U19", clubUid: 102 },
      ];
      let managedClub = squadOverview
        ? { clubName: "Barcelona", clubUid: 100 }
        : null;

      // The dashboard's XI and candidates: [laneId, IP position, IP role, OOP
      // position, OOP role, incumbent, General score, Moneyball score, cover].
      const dashboardSlots = [
        ["goalkeeper", "GK", "goalkeeper_ip", "GK", "line_holding_keeper_oop", "Marc-André ter Stegen", 86, 78, "Iñaki Peña"],
        ["right_back", "DR", "full_back_ip", "DR", "holding_full_back_oop", "Jules Koundé", 82, 74, "Héctor Fort"],
        ["right_centre_back", "DCL", "centre_back_ip", "DCL", "covering_centre_back_oop", "Ronald Araújo", 84, 71, "Andreas Christensen"],
        ["left_centre_back", "DCR", "centre_back_ip", "DCR", "covering_centre_back_oop", "Pau Cubarsí", 79, 69, null],
        ["left_back", "DL", "full_back_ip", "DL", "holding_full_back_oop", "Alejandro Balde", 68, 58, "Gerard Martín"],
        ["defensive_midfielder", "DM", "defensive_midfielder_ip", "DM", "screening_defensive_midfielder_oop", "Marc Casadó", 74, 66, "Marc Bernal"],
        ["right_central_midfielder", "MCL", "central_midfielder_ip", "MCL", "pressing_central_midfielder_oop", "Pedri", 91, 88, "Fermín López"],
        ["left_central_midfielder", "MCR", "central_midfielder_ip", "MCR", "pressing_central_midfielder_oop", "Frenkie de Jong", 85, 80, "Gavi"],
        ["right_winger", "AMR", "winger_ip", "MR", "tracking_wide_midfielder_oop", "Lamine Yamal", 93, 95, "Ferran Torres"],
        ["left_winger", "AML", "winger_ip", "ML", "tracking_wide_midfielder_oop", "Raphinha", 88, 90, "Ansu Fati"],
        ["centre_forward", "STC", "centre_forward_ip", "STC", "central_outlet_centre_forward_oop", null, null, null, null],
      ];
      // [uid, name, club, age, value, lane, General score, Moneyball score, minutes, shortlist].
      const dashboardTargets = [
        [501, "Viktor Gyökeres", "Sporting CP", 27, 62000000, "centre_forward", 87, 91, 2890, 1],
        [502, "Nuno Mendes", "Paris Saint-Germain", 23, 54000000, "left_back", 84, 79, 2410, 1],
        [503, "Alejandro Grimaldo García", "Bayer 04 Leverkusen", 29, 31000000, "left_back", 81, 83, 2975, 1],
        [504, "Martín Zubimendi", "Real Sociedad", 26, 51000000, "defensive_midfielder", 83, 77, 3012, 1],
        [505, "Jonathan Tah", "Bayer 04 Leverkusen", 29, 24000000, "left_centre_back", 85, 76, 2760, 2],
        [506, "Benjamin Šeško", "RB Leipzig", 22, 58000000, "centre_forward", 82, 74, 2104, 2],
        [507, "Giorgi Mamardashvili", "Valencia", 24, 38000000, "goalkeeper", 88, 81, 3240, 2],
        [508, "Ђорђе Петровић", "Chelsea", 25, 19500000, "goalkeeper", 87, 72, 1620, 1],
        [509, "Jeremie Frimpong", "Bayer 04 Leverkusen", 24, 43000000, "right_back", 85, 86, 2688, 1],
        [510, "Κωνσταντίνος Κουλιεράκης", "PAOK", 21, 12500000, "left_centre_back", 81, 73, 2455, 2],
        [511, "Rayan Aït-Nouri", "Wolverhampton Wanderers", 23, 33000000, "left_back", 80, 75, 2890, 2],
        [512, "Nguyễn Đình Bắc", null, 20, 750000, "centre_forward", 61, null, null, 2],
      ];
      const dashboardSigningsFor = (request) => {
        const moneyball = request?.mode === "moneyball";
        if (moneyball && moneyballStatsGameDate === null) {
          return {
            tacticIsDefault: true,
            scoresAvailable: false,
            slots: [],
            shortlistedPlayers: 0,
            totalRecommendations: 0,
            recommendations: [],
          };
        }
        const slots = dashboardSlots.map((slot, index) => {
          const score = moneyball ? slot[7] : slot[6];
          return {
            laneId: slot[0],
            ipPosition: slot[1],
            ipRoleId: slot[2],
            oopPosition: slot[3],
            oopRoleId: slot[4],
            scoreBasis: moneyball
              ? slot[0] === "centre_forward" ? "moneyball_custom" : "moneyball_roles"
              : "attributes",
            incumbent: slot[5] === null ? null : {
              uid: 300 + index,
              name: slot[5],
              score,
              source: index < 4 ? "planner" : "best_fit",
            },
            cover: slot[8] === null ? null : {
              uid: 400 + index,
              name: slot[8],
              score: Math.max(1, score - 9 - index),
            },
            unscoredPlannerPlayer:
              moneyball && slot[0] === "left_centre_back" ? "Iñigo Martínez" : null,
          };
        });
        const incumbentScore = (laneId) =>
          slots.find((slot) => slot.laneId === laneId)?.incumbent?.score ?? 0;
        const shortlistHasTargets = saves
          .find((save) => save.isActive)
          ?.playerShortlists.some(
            (shortlist) =>
              request?.shortlistId == null || shortlist.id === request.shortlistId,
          );
        const candidates = shortlistHasTargets
          ? dashboardTargets.filter(
              (target) =>
                request?.shortlistId == null || target[9] === request.shortlistId,
            )
          : [];
        const improving = candidates
          .map((target) => {
            const score = moneyball ? target[7] : target[6];
            return score === null ? null : {
              uid: target[0],
              name: target[1],
              age: target[3],
              club: target[2],
              clubUid: target[2] === null ? null : 9000 + target[0],
              marketValueGbp: target[4],
              minutes: moneyball ? target[8] : null,
              laneId: target[5],
              score,
              improvement: score - incumbentScore(target[5]),
              otherLanes:
                target[5] === "left_centre_back" && score > incumbentScore("right_centre_back")
                  ? [{ laneId: "right_centre_back", improvement: score - incumbentScore("right_centre_back") }]
                  : [],
            };
          })
          .filter((target) => target !== null && target.improvement > 0)
          .sort(
            (left, right) =>
              right.improvement - left.improvement || right.score - left.score,
          );
        const recommendations = improving.filter(
          (target) => request?.laneId == null || target.laneId === request.laneId,
        );
        for (const slot of slots) {
          const best = improving.find((target) => target.laneId === slot.laneId);
          slot.bestSigning = best ? { ...best, otherLanes: [] } : null;
        }
        return {
          tacticIsDefault: true,
          scoresAvailable: true,
          slots,
          shortlistedPlayers: candidates.length,
          totalRecommendations: recommendations.length,
          recommendations,
        };
      };
      const dashboardExpiringContracts = {
        windowMonths: 12,
        total: 7,
        players: [
          [301, "Jules Koundé", 27, 158, 52000000, 210000, "2026-12-31", 153],
          [401, "Héctor Fort", 19, 121, 8500000, 14000, "2027-01-31", 184],
          [304, "Alejandro Balde", 22, 149, 41000000, 95000, "2027-06-30", 334],
          [408, "Ferran Torres", 26, 151, 27000000, 180000, "2027-06-30", 334],
          [409, "Ansu Fati", 23, 144, null, 230000, "2027-06-30", 334],
          [410, "Wojciech Szczęsny", 36, 139, 750000, null, "2027-06-30", 334],
        ].map((player) => ({
          uid: player[0],
          name: player[1],
          age: player[2],
          ca: player[3],
          marketValueGbp: player[4],
          weeklyWageGbp: player[5],
          expiryDate: player[6],
          daysRemaining: player[7],
        })),
      };
      // [uid, name, age, slot index, current score, potential score].
      const dashboardYoungPlayers = [
        [308, "Lamine Yamal", 18, 8, 93, 97],
        [303, "Pau Cubarsí", 19, 3, 79, 92],
        [405, "Marc Bernal", 18, 5, 62, 86],
        [401, "Héctor Fort", 19, 1, 64, 81],
        [416, "Dro Fernández", 18, 7, 57, 80],
        [411, "Toni Fernández Casino", 17, 9, 48, 79],
        [412, "Guille Fernández", 17, 6, 51, 77],
        [413, "Diego Kochen", 20, 0, 55, 76],
        [414, "Andrés Cuenca", 19, 2, 58, 75],
        [415, "Jofre Torrents", 19, 4, 54, 74],
      ];
      const dashboardSlotPlacement = (index) => ({
        laneId: dashboardSlots[index][0],
        ipPosition: dashboardSlots[index][1],
        ipRoleId: dashboardSlots[index][2],
        oopPosition: dashboardSlots[index][3],
        oopRoleId: dashboardSlots[index][4],
      });
      const dashboardYoungPlayer = (player) => ({
        uid: player[0],
        name: player[1],
        age: player[2],
        currentScore: player[4],
        potentialScore: player[5],
      });
      // The youth XI has no striker, so its last slot stays empty.
      const dashboardProspects = {
        players: dashboardYoungPlayers.map((player) => ({
          ...dashboardYoungPlayer(player),
          ...dashboardSlotPlacement(player[3]),
        })),
        youthXi: dashboardSlots.map((_, index) => {
          const player = dashboardYoungPlayers.find((young) => young[3] === index);
          return {
            ...dashboardSlotPlacement(index),
            player: player ? dashboardYoungPlayer(player) : null,
          };
        }),
      };

      window.__TAURI_INTERNALS__ = {
        invoke: async (cmd, args) => {
          if (cmd === "plugin:dialog|open") {
            if (args?.options?.filters?.[0]?.extensions?.includes("fmf")) {
              return "/tmp/Smoke Targets.fmf";
            }
            return csvImportFormat ? "/tmp/smoke-import.csv" : null;
          }

          if (cmd === "get_graphics_status") {
            return {
              generation: 0,
              selected: false,
              candidate: { available: false, source: "absent" },
              summary: {
                configs: 0,
                mappings: 0,
                truncated: false,
                diagnostics: { configLimit: 0, entryLimit: 0, depthLimit: 0, mappingLimit: 0, configTooLarge: 0, configUnreadable: 0, malformedConfig: 0, invalidMapping: 0, sourceUnreadable: 0 },
              },
            };
          }

          if (cmd === "choose_graphics_root" || cmd === "clear_graphics_root" || cmd === "rescan_graphics") {
            return {
              generation: 1,
              selected: cmd !== "clear_graphics_root",
              candidate: { available: false, source: "absent" },
              summary: { configs: 0, mappings: 0, truncated: false, diagnostics: { configLimit: 0, entryLimit: 0, depthLimit: 0, mappingLimit: 0, configTooLarge: 0, configUnreadable: 0, malformedConfig: 0, invalidMapping: 0, sourceUnreadable: 0 } },
            };
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
              linkedSaveFileName: args?.saveFilePath
                ? args.saveFilePath.split(/[\\\\/]/).pop()
                : null,
              playerShortlists: [],
            };
            nextSaveId += 1;
            saves.push(created);
            return created;
          }

          if (cmd === "set_save_file_link") {
            const index = saves.findIndex((save) =>
              save.id === args?.saveId && save.contextToken === args?.contextToken
            );
            if (index < 0) throw "Save changed or no longer exists";
            const updated = {
              ...saves[index],
              linkedSaveFileName: args?.path ? args.path.split(/[\\\\/]/).pop() : null,
              updatedAtUtc: "2026-07-28T16:06:00.000Z",
            };
            saves[index] = updated;
            return updated;
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

          if (cmd === "update_snapshot_game_date") {
            const isCanonicalGameDate = (value) => {
              if (typeof value !== "string" || value.length !== 10 || value[4] !== "-" || value[7] !== "-") {
                return false;
              }
              for (let index = 0; index < 10; index += 1) {
                if (index === 4 || index === 7) continue;
                const code = value.charCodeAt(index);
                if (code < 48 || code > 57) return false;
              }
              const year = Number(value.slice(0, 4));
              const month = Number(value.slice(5, 7));
              const day = Number(value.slice(8, 10));
              if (year < 1 || month < 1 || month > 12 || day < 1) return false;
              const leap = year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0);
              const daysInMonth = month === 2
                ? (leap ? 29 : 28)
                : [4, 6, 9, 11].includes(month) ? 30 : 31;
              return day <= daysInMonth;
            };
            if (!isCanonicalGameDate(args?.gameDate)) {
              throw new Error("Game date must be a valid date in YYYY-MM-DD format");
            }
            const target = snapshots.find((snapshot) =>
              snapshot.id === args?.snapshotId &&
              snapshot.contextToken === args?.contextToken,
            );
            if (!target) throw new Error("Snapshot changed or no longer exists");
            const previousCurrentSnapshotId = snapshotsForSave(target.saveId)
              .find((snapshot) => snapshot.isCurrent)?.id ?? null;
            snapshots = snapshots.map((snapshot) => snapshot.id === target.id
              ? { ...snapshot, gameDate: args.gameDate }
              : snapshot);
            const currentSnapshotId = promoteCurrentSnapshot(target.saveId);
            return {
              snapshot: { ...snapshots.find((snapshot) => snapshot.id === target.id) },
              previousCurrentSnapshotId,
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
                  id: target.id,
                  contextToken: "save-token-" + target.id + "-replacement",
                  name: "Default save",
                  isActive: true,
                  createdAtUtc: "2026-07-28T16:20:00.000Z",
                  updatedAtUtc: "2026-07-28T16:20:00.000Z",
                  linkedSaveFileName: null,
                  playerShortlists: [],
                }];
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
            if (staffAssignment) return snapshotSummary(staffAssignmentSnapshot());
            return plannerSnapshot || squadOverview || playerProfile || staffWorkspace
              ? {
                  id: 1,
                  contextToken: "snapshot-token-1",
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
                  moneyballStatsGameDate,
                }
              : null;
          }

          if (cmd === "list_academy_classes") {
            return academyClasses.map((academyClass) => ({ ...academyClass }));
          }

          if (cmd === "get_academy_class") {
            const academyClass = academyClasses.find(
              (candidate) => candidate.id === args?.classId,
            );
            if (!academyClass) throw "Academy class " + args?.classId + " not found";
            return {
              ...academyClass,
              members: academyMembers.map((member) => ({ ...member })),
            };
          }

          if (cmd === "list_academy_candidates") {
            return academyWorkspace
              ? [{
                  playerUid: 100,
                  name: "Morgan Prospect",
                  age: 16,
                  positions: { AML: 17 },
                  currentClub: "Barcelona U19",
                }]
              : [];
          }

          if (cmd === "import_csv") {
            return {
              format: csvImportFormat,
              totalPlayers: 3,
              storedPlayers: 3,
              skippedPlayers: 0,
            };
          }

          if (cmd === "choose_player_shortlist" || cmd === "remove_player_shortlist") {
            const index = saves.findIndex((save) =>
              save.id === args?.saveId && save.contextToken === args?.contextToken
            );
            if (index < 0) throw { kind: "stale_context", fileName: null };
            const shortlists = saves[index].playerShortlists;
            if (cmd === "remove_player_shortlist") {
              saves[index] = {
                ...saves[index],
                playerShortlists: shortlists.filter(
                  (shortlist) => shortlist.id !== args?.shortlistId,
                ),
              };
              return null;
            }
            // Adding a linked file again updates its shortlist in place.
            const fileName = args.path.split(/[\\\\/]/).pop();
            const linked = shortlists.find((shortlist) => shortlist.fileName === fileName);
            const added = {
              id: linked ? linked.id : nextShortlistId++,
              name: "Smoke Targets",
              fileName,
            };
            saves[index] = {
              ...saves[index],
              playerShortlists: [
                ...shortlists.filter((shortlist) => shortlist.id !== added.id),
                added,
              ].sort((left, right) => left.name.localeCompare(right.name)),
            };
            return { ...added, totalPlayers: 3, matchedPlayers: 2 };
          }

          if (cmd === "search_players") {
            const offset = Number.isInteger(args?.offset)
              ? Math.max(0, args.offset)
              : 0;
            const limit = Number.isInteger(args?.limit)
              ? Math.min(200, Math.max(1, args.limit))
              : 50;
            const shortlistId = typeof args?.shortlistId === "number"
              ? args.shortlistId
              : null;
            if (
              shortlistId !== null &&
              !activeSave().playerShortlists.some((shortlist) => shortlist.id === shortlistId)
            ) {
              return { players: [], total: 0, state: "no_shortlist" };
            }
            if (shortlistId !== null && args?.searchView === "general") {
              // Loans holds the first three shortlist players; every other
              // shortlist holds all of them.
              const generalShortlistPlayers = shortlistId === 2
                ? shortlistPlayers.slice(0, 3)
                : shortlistPlayers;
              const filterRules = Array.isArray(args?.filters)
                ? args.filters
                : [];
              const filtered = filterRules.length > 0
                ? generalShortlistPlayers.filter((player) => {
                    const matches = filterRules.map((rule) => {
                      const field = rule.field;
                      let raw = null;
                      if (field === "ca") raw = player.ca;
                      else if (field === "pa") raw = player.pa;
                      else if (field === "value") raw = player.marketValueGbp;
                      else raw = player.dynamicValues?.[field] ?? null;
                      if (
                        typeof raw === "number" &&
                        rule.op === "between" &&
                        Array.isArray(rule.value)
                      ) {
                        return raw >= rule.value[0] && raw <= rule.value[1];
                      }
                      if (typeof raw !== "number" || typeof rule.value !== "number") {
                        if (typeof raw === "string" && typeof rule.value === "string") {
                          const hay = String(raw).toLowerCase();
                          const needle = String(rule.value).toLowerCase();
                          switch (rule.op) {
                            case "contains": return hay.includes(needle);
                            case "is": return hay === needle;
                            default: return false;
                          }
                        }
                        return false;
                      }
                      switch (rule.op) {
                        case "gt": return raw > rule.value;
                        case "gte": return raw >= rule.value;
                        case "lt": return raw < rule.value;
                        case "lte": return raw <= rule.value;
                        case "eq": return raw === rule.value;
                        case "neq": return raw !== rule.value;
                        default: return false;
                      }
                    });
                    return args?.filterCombine === "or"
                      ? matches.some(Boolean)
                      : matches.every(Boolean);
                  })
                : generalShortlistPlayers;
              const sorted = [...filtered].sort((left, right) => {
                const sortBy = typeof args?.sortBy === "string" ? args.sortBy : "ca";
                let leftVal = null;
                let rightVal = null;
                if (sortBy === "ca") { leftVal = left.ca; rightVal = right.ca; }
                else if (sortBy === "pa") { leftVal = left.pa; rightVal = right.pa; }
                else { leftVal = left.dynamicValues?.[sortBy] ?? null; rightVal = right.dynamicValues?.[sortBy] ?? null; }
                if (typeof leftVal !== "number" || typeof rightVal !== "number") {
                  const l = leftVal === null ? "" : String(leftVal);
                  const r = rightVal === null ? "" : String(rightVal);
                  const cmp = l.localeCompare(r);
                  if (cmp === 0) return left.uid - right.uid;
                  return args?.sortDir === "asc" ? cmp : -cmp;
                }
                if (leftVal === rightVal) return left.uid - right.uid;
                return args?.sortDir === "asc" ? leftVal - rightVal : rightVal - leftVal;
              });
              return {
                players: sorted.slice(offset, offset + limit),
                total: sorted.length,
              };
            }
            if (args?.searchView === "moneyball") {
              const filterRules = Array.isArray(args?.filters)
                ? args.filters
                : [];
              // Loans restricts Moneyball to players 001 and 003; every other
              // shortlist to the even players 002, 004, and 006.
              const shortlistUids = shortlistId === 2 ? [1, 3] : [2, 4, 6];
              const cohort = shortlistId !== null
                ? moneyballPlayers.filter((player) => shortlistUids.includes(player.uid))
                : moneyballPlayers;
              const filtered = filterRules.length > 0
                ? cohort.filter((player) => {
                    const matches = filterRules.map((rule) => {
                      const value = player.dynamicValues?.[rule.field];
                      if (
                        typeof value === "number" &&
                        rule.op === "between" &&
                        Array.isArray(rule.value)
                      ) {
                        return value >= rule.value[0] && value <= rule.value[1];
                      }
                      if (
                        typeof value !== "number" ||
                        typeof rule.value !== "number"
                      ) {
                        return false;
                      }
                      switch (rule.op) {
                        case "gt":
                          return value > rule.value;
                        case "gte":
                          return value >= rule.value;
                        case "lt":
                          return value < rule.value;
                        case "lte":
                          return value <= rule.value;
                        case "eq":
                          return value === rule.value;
                        case "neq":
                          return value !== rule.value;
                        default:
                          return false;
                      }
                    });
                    return args?.filterCombine === "or"
                      ? matches.some(Boolean)
                      : matches.every(Boolean);
                  })
                : cohort;
              const sorted = [...filtered];
              if (
                typeof args?.sortBy === "string" &&
                args.sortBy.startsWith("moneyball_role.")
              ) {
                sorted.sort((left, right) => {
                  const leftValue = left.dynamicValues?.[args.sortBy];
                  const rightValue = right.dynamicValues?.[args.sortBy];
                  const leftMissing = typeof leftValue !== "number";
                  const rightMissing = typeof rightValue !== "number";
                  if (leftMissing || rightMissing) {
                    if (leftMissing && rightMissing) {
                      return left.uid - right.uid;
                    }
                    return leftMissing ? 1 : -1;
                  }
                  const comparison = leftValue - rightValue;
                  if (comparison === 0) {
                    return left.uid - right.uid;
                  }
                  return args?.sortDir === "asc" ? comparison : -comparison;
                });
              }
              const players = args?.comparisonPool === "fullCsv"
                ? sorted.map((player) => ({
                    ...player,
                    moneyballPercentiles: {
                      ...player.moneyballPercentiles,
                      "moneyball.average_rating": 71,
                    },
                  }))
                : sorted;
              return {
                players: players.slice(offset, offset + limit),
                total: players.length,
              };
            }
            return squadOverview
              ? {
                  players: squadPlayers.slice(offset, offset + limit),
                  total: squadPlayers.length,
                }
              : { players: [], total: 0 };
          }

          if (cmd === "get_staff_assignment_targets") {
            if (args?.expectedSaveContextToken !== activeSave().contextToken) {
              throw new Error("Staff assignment save context changed");
            }
            return staffAssignmentTargetResponse();
          }

          if (cmd === "save_staff_assignment_targets") {
            if (args?.expectedSaveContextToken !== activeSave().contextToken) {
              throw new Error("Staff assignment save context changed");
            }
            if (!Array.isArray(args?.targets)) {
              throw new Error("Staff assignment targets are required");
            }
            staffAssignmentTargets = staffAssignmentTargets.map((target) => {
              const saved = args.targets.find(
                (candidate) =>
                  candidate?.scope === target.scope &&
                  candidate?.jobId === target.jobId,
              );
              return saved ? { ...target, slotCount: saved.slotCount } : target;
            });
            return staffAssignmentTargetResponse();
          }

          if (cmd === "optimize_staff_assignments") {
            if (
              args?.expectedSaveContextToken !== activeSave().contextToken ||
              args?.expectedSnapshotContextToken !== staffAssignmentSnapshotToken
            ) {
              throw new Error("Staff assignment context changed");
            }
            return {
              state: "ready",
              saveId: activeSave().id,
              saveContextToken: activeSave().contextToken,
              snapshotId: 1,
              snapshotContextToken: staffAssignmentSnapshotToken,
              joinedCandidateCount: 5,
              configuredSlotCount: 4,
              unsupportedPreferredJobCount: 1,
              slots: [
                {
                  kind: "recommendation",
                  scope: "senior",
                  scopeDisplayName: "First Team",
                  jobId: "assistant_manager",
                  jobLabel: "Assistant Manager",
                  slotNumber: 1,
                  uid: 101,
                  name: "Alex Assistant",
                  preferredJob: "Assistant Manager",
                  classification: "current_staff",
                  score: 82,
                  coachRequirement: null,
                },
                {
                  kind: "recommendation",
                  scope: "senior",
                  scopeDisplayName: "First Team",
                  jobId: "coaches",
                  jobLabel: "Coaches",
                  slotNumber: 1,
                  uid: 102,
                  name: "Coach Casey",
                  preferredJob: "Coach",
                  classification: "current_staff",
                  score: 79,
                  coachRequirement: "attacking_technical",
                },
                {
                  kind: "recommendation",
                  scope: "club",
                  scopeDisplayName: "Club",
                  jobId: "scout",
                  jobLabel: "Scout",
                  slotNumber: 1,
                  uid: 103,
                  name: "Riley Scout",
                  preferredJob: "Scout",
                  classification: "recruitment",
                  score: 74,
                  coachRequirement: null,
                },
                {
                  kind: "vacancy",
                  scope: "reserves",
                  scopeDisplayName: "Reserves",
                  jobId: "coaches",
                  jobLabel: "Coaches",
                  slotNumber: 2,
                  coachRequirement: "goalkeeping",
                  evidence: {
                    jobId: "coaches",
                    joinedCandidateCount: 1,
                    eligibleScoreCount: 0,
                    unavailableScoreCount: 1,
                  },
                },
              ],
              evidence: [
                { jobId: "manager", joinedCandidateCount: 1, eligibleScoreCount: 0, unavailableScoreCount: 1 },
                { jobId: "assistant_manager", joinedCandidateCount: 1, eligibleScoreCount: 1, unavailableScoreCount: 0 },
                { jobId: "coaches", joinedCandidateCount: 1, eligibleScoreCount: 1, unavailableScoreCount: 0 },
                { jobId: "set_piece_coach", joinedCandidateCount: 0, eligibleScoreCount: 0, unavailableScoreCount: 0 },
                { jobId: "head_performance_analyst", joinedCandidateCount: 0, eligibleScoreCount: 0, unavailableScoreCount: 0 },
                { jobId: "performance_analyst", joinedCandidateCount: 0, eligibleScoreCount: 0, unavailableScoreCount: 0 },
                { jobId: "head_physio", joinedCandidateCount: 0, eligibleScoreCount: 0, unavailableScoreCount: 0 },
                { jobId: "physio", joinedCandidateCount: 0, eligibleScoreCount: 0, unavailableScoreCount: 0 },
                { jobId: "head_sports_science", joinedCandidateCount: 0, eligibleScoreCount: 0, unavailableScoreCount: 0 },
                { jobId: "sports_scientist", joinedCandidateCount: 0, eligibleScoreCount: 0, unavailableScoreCount: 0 },
                { jobId: "head_of_youth_development", joinedCandidateCount: 0, eligibleScoreCount: 0, unavailableScoreCount: 0 },
                { jobId: "director_of_football", joinedCandidateCount: 0, eligibleScoreCount: 0, unavailableScoreCount: 0 },
                { jobId: "technical_director", joinedCandidateCount: 0, eligibleScoreCount: 0, unavailableScoreCount: 0 },
                { jobId: "loan_manager", joinedCandidateCount: 0, eligibleScoreCount: 0, unavailableScoreCount: 0 },
                { jobId: "chief_scout", joinedCandidateCount: 0, eligibleScoreCount: 0, unavailableScoreCount: 0 },
                { jobId: "scout", joinedCandidateCount: 1, eligibleScoreCount: 1, unavailableScoreCount: 0 },
              ],
            };
          }

          if (cmd === "search_staff") {
            if (args?.shortlistOnly === true) {
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
                  : "no_managed_club",
              staff: staffFamilyConfigured
                ? staffRows.slice(offset, offset + limit)
                : [],
              total: staffFamilyConfigured ? staffRows.length : 0,
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
              nationalities: ["England"],
              heightCm: 182,
              preferredFoot: "right",
              positions: {
                GK: 14,
                SW: 18,
                DL: 1,
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
                ...profileAttributes,
                Passing: 14,
                Determination: playerProfileMentalityUpdated ? 18 : 8,
              },
              potentialAttributes: { ...profileAttributes, Passing: 16 },
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
                  displayName: playerProfileLayout ? "Wide Covering Defensive Midfielder" : "Current Specialist",
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

          if (cmd === "refresh_season_stats") {
            moneyballStatsGameDate = "2027-01-01";
            return {
              status: "attached",
              matchedPlayers: 3,
              unmatchedPlayers: 0,
              gameDate: "2027-01-01",
              shortlists: { status: "notRead" },
              elapsedMs: 12,
            };
          }

          if (
            cmd === "get_moneyball_league_adjustment" ||
            cmd === "set_moneyball_league_adjustment"
          ) {
            if (cmd === "set_moneyball_league_adjustment") {
              moneyballLeagueAdjusted = args.enabled;
            }
            return {
              enabled: moneyballLeagueAdjusted,
              referenceDivision: "Premier Division",
            };
          }

          if (cmd === "get_player_moneyball") {
            return moneyballSearch ? {
              state: "ready",
              starts: 20,
              substituteAppearances: 4,
              minutes: 1200,
              statistics: {
                goals: 10,
                average_rating: 7.2,
                goals_per_90: 0.5,
                assists_per_90: 0.3,
                xg_per_90: 0.4,
                xa_per_90: 0.2,
              },
              percentiles: {
                goals: 83,
                average_rating: 83,
                goals_per_90: 76,
                assists_per_90: 68,
                xg_per_90: 72,
                xa_per_90: 65,
              },
              comparisonBasis: {
                kind: "available",
                naturalPositions: ["MC"],
                comparisonPlayerCount: 24,
              },
              // The stub's player is in the reference league, so his scores do not move.
              leagueAdjustment: moneyballLeagueAdjusted
                ? {
                    division: "Premier Division",
                    strength: 149,
                    estimated: false,
                    referenceDivision: "Premier Division",
                    referenceStrength: 149,
                    shift: 0,
                  }
                : null,
              roleCatalogVersion: 1,
              roleScores: [
                {
                  roleId: "mc_central_midfielder_ip",
                  displayName: "Central Midfielder",
                  phase: "in_possession",
                  positionFamily: "central_midfielder",
                  positionTags: ["MC"],
                  score: 86,
                  contributions: [
                    {
                      metricKey: "progressive_passes_per_90",
                      sourceLabel: "Progressive Passes per 90",
                      weight: 0.2,
                      direction: "higher",
                      percentile: 90,
                      weightedContribution: 18,
                      availableFromSource: true,
                    },
                    {
                      metricKey: "pass_completion_ratio",
                      sourceLabel: "Pass Completion Ratio",
                      weight: 0.25,
                      direction: "higher",
                      percentile: 80,
                      weightedContribution: 20,
                      availableFromSource: true,
                    },
                    {
                      metricKey: "distance_covered_per_90",
                      sourceLabel: "Distance Covered per 90",
                      weight: 0.2,
                      direction: "higher",
                      percentile: 90,
                      weightedContribution: 18,
                      availableFromSource: true,
                    },
                    {
                      metricKey: "tackles_completed_per_90",
                      sourceLabel: "Tackles Completed per 90",
                      weight: 0.2,
                      direction: "higher",
                      percentile: 80,
                      weightedContribution: 16,
                      availableFromSource: true,
                    },
                    {
                      metricKey: "interceptions_per_90",
                      sourceLabel: "Interceptions per 90",
                      weight: 0.15,
                      direction: "higher",
                      percentile: 90,
                      weightedContribution: 13.5,
                      availableFromSource: true,
                    },
                  ],
                },
                {
                  roleId: "mc_pressing_central_midfielder_oop",
                  displayName: "Pressing Central Midfielder",
                  phase: "out_of_possession",
                  positionFamily: "central_midfielder",
                  positionTags: ["MC"],
                  score: 64,
                  contributions: [
                    {
                      metricKey: "tackles_per_90",
                      sourceLabel: "Tackles per 90",
                      weight: 0.5,
                      direction: "higher",
                      percentile: 70,
                      weightedContribution: 35,
                      availableFromSource: true,
                    },
                    {
                      metricKey: "possession_lost_per_90",
                      sourceLabel: "Possession Lost per 90",
                      weight: 0.5,
                      direction: "lower",
                      percentile: 58,
                      weightedContribution: 29,
                      availableFromSource: true,
                    },
                  ],
                },
              ],
            } : { state: "noData" };
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

          if (cmd === "get_club_dna") {
            return clubDnaDefinition;
          }

          if (cmd === "set_club_dna") {
            const created = clubDnaDefinition === null;
            clubDnaDefinition = {
              attributeIds: Array.isArray(args?.attributeIds)
                ? [...args.attributeIds]
                : [],
            };
            return { definition: clubDnaDefinition, created };
          }

          if (cmd === "remove_club_dna") {
            const removed = clubDnaDefinition !== null;
            clubDnaDefinition = null;
            return { removed };
          }

          if (cmd === "get_managed_club") {
            return managedClub
              ? {
                  ...managedClub,
                  status: "available",
                  unclassifiedPlayerCount: 0,
                }
              : {
                  clubName: null,
                  clubUid: null,
                  status: "unconfigured",
                  unclassifiedPlayerCount: 0,
                };
          }

          if (cmd === "get_dashboard_signings") {
            return managedClub ? dashboardSigningsFor(args) : null;
          }

          if (cmd === "get_dashboard_expiring_contracts") {
            return managedClub ? dashboardExpiringContracts : null;
          }

          if (cmd === "get_dashboard_prospects") {
            return managedClub ? dashboardProspects : null;
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
            if (squadPageFailure && offset >= 50) {
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

          if (cmd === "list_managed_club_options") {
            return plannerSnapshot ? managedClubOptions : [];
          }

          if (cmd === "set_managed_club") {
            const selectedClub = managedClubOptions.find(
              (option) =>
                option.clubName === args?.clubName &&
                option.clubUid === args?.clubUid,
            );
            if (!selectedClub) {
              throw new Error("Managed club selection is not an available pair");
            }
            managedClub = { ...selectedClub };
            return {
              ...managedClub,
              status: "available",
              unclassifiedPlayerCount: 0,
            };
          }

          if (cmd === "get_planner_tactic") {
            const { saveId, contextToken } = validatePlannerTacticContext(args);
            ensurePlannerSaveContext(saveId, contextToken);
            const key = plannerContextKey(saveId, contextToken);
            return plannerTacticStore.get(key) ?? JSON.parse(JSON.stringify(plannerTactic));
          }

          if (cmd === "get_planner_depth") {
            return plannerDepth;
          }

          if (cmd === "get_planner_role_reference") {
            if (
              args?.phase !== "in_possession" &&
              args?.phase !== "out_of_possession"
            ) {
              throw new Error("Invalid planner role reference phase");
            }
            if (
              args?.scoreBasis !== "current" &&
              args?.scoreBasis !== "potential"
            ) {
              throw new Error("Invalid planner role reference score basis");
            }
            return {
              lanes: plannerTactic.lanes.map((lane) => ({
                laneId: lane.laneId,
                players:
                  lane.laneId === "goalkeeper"
                    ? [
                        {
                          playerUid: 77,
                          name: "Potential Keeper",
                          currentScore: 82,
                          potentialScore: 91,
                        },
                      ]
                    : lane.laneId === "left_back"
                      ? [
                          {
                            playerUid: 78,
                            name: "Potential Full-Back",
                            currentScore: 79,
                            potentialScore: 88,
                          },
                        ]
                      : [],
              })),
              noEligible: [],
            };
          }

          if (cmd === "get_planner_team_removal_impacts") {
            if (!Array.isArray(args?.teams)) {
              throw new Error("Invalid planner team settings");
            }
            const included = new Map(args.teams.map((team) => [team?.team, team]));
            const impacts = [];
            for (const team of plannerDepth.teams) {
              const input = included.get(team.team);
              if (!input) {
                impacts.push({
                  team: team.team,
                  displayName: team.displayName,
                  assignmentCount: team.strings.reduce(
                    (count, plannerString) =>
                      count + plannerString.assignments.length,
                    0,
                  ),
                  staffingTargets: [],
                  strings: team.strings.map((plannerString) => ({
                    stringId: plannerString.id,
                    displayName: plannerString.displayName,
                    assignmentCount: plannerString.assignments.length,
                  })),
                });
                continue;
              }
              const desiredIds = new Set(
                Array.isArray(input?.strings)
                  ? input.strings
                      .map((plannerString) => plannerString?.id)
                      .filter((id) => typeof id === "number")
                  : [],
              );
              const removed = team.strings.filter(
                (plannerString) => !desiredIds.has(plannerString.id),
              );
              if (removed.length > 0) {
                impacts.push({
                  team: team.team,
                  displayName: team.displayName,
                  assignmentCount: removed.reduce(
                    (count, plannerString) =>
                      count + plannerString.assignments.length,
                    0,
                  ),
                  staffingTargets: [],
                  strings: removed.map((plannerString) => ({
                    stringId: plannerString.id,
                    displayName: plannerString.displayName,
                    assignmentCount: plannerString.assignments.length,
                  })),
                });
              }
            }
            return impacts;
          }

          if (cmd === "save_planner_teams") {
            if (!Array.isArray(args?.teams) || args.teams.length < 1 || args.teams.length > 3) {
              throw new Error("Planner configuration must contain one to three teams");
            }
            const inputs = args.teams.map((input) => {
              if (
                !["senior", "reserves", "youth"].includes(input?.team) ||
                typeof input?.displayName !== "string" ||
                !Array.isArray(input?.strings)
              ) {
                throw new Error("Invalid planner team settings");
              }
              return {
                team: input.team,
                displayName: input.displayName.trim(),
                maxAge: input.maxAge ?? null,
                strings: input.strings.map((plannerString) => {
                  if (
                    (plannerString?.id !== null &&
                      typeof plannerString?.id !== "number") ||
                    typeof plannerString?.displayName !== "string"
                  ) {
                    throw new Error("Invalid planner team settings");
                  }
                  return {
                    id: plannerString.id,
                    displayName: plannerString.displayName.trim(),
                  };
                }),
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
                if (existing) {
                  const retained = new Map(
                    existing.strings.map((plannerString) => [
                      plannerString.id,
                      plannerString,
                    ]),
                  );
                  return {
                    ...existing,
                    displayName: input.displayName,
                    maxAge: input.maxAge,
                    strings: input.strings.map((plannerString, stringOrder) => {
                      if (plannerString.id === null) {
                        return {
                          id: nextStringId++,
                          stringOrder,
                          displayName:
                            plannerString.displayName ||
                            ordinalStringLabel(stringOrder),
                          assignments: [],
                        };
                      }
                      const current = retained.get(plannerString.id);
                      if (!current) {
                        throw new Error("Planner string not found");
                      }
                      return {
                        ...current,
                        stringOrder,
                        displayName: plannerString.displayName,
                      };
                    }),
                  };
                }
                return {
                      team,
                      displayName: input.displayName,
                      maxAge: input.maxAge,
                      strings: [{ id: nextStringId++, stringOrder: 0, displayName: ordinalStringLabel(0), assignments: [] }],
                    };
              });
            return plannerDepth;
          }

          if (cmd === "get_planner_unassigned_players") {
            const placed = new Set(
              plannerDepth.teams.flatMap((team) =>
                team.strings.flatMap((plannerString) =>
                  plannerString.assignments.map(
                    (assignment) => assignment.playerUid,
                  ),
                ),
              ),
            );
            // [uid, name, age, lane, current, expected].
            return [
              [501, "Senior Keeper", 29, "goalkeeper", 78, 78],
              [503, "Left-Footed Keeper", 24, "goalkeeper", 70, 73],
              [502, "Young Keeper", 17, "goalkeeper", 61, 80],
              [504, "Surplus Player", 31, null, null, null],
            ]
              .filter((player) => !placed.has(player[0]))
              .map((player) => ({
                playerUid: player[0],
                name: player[1],
                age: player[2],
                laneId: player[3],
                currentScore: player[4],
                expectedScore: player[5],
              }));
          }

          if (cmd === "get_planner_slot_candidates") {
            const team = plannerDepth.teams.find(
              (candidate) => candidate.team === args?.team,
            );
            if (!team) {
              throw new Error("Planner team not found");
            }
            const search = String(args?.search ?? "").toLowerCase();
            // [uid, name, age, current, expected, position fit, foot fit].
            return [
              [77, "Potential Keeper", 19, 82, 88, "suited", "matches"],
              [501, "Senior Keeper", 29, 78, 78, "suited", "matches"],
              [502, "Young Keeper", 17, 61, 80, "unfamiliar", "matches"],
              [503, "Left-Footed Keeper", 24, 70, 73, "suited", "mismatch"],
            ]
              .filter((candidate) => candidate[1].toLowerCase().includes(search))
              .map((candidate) => {
                const location = plannerDepth.teams
                  .flatMap((placedTeam) =>
                    placedTeam.strings.flatMap((plannerString) =>
                      plannerString.assignments.map((assignment) => ({
                        playerUid: assignment.playerUid,
                        team: placedTeam.team,
                        stringId: plannerString.id,
                        stringOrder: plannerString.stringOrder,
                        laneId: assignment.laneId,
                      })),
                    ),
                  )
                  .find((placed) => placed.playerUid === candidate[0]);
                return {
                  playerUid: candidate[0],
                  name: candidate[1],
                  currentClub: "Barcelona",
                  age: candidate[2],
                  overAge: team.maxAge !== null && candidate[2] > team.maxAge,
                  ipScore: candidate[3],
                  oopScore: candidate[3],
                  combinedScore: candidate[3],
                  expectedCombinedScore: candidate[4],
                  positionFit: candidate[5],
                  footFit: candidate[6],
                  assignmentLocation: location
                    ? {
                        team: location.team,
                        stringId: location.stringId,
                        stringOrder: location.stringOrder,
                        laneId: location.laneId,
                      }
                    : null,
                };
              });
          }

          if (
            cmd === "assign_planner_player" ||
            cmd === "move_planner_player" ||
            cmd === "clear_planner_assignment" ||
            cmd === "set_planner_assignment_pinned"
          ) {
            const strings = plannerDepth.teams.flatMap((team) => team.strings);
            const target = strings.find(
              (plannerString) => plannerString.id === args?.stringId,
            );
            if (!target) {
              throw new Error("Planner string not found");
            }
            if (cmd === "set_planner_assignment_pinned") {
              const assignment = target.assignments.find(
                (candidate) => candidate.laneId === args?.laneId,
              );
              if (!assignment) {
                throw new Error("Planner cell is empty");
              }
              assignment.pinned = args?.pinned === true;
              return plannerDepth;
            }
            const names = {
              77: "Potential Keeper",
              501: "Senior Keeper",
              502: "Young Keeper",
              503: "Left-Footed Keeper",
            };
            // A pick leaves his old cell and replaces the target's player.
            for (const plannerString of strings) {
              plannerString.assignments = plannerString.assignments.filter(
                (assignment) =>
                  assignment.playerUid !== args?.playerUid &&
                  !(
                    plannerString === target &&
                    assignment.laneId === args?.laneId
                  ),
              );
            }
            if (cmd !== "clear_planner_assignment") {
              target.assignments.push({
                id: 1000 + args.playerUid,
                laneId: args.laneId,
                playerUid: args.playerUid,
                lastKnownName: names[args.playerUid],
                currentName: names[args.playerUid],
                state: "resolved",
                age: 24,
                pinned: true,
                overAge: false,
                combinedScore: 70,
                expectedCombinedScore: 73,
                potentialCombinedScore: 76,
              });
            }
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
                age: 19,
                pinned: false,
                overAge: false,
                combinedScore: 82,
                expectedCombinedScore: 88,
                potentialCombinedScore: 91,
              },
            ];
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
            const { saveId, contextToken } = validatePlannerTacticContext(args);
            ensurePlannerSaveContext(saveId, contextToken);
            return {
              placements: ["GK", "DL", "DCR", "DC", "DCL", "DR", "WBL", "WBR", "DMCR", "DM", "DMCL", "MCR", "MC", "MCL", "ML", "MR", "AML", "AMCR", "AMC", "AMCL", "AMR", "STCR", "STC", "STCL"],
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
            const { saveId, contextToken } = validatePlannerTacticContext(args);
            ensurePlannerSaveContext(saveId, contextToken);
            if (!args?.tactic) throw new Error("Tactic is required");
            const key = plannerContextKey(saveId, contextToken);
            plannerTacticStore.set(key, JSON.parse(JSON.stringify(args.tactic)));
            return args.tactic;
          }

          if (cmd === "load_data") {
            if (staffAssignment) {
              staffAssignmentSnapshotToken = "snapshot-token-replacement";
            }
            const loadedSnapshot = staffAssignment
              ? snapshotSummary(staffAssignmentSnapshot())
              : {
              id: 1,
              contextToken: "snapshot-token-1",
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
              timings: { scanMs: 0, ingestMs: 0, totalMs: 0 },
              storedSnapshot: loadedSnapshot,
              effectiveSnapshot: loadedSnapshot,
              stats: { status: "notLinked", shortlists: { status: "notRead" }, elapsedMs: 0 },
              shortlists: { read: activeSave().playerShortlists.length, failed: [] },
            };
          }

          if (cmd === "get_bridge_install_status") {
            return {
              pluginsPath:
                "C:\\\\Program Files (x86)\\\\Steam\\\\steamapps\\\\common\\\\Football Manager 27\\\\BepInEx\\\\plugins",
              pluginPresent: false,
              bepinexPresent: true,
              pluginsDirPresent: true,
            };
          }

          if (cmd === "install_bridge_plugin") {
            return {
              pluginsPath:
                "C:\\\\Program Files (x86)\\\\Steam\\\\steamapps\\\\common\\\\Football Manager 27\\\\BepInEx\\\\plugins",
              pluginPresent: true,
              bepinexPresent: true,
              pluginsDirPresent: true,
            };
          }

          if (cmd === "remove_bridge_plugin") {
            return {
              pluginsPath:
                "C:\\\\Program Files (x86)\\\\Steam\\\\steamapps\\\\common\\\\Football Manager 27\\\\BepInEx\\\\plugins",
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
