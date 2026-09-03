import { describe, expect, it } from "vitest";
import { ROLE_CATALOG } from "@/utils/role-catalog";
import { createFilterRuleId } from "../types/filter-rule";
import { dynamicColumnFields, isVisibleSortField } from "./dynamic-columns";

describe("dynamicColumnFields", () => {
  it("returns unique non-basic complete filter fields", () => {
    const fields = dynamicColumnFields([
      {
        id: createFilterRuleId(),
        field: "ca",
        op: "gt",
        value: { type: "integer", value: 100 },
      },
      {
        id: createFilterRuleId(),
        field: "role.deep_lying_playmaker_ip",
        op: "gt",
        value: { type: "integer", value: 70 },
      },
      {
        id: createFilterRuleId(),
        field: "attr.Acceleration",
        op: "gt",
        value: { type: "integer", value: 12 },
      },
      {
        id: createFilterRuleId(),
        field: "role.deep_lying_playmaker_ip",
        op: "lt",
        value: { type: "integer", value: 95 },
      },
    ]);

    expect(fields).toEqual([
      "role.deep_lying_playmaker_ip",
      "attr.Acceleration",
    ]);
  });

  it("requests Club DNA filters and accepts the fixed metric for sorting", () => {
    const fields = dynamicColumnFields([
      {
        id: createFilterRuleId(),
        field: "club_dna",
        op: "gt",
        value: { type: "integer", value: 70 },
      },
    ]);

    expect(fields).toEqual(["club_dna"]);
    expect(isVisibleSortField("club_dna", [])).toBe(true);
  });

  it("allows sorting by known metrics whether or not they are requested", () => {
    const filters = [
      {
        id: createFilterRuleId(),
        field: "role.deep_lying_playmaker_ip",
        op: "gt",
        value: { type: "integer" as const, value: 70 },
      },
    ];
    expect(isVisibleSortField("role.deep_lying_playmaker_ip", filters)).toBe(
      true,
    );
    expect(isVisibleSortField("attr.Acceleration", filters)).toBe(true);
    expect(isVisibleSortField("ca", filters)).toBe(true);
  });

  it("includes Position and position-suitability display fields", () => {
    expect(
      dynamicColumnFields([
        {
          id: createFilterRuleId(),
          field: "position",
          op: "is",
          value: { type: "text", value: "MC" },
        },
        {
          id: createFilterRuleId(),
          field: "pos.MC",
          op: "gt",
          value: { type: "integer", value: 15 },
        },
      ]),
    ).toEqual(["position", "pos.MC"]);
  });

  it("includes potential role filters in requested table fields", () => {
    const filters = [
      {
        id: createFilterRuleId(),
        field: "potential_role.goalkeeper_ip",
        op: "gt",
        value: { type: "integer" as const, value: 70 },
      },
    ];

    expect(dynamicColumnFields(filters)).toEqual([
      "potential_role.goalkeeper_ip",
    ]);
    expect(isVisibleSortField("potential_role.goalkeeper_ip", filters)).toBe(
      true,
    );
  });

  it("allows a known metric to sort even while its column is not requested", () => {
    expect(isVisibleSortField("position", [])).toBe(true);
    expect(isVisibleSortField("attr.Acceleration", [])).toBe(true);
    expect(isVisibleSortField("unknown.metric", [])).toBe(false);
  });

  it("allows only canonical tactic sorts present in the current table layout", () => {
    const tacticSort = "tactic_current.goalkeeper";

    expect(isVisibleSortField(tacticSort, [], "general", [tacticSort])).toBe(
      true,
    );
    expect(isVisibleSortField(tacticSort, [], "general", [])).toBe(false);
    expect(
      isVisibleSortField("tactic_current.unknown", [], "general", [
        "tactic_current.unknown",
      ]),
    ).toBe(false);
  });

  it("requests and sorts Moneyball role fields in the Moneyball view only", () => {
    const filters = [
      {
        id: createFilterRuleId(),
        field: "moneyball_role.wbl_wbr_wing_back_ip",
        op: "gt",
        value: { type: "integer" as const, value: 70 },
      },
    ];

    expect(dynamicColumnFields(filters, "moneyball")).toEqual([
      "moneyball_role.wbl_wbr_wing_back_ip",
    ]);
    expect(
      isVisibleSortField(
        "moneyball_role.wbl_wbr_wing_back_ip",
        [],
        "moneyball",
      ),
    ).toBe(true);
    expect(
      isVisibleSortField("moneyball_role.wbl_wbr_wing_back_ip", [], "general"),
    ).toBe(false);
  });

  it("treats Shortlist as General-family for visible sorts", () => {
    expect(isVisibleSortField("ca", [], "shortlist")).toBe(true);
    expect(isVisibleSortField("pa", [], "shortlist")).toBe(true);
    expect(
      isVisibleSortField("role.deep_lying_playmaker_ip", [], "shortlist"),
    ).toBe(true);
    expect(
      isVisibleSortField("potential_role.goalkeeper_ip", [], "shortlist"),
    ).toBe(true);
    expect(isVisibleSortField("attr.Acceleration", [], "shortlist")).toBe(true);
    expect(isVisibleSortField("club_dna", [], "shortlist")).toBe(true);
    expect(isVisibleSortField("moneyball.goals", [], "shortlist")).toBe(false);
    expect(
      isVisibleSortField(
        "moneyball_role.wbl_wbr_wing_back_ip",
        [],
        "shortlist",
      ),
    ).toBe(false);
  });

  it("keeps Moneyball basic identity and value sorts while rejecting CA and PA", () => {
    expect(isVisibleSortField("name", [], "moneyball")).toBe(true);
    expect(isVisibleSortField("age", [], "moneyball")).toBe(true);
    expect(isVisibleSortField("nationality", [], "moneyball")).toBe(true);
    expect(isVisibleSortField("club", [], "moneyball")).toBe(true);
    expect(isVisibleSortField("division", [], "moneyball")).toBe(true);
    expect(isVisibleSortField("value", [], "moneyball")).toBe(true);
    expect(isVisibleSortField("ca", [], "moneyball")).toBe(false);
    expect(isVisibleSortField("pa", [], "moneyball")).toBe(false);
    expect(
      isVisibleSortField("moneyball.average_rating", [], "moneyball"),
    ).toBe(true);
  });

  it("mirrors General dynamic columns for Shortlist and rejects Moneyball fields", () => {
    const generalFields = [
      {
        id: createFilterRuleId(),
        field: "role.deep_lying_playmaker_ip",
        op: "gt",
        value: { type: "integer" as const, value: 70 },
      },
      {
        id: createFilterRuleId(),
        field: "potential_role.goalkeeper_ip",
        op: "gt",
        value: { type: "integer" as const, value: 70 },
      },
      {
        id: createFilterRuleId(),
        field: "attr.Acceleration",
        op: "gt",
        value: { type: "integer" as const, value: 12 },
      },
      {
        id: createFilterRuleId(),
        field: "club_dna",
        op: "gt",
        value: { type: "integer" as const, value: 70 },
      },
    ];

    expect(dynamicColumnFields(generalFields, "shortlist")).toEqual(
      dynamicColumnFields(generalFields, "general"),
    );
    expect(dynamicColumnFields(generalFields, "shortlist")).toEqual([
      "role.deep_lying_playmaker_ip",
      "potential_role.goalkeeper_ip",
      "attr.Acceleration",
      "club_dna",
    ]);

    const moneyballFields = [
      {
        id: createFilterRuleId(),
        field: "moneyball.goals",
        op: "gt",
        value: { type: "integer" as const, value: 5 },
      },
      {
        id: createFilterRuleId(),
        field: "moneyball_role.wbl_wbr_wing_back_ip",
        op: "gt",
        value: { type: "integer" as const, value: 70 },
      },
    ];
    expect(dynamicColumnFields(moneyballFields, "shortlist")).toEqual([]);
    expect(dynamicColumnFields(moneyballFields, "general")).toEqual([]);
  });
});

describe("ROLE_CATALOG", () => {
  it("mirrors the Rust catalog order and labels exactly", () => {
    // Keep in exact sync with scoring::catalog::all_roles (79 roles):
    // any reorder, rename, removal, or truncation fails here.
    expect(ROLE_CATALOG.map((role) => [role.id, role.label])).toEqual([
      ["goalkeeper_ip", "Goalkeeper (IP)"],
      ["ball_playing_goalkeeper_ip", "Ball-Playing Goalkeeper (IP)"],
      ["no_nonsense_goalkeeper_ip", "No-Nonsense Goalkeeper (IP)"],
      ["line_holding_keeper_oop", "Line-Holding Keeper (OOP)"],
      ["sweeper_keeper_oop", "Sweeper Keeper (OOP)"],
      ["centre_back_ip", "Centre-Back (IP)"],
      ["ball_playing_centre_back_ip", "Ball-Playing Centre-Back (IP)"],
      ["no_nonsense_centre_back_ip", "No-Nonsense Centre-Back (IP)"],
      ["wide_centre_back_ip", "Wide Centre-Back (IP)"],
      ["advanced_centre_back_ip", "Advanced Centre-Back (IP)"],
      ["overlapping_centre_back_ip", "Overlapping Centre-Back (IP)"],
      ["covering_centre_back_oop", "Covering Centre-Back (OOP)"],
      ["stopping_centre_back_oop", "Stopping Centre-Back (OOP)"],
      ["covering_wide_centre_back_oop", "Covering Wide Centre-Back (OOP)"],
      ["stopping_wide_centre_back_oop", "Stopping Wide Centre-Back (OOP)"],
      ["full_back_ip", "Full-Back (IP)"],
      ["inside_full_back_ip", "Inside Full-Back (IP)"],
      ["holding_full_back_oop", "Holding Full-Back (OOP)"],
      ["pressing_full_back_oop", "Pressing Full-Back (OOP)"],
      ["inside_wing_back_ip", "Inside Wing-Back (IP)"],
      ["playmaking_wing_back_ip", "Playmaking Wing-Back (IP)"],
      ["wing_back_ip", "Wing-Back (IP)"],
      ["advanced_wing_back_ip", "Advanced Wing-Back (IP)"],
      ["holding_wing_back_oop", "Holding Wing-Back (OOP)"],
      ["pressing_wing_back_oop", "Pressing Wing-Back (OOP)"],
      ["defensive_midfielder_ip", "Defensive Midfielder (IP)"],
      ["box_to_box_midfielder_ip", "Box-to-Box Midfielder (IP)"],
      ["box_to_box_playmaker_ip", "Box-to-Box Playmaker (IP)"],
      ["deep_lying_playmaker_ip", "Deep-Lying Playmaker (IP)"],
      ["half_back_ip", "Half-Back (IP)"],
      [
        "dropping_defensive_midfielder_oop",
        "Dropping Defensive Midfielder (OOP)",
      ],
      [
        "pressing_defensive_midfielder_oop",
        "Pressing Defensive Midfielder (OOP)",
      ],
      [
        "screening_defensive_midfielder_oop",
        "Screening Defensive Midfielder (OOP)",
      ],
      [
        "wide_covering_defensive_midfielder_oop",
        "Wide Covering Defensive Midfielder (OOP)",
      ],
      ["central_midfielder_ip", "Central Midfielder (IP)"],
      ["advanced_playmaker_ip", "Advanced Playmaker (IP)"],
      ["midfield_playmaker_ip", "Midfield Playmaker (IP)"],
      ["wide_central_midfielder_ip", "Wide Central Midfielder (IP)"],
      ["pressing_central_midfielder_oop", "Pressing Central Midfielder (OOP)"],
      [
        "screening_central_midfielder_oop",
        "Screening Central Midfielder (OOP)",
      ],
      [
        "wide_covering_central_midfielder_oop",
        "Wide Covering Central Midfielder (OOP)",
      ],
      ["wide_midfielder_ip", "Wide Midfielder (IP)"],
      ["tracking_wide_midfielder_oop", "Tracking Wide Midfielder (OOP)"],
      ["wide_outlet_wide_midfielder_oop", "Wide Outlet Wide Midfielder (OOP)"],
      ["inside_winger_ip", "Inside Winger (IP)"],
      ["playmaking_winger_ip", "Playmaking Winger (IP)"],
      ["winger_ip", "Winger (IP)"],
      ["attacking_midfielder_ip", "Attacking Midfielder (IP)"],
      ["channel_midfielder_ip", "Channel Midfielder (IP)"],
      ["free_role_ip", "Free Role (IP)"],
      ["second_striker_ip", "Second Striker (IP)"],
      [
        "central_outlet_attacking_midfielder_oop",
        "Central Outlet Attacking Midfielder (OOP)",
      ],
      [
        "splitting_outlet_attacking_midfielder_oop",
        "Splitting Outlet Attacking Midfielder (OOP)",
      ],
      [
        "tracking_attacking_midfielder_oop",
        "Tracking Attacking Midfielder (OOP)",
      ],
      ["wide_forward_ip", "Wide Forward (IP)"],
      ["inside_forward_ip", "Inside Forward (IP)"],
      ["inside_outlet_winger_oop", "Inside Outlet Winger (OOP)"],
      ["tracking_winger_oop", "Tracking Winger (OOP)"],
      ["wide_outlet_winger_oop", "Wide Outlet Winger (OOP)"],
      ["centre_forward_ip", "Centre Forward (IP)"],
      ["channel_forward_ip", "Channel Forward (IP)"],
      ["deep_lying_forward_ip", "Deep-Lying Forward (IP)"],
      ["false_nine_ip", "False Nine (IP)"],
      ["poacher_ip", "Poacher (IP)"],
      ["target_forward_ip", "Target Forward (IP)"],
      [
        "central_outlet_centre_forward_oop",
        "Central Outlet Centre Forward (OOP)",
      ],
      [
        "splitting_outlet_centre_forward_oop",
        "Splitting Outlet Centre Forward (OOP)",
      ],
      ["tracking_centre_forward_oop", "Tracking Centre Forward (OOP)"],
      ["goalkeeper_oop", "Goalkeeper (OOP)"],
      ["centre_back_oop", "Centre-Back (OOP)"],
      ["wide_centre_back_oop", "Wide Centre-Back (OOP)"],
      ["full_back_oop", "Full-Back (OOP)"],
      ["wing_back_oop", "Wing-Back (OOP)"],
      ["defensive_midfielder_oop", "Defensive Midfielder (OOP)"],
      ["central_midfielder_oop", "Central Midfielder (OOP)"],
      ["wide_midfielder_oop", "Wide Midfielder (OOP)"],
      ["attacking_midfielder_oop", "Attacking Midfielder (OOP)"],
      ["winger_oop", "Winger (OOP)"],
      ["centre_forward_oop", "Centre Forward (OOP)"],
    ]);
  });
});
