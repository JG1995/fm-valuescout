import type { TableGroupInput } from "@/components/player-table/table-groups";
import { getMoneyballSearchMetric } from "@/utils/moneyball-search-metrics";
import { getPlayerMetric } from "@/utils/player-metrics";
import { isValidTacticColumnId } from "@/utils/tactic-ids";

/**
 * Player Search's header groups. The header, Columns, the field pickers, and
 * column insertion all use these, so a field sits in the same place
 * everywhere. A tactic slot's recruitment filters use them too; groups with no
 * offered field are left out.
 */
export const SEARCH_TABLE_GROUPS: TableGroupInput = {
  groups: [
    { id: "profile", label: "Profile" },
    { id: "ability", label: "Ability" },
    { id: "market", label: "Market" },
    { id: "role-fit", label: "Role Scores" },
    { id: "playing-time", label: "Playing Time" },
    { id: "performance", label: "Performance" },
    { id: "moneyball-scores", label: "Moneyball Scores" },
    { id: "tactic", label: "Tactic" },
  ],
  groupForColumn: (columnId) => {
    if (isValidTacticColumnId(columnId)) {
      return "tactic";
    }
    const moneyball = getMoneyballSearchMetric(columnId);
    if (moneyball) {
      if (moneyball.context) {
        return "playing-time";
      }
      return moneyball.role ? "moneyball-scores" : "performance";
    }
    switch (getPlayerMetric(columnId)?.category) {
      case "identity":
        return "profile";
      case "ability-reputation":
        return columnId === "club_dna" ? "role-fit" : "ability";
      case "visible-attributes":
      case "hidden-attributes":
      case "personality":
        return "ability";
      case "club-contract":
        return "market";
      case "position-suitability":
      case "current-role-scores":
      case "potential-role-scores":
        return "role-fit";
      default:
        return "other";
    }
  },
};
