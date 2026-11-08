import type { TableGroupInput } from "@/components/player-table/table-groups";
import { getMoneyballSearchMetric } from "@/utils/moneyball-search-metrics";
import { getPlayerMetric } from "@/utils/player-metrics";
import { isValidTacticColumnId } from "@/utils/tactic-ids";
import type { SearchView } from "../types/search-view";

/**
 * Header groups for each Search view. The header, Columns, field pickers, and
 * column insertion all use these, so a field sits in the same place everywhere.
 */
export const SEARCH_TABLE_GROUPS: TableGroupInput = {
  groups: [
    { id: "profile", label: "Profile" },
    { id: "ability", label: "Ability" },
    { id: "market", label: "Market" },
    { id: "tactic-fit", label: "Tactic Fit" },
  ],
  groupForColumn: (columnId) => {
    if (
      isValidTacticColumnId(columnId) ||
      columnId.startsWith("role.") ||
      columnId.startsWith("potential_role.") ||
      columnId === "club_dna"
    ) {
      return "tactic-fit";
    }
    switch (getPlayerMetric(columnId)?.category) {
      case "identity":
        return "profile";
      case "ability-reputation":
      case "visible-attributes":
      case "hidden-attributes":
      case "personality":
        return "ability";
      case "club-contract":
        return "market";
      case "position-suitability":
      case "current-role-scores":
      case "potential-role-scores":
        return "tactic-fit";
      default:
        return "other";
    }
  },
};

export const MONEYBALL_TABLE_GROUPS: TableGroupInput = {
  groups: [
    { id: "profile", label: "Profile" },
    { id: "market", label: "Market" },
    { id: "playing-time", label: "Playing Time" },
    { id: "performance", label: "Performance" },
    { id: "role-fit", label: "Role Fit" },
  ],
  groupForColumn: (columnId) => {
    if (columnId === "position") {
      return "profile";
    }
    if (columnId.startsWith("moneyball_role.")) {
      return "role-fit";
    }
    const metric = getMoneyballSearchMetric(columnId);
    if (!metric) {
      return "other";
    }
    if (metric.context) {
      return "playing-time";
    }
    if (metric.metric) {
      return "performance";
    }
    if (metric.role) {
      return "role-fit";
    }
    switch (metric.category) {
      case "Identity":
        return "profile";
      case "Club and value":
        return "market";
      case "Playing time":
        return "playing-time";
      default:
        return "other";
    }
  },
};

export function searchTableGroups(view: SearchView): TableGroupInput {
  return view === "moneyball" ? MONEYBALL_TABLE_GROUPS : SEARCH_TABLE_GROUPS;
}
