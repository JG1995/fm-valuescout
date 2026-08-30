import { describe, expect, it } from "vitest";
import backendRoleDefinitionsJson from "../../src-tauri/src/features/moneyball/builtin_role_definitions_v1.json?raw";
import {
  MONEYBALL_ROLE_CATALOG,
  MONEYBALL_ROLE_CATALOG_VERSION,
  type MoneyballRolePhase,
  type MoneyballRolePositionFamily,
} from "./moneyball-role-catalog";
import { orderedPositions } from "./position-order";

type BackendRole = {
  id: string;
  display_name: string;
  phase: MoneyballRolePhase;
  position_family: MoneyballRolePositionFamily;
  position_tags: string[];
};

const BACKEND_ROLE_CATALOG = (
  JSON.parse(backendRoleDefinitionsJson) as { definitions: BackendRole[] }
).definitions;

const EXPECTED_ROLE_IDS = [
  "amc_attacking_midfielder_ip",
  "amc_advanced_playmaker_ip",
  "amc_free_role_ip",
  "amc_second_striker_ip",
  "amc_channel_midfielder_ip",
  "amc_attacking_midfielder_oop",
  "amc_tracking_attacking_midfielder_oop",
  "amc_central_outlet_attacking_midfielder_oop",
  "amc_splitting_outlet_attacking_midfielder_oop",
  "dc_centre_back_ip",
  "dc_advanced_centre_back_ip",
  "dc_ball_playing_centre_back_ip",
  "dc_no_nonsense_centre_back_ip",
  "dc_wide_centre_back_ip",
  "dc_overlapping_centre_back_ip",
  "dc_centre_back_oop",
  "dc_stopping_centre_back_oop",
  "dc_covering_centre_back_oop",
  "dc_wide_centre_back_oop",
  "dc_wide_stopping_centre_back_oop",
  "dc_wide_covering_centre_back_oop",
  "mc_central_midfielder_ip",
  "mc_attacking_midfielder_ip",
  "mc_advanced_playmaker_ip",
  "mc_wide_central_midfielder_ip",
  "mc_channel_midfielder_ip",
  "mc_midfield_playmaker_ip",
  "mc_central_midfielder_oop",
  "mc_pressing_central_midfielder_oop",
  "mc_screening_central_midfielder_oop",
  "mc_wide_covering_central_midfielder_oop",
  "dm_defensive_midfielder_ip",
  "dm_deep_lying_playmaker_ip",
  "dm_box_to_box_midfielder_ip",
  "dm_half_back_ip",
  "dm_box_to_box_playmaker_ip",
  "dm_defensive_midfielder_oop",
  "dm_dropping_defensive_midfielder_oop",
  "dm_pressing_defensive_midfielder_oop",
  "dm_screening_defensive_midfielder_oop",
  "dm_wide_covering_defensive_midfielder_oop",
  "dl_dr_full_back_ip",
  "dl_dr_wing_back_ip",
  "dl_dr_inside_wing_back_ip",
  "dl_dr_inside_full_back_ip",
  "dl_dr_playmaking_wing_back_ip",
  "dl_dr_full_back_oop",
  "dl_dr_pressing_full_back_oop",
  "dl_dr_holding_full_back_oop",
  "gk_traditional_goalkeeper_ip",
  "gk_ball_playing_goalkeeper_ip",
  "gk_no_nonsense_goalkeeper_ip",
  "gk_traditional_goalkeeper_oop",
  "gk_sweeper_keeper_oop",
  "gk_line_holding_keeper_oop",
  "st_deep_lying_forward_ip",
  "st_centre_forward_ip",
  "st_target_forward_ip",
  "st_poacher_ip",
  "st_channel_forward_ip",
  "st_false_nine_ip",
  "st_centre_forward_oop",
  "st_tracking_centre_forward_oop",
  "st_central_outlet_centre_forward_oop",
  "st_splitting_outlet_centre_forward_oop",
  "ml_mr_wide_midfielder_ip",
  "ml_mr_winger_ip",
  "ml_mr_playmaking_winger_ip",
  "ml_mr_inside_winger_ip",
  "ml_mr_wide_midfielder_oop",
  "ml_mr_tracking_wide_midfielder_oop",
  "ml_mr_wide_outlet_wide_midfielder_oop",
  "wbl_wbr_wing_back_ip",
  "wbl_wbr_advanced_wing_back_ip",
  "wbl_wbr_inside_wing_back_ip",
  "wbl_wbr_playmaking_wing_back_ip",
  "wbl_wbr_wing_back_oop",
  "wbl_wbr_pressing_wing_back_oop",
  "wbl_wbr_holding_wing_back_oop",
  "aml_amr_winger_ip",
  "aml_amr_inside_forward_ip",
  "aml_amr_playmaking_winger_ip",
  "aml_amr_wide_forward_ip",
  "aml_amr_inside_winger_ip",
  "aml_amr_winger_oop",
  "aml_amr_tracking_winger_oop",
  "aml_amr_inside_outlet_winger_oop",
  "aml_amr_wide_outlet_winger_oop",
] as const;

describe("Moneyball role catalog", () => {
  it("pins the versioned backend ID order and exact size", () => {
    expect(MONEYBALL_ROLE_CATALOG_VERSION).toBe(1);
    expect(MONEYBALL_ROLE_CATALOG).toHaveLength(88);
    expect(MONEYBALL_ROLE_CATALOG.map((role) => role.id)).toEqual(
      EXPECTED_ROLE_IDS,
    );
    expect(new Set(MONEYBALL_ROLE_CATALOG.map((role) => role.id)).size).toBe(
      88,
    );
  });

  it("matches every frontend metadata tuple to the backend catalog", () => {
    expect(
      MONEYBALL_ROLE_CATALOG.map(
        ({ id, label, phase, positionFamily, positionTags }) => ({
          id,
          label,
          phase,
          positionFamily,
          positionTags,
        }),
      ),
    ).toEqual(
      BACKEND_ROLE_CATALOG.map((role) => ({
        id: role.id,
        label: `${role.display_name} (${role.phase === "in_possession" ? "IP" : "OOP"} · ${orderedPositions(role.position_tags).join("/")})`,
        phase: role.phase,
        positionFamily: role.position_family,
        positionTags: role.position_tags,
      })),
    );
  });

  it("keeps phase, position-family, and position-tag identity distinct", () => {
    expect(
      MONEYBALL_ROLE_CATALOG.filter((role) => role.phase === "in_possession"),
    ).toHaveLength(49);
    expect(
      MONEYBALL_ROLE_CATALOG.filter(
        (role) => role.phase === "out_of_possession",
      ),
    ).toHaveLength(39);

    expect(
      MONEYBALL_ROLE_CATALOG.filter(
        (role) => role.positionFamily === "wing_back",
      ),
    ).toHaveLength(7);
    expect(
      MONEYBALL_ROLE_CATALOG.find((role) => role.id === "wbl_wbr_wing_back_ip"),
    ).toMatchObject({
      label: "Wing-Back (IP · WBR/WBL)",
      phase: "in_possession",
      positionFamily: "wing_back",
      positionTags: ["WBL", "WBR"],
    });
    expect(
      MONEYBALL_ROLE_CATALOG.find((role) => role.id === "dl_dr_wing_back_ip"),
    ).toMatchObject({ label: "Wing-Back (IP · DR/DL)" });
  });
});
