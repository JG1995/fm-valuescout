import { describe, expect, it } from "vitest";
import { PLAYER_METRICS } from "./player-metrics";

function roleMetric(roleId: string) {
  return PLAYER_METRICS.find((metric) => metric.id === `role.${roleId}`);
}

describe("PLAYER_METRICS", () => {
  it("keeps potential role metrics in their own grouped family", () => {
    expect(
      PLAYER_METRICS.find(
        (metric) => metric.id === "potential_role.goalkeeper_ip",
      ),
    ).toMatchObject({
      label: "Potential role · Goalkeeper (IP)",
      category: "potential-role-scores",
      roleFamily: "Goalkeepers",
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
  ])("groups %s by its catalog playing area", (roleId, roleFamily) => {
    expect(roleMetric(roleId)).toMatchObject({ roleFamily });
  });
});
