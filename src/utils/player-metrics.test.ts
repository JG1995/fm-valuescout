import { describe, expect, it } from "vitest";
import { PLAYER_METRICS } from "./player-metrics";

function roleMetric(roleId: string) {
  return PLAYER_METRICS.find((metric) => metric.id === `role.${roleId}`);
}

describe("PLAYER_METRICS", () => {
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
