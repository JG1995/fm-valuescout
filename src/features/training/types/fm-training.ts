/** Where the Planner has a player. */
export type FmTrainingSlot = {
  team: "senior" | "reserves" | "youth";
  teamName: string;
  stringName: string;
  /** The slot's placements in and out of possession, such as "AMR" and "MR". */
  ipPosition: string;
  oopPosition: string;
  /**
   * The slot's own in-possession role, which he trains unless he is learning
   * the out-of-possession position.
   */
  inPossessionRole: string;
};

/**
 * The training Apply in FM sets for one player. Roles come from his Planner
 * slot and are null for a player in no slot, whose roles FM keeps.
 */
export type FmTrainingPlayer = {
  playerUid: number;
  name: string;
  age: number | null;
  slot: FmTrainingSlot | null;
  /** The position his in-possession role is trained at, such as "DC". */
  position: string | null;
  inPossessionRole: string | null;
  outOfPossessionRole: string | null;
  /**
   * The slot position he trains to become fully familiar with it, his
   * familiarity there out of 20 (0 when he has none), and whether it is the
   * slot's out-of-possession position, which he trains in place of the slot's
   * own in-possession role and position.
   */
  learning: {
    position: string;
    familiarity: number;
    outOfPossession: boolean;
  } | null;
  /** His Suggested Training; null clears his focus in FM. */
  focus: string | null;
};

export type FmTraining = {
  /** Players in a Planner slot by squad, string, and slot, then the rest by name. */
  players: FmTrainingPlayer[];
  /** Why the training cannot be set now; null when it can. */
  unavailableReason: string | null;
};

export type FmTrainingResult = {
  /** Players whose training FM now holds as planned. */
  applied: number[];
  failed: { playerUid: number; name: string; reason: string }[];
  /** The error that stopped the request early; FM may have taken some changes first. */
  error: string | null;
};
