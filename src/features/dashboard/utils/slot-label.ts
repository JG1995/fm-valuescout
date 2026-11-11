import { compareTacticPositions } from "@/utils/position-order";
import { ROLE_CATALOG } from "@/utils/role-catalog";
import type { DashboardLineupSlot } from "../types/dashboard";

type SlotPlacement = Pick<
  DashboardLineupSlot,
  "ipPosition" | "ipRoleId" | "oopPosition" | "oopRoleId"
>;

export type SlotLabel = {
  /** Placement identifier: one token, or the IP/OOP pair such as `AML/ML`. */
  compact: string;
  /** Role names without their phase suffix. */
  context: string;
  /** The whole definition, for accessible names and titles. */
  full: string;
};

const roleLabels = new Map<string, string>(
  ROLE_CATALOG.map((role) => [role.id, role.label]),
);

function roleLabel(roleId: string): string {
  return roleLabels.get(roleId) ?? roleId;
}

function roleName(roleId: string): string {
  return roleLabel(roleId).replace(/ \((IP|OOP)\)$/, "");
}

/** Names a tactic slot the way Search's tactic columns do. */
export function slotLabel(slot: SlotPlacement): SlotLabel {
  const ipRole = roleName(slot.ipRoleId);
  const oopRole = roleName(slot.oopRoleId);
  return {
    compact:
      slot.ipPosition === slot.oopPosition
        ? slot.ipPosition
        : `${slot.ipPosition}/${slot.oopPosition}`,
    context: ipRole === oopRole ? ipRole : `${ipRole} / ${oopRole}`,
    full: `${slot.ipPosition} (${roleLabel(slot.ipRoleId)}) / ${slot.oopPosition} (${roleLabel(slot.oopRoleId)})`,
  };
}

/**
 * The line under a slot's placement: its roles, or what its Moneyball score is
 * made of when that is not the roles.
 */
export function slotContext(
  slot: SlotPlacement & Pick<DashboardLineupSlot, "scoreBasis">,
): string {
  if (slot.scoreBasis === "moneyball_custom") return "Custom metrics";
  if (slot.scoreBasis === "unavailable") return "No Moneyball role";
  return slotLabel(slot).context;
}

/** Slots from the goalkeeper to the striker, by IP position and then OOP position. */
export function orderedSlots<T extends SlotPlacement>(
  slots: readonly T[],
): T[] {
  return [...slots].sort(
    (left, right) =>
      compareTacticPositions(left.ipPosition, right.ipPosition) ||
      compareTacticPositions(left.oopPosition, right.oopPosition),
  );
}
