import { describe, expect, it } from "vitest";
import { getPlayerMetric, PLAYER_METRICS } from "./player-metrics";

describe("player metric table metadata", () => {
  it("gives every selectable metric a fixed table width and alignment", () => {
    expect(getPlayerMetric("name")).toMatchObject({
      align: "left",
      defaultWidth: 224,
      sortable: true,
    });
    expect(getPlayerMetric("attr.Acceleration")).toMatchObject({
      align: "right",
      defaultWidth: 88,
      sortable: true,
    });
    expect(getPlayerMetric("potential_role.goalkeeper_ip")).toMatchObject({
      align: "right",
      defaultWidth: 112,
      sortable: true,
    });
  });

  it.each([
    ["goalkeeper_ip", "Goalkeepers"],
    ["centre_back_ip", "Central defense"],
    ["wing_back_ip", "Full-back and wing-back"],
    ["box_to_box_midfielder_ip", "Defensive midfield"],
    ["central_midfielder_ip", "Central midfield"],
    ["inside_winger_ip", "Wide midfield and wings"],
    ["channel_midfielder_ip", "Attacking midfield"],
    ["wide_forward_ip", "Forwards"],
  ])("keeps %s in its catalog playing area", (roleId, roleFamily) => {
    expect(
      PLAYER_METRICS.find((metric) => metric.id === `role.${roleId}`),
    ).toMatchObject({
      category: "current-role-scores",
      roleFamily,
    });
    expect(
      PLAYER_METRICS.find((metric) => metric.id === `potential_role.${roleId}`),
    ).toMatchObject({
      category: "potential-role-scores",
      roleFamily,
    });
  });
});
