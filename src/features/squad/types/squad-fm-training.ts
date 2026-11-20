/**
 * The training Apply in FM sets for one player. Roles come from his Planner
 * slot and are null for a player in no slot, whose roles FM keeps.
 */
export type SquadFmTrainingPlayer = {
  playerUid: number;
  name: string;
  /** The slot's in-possession position, such as "DC". */
  position: string | null;
  inPossessionRole: string | null;
  outOfPossessionRole: string | null;
  /** His Suggested Training; null clears his focus in FM. */
  focus: string | null;
};

export type SquadFmTraining = {
  players: SquadFmTrainingPlayer[];
  /** Why the training cannot be set now; null when it can. */
  unavailableReason: string | null;
};

export type SquadFmTrainingResult = {
  /** Players whose training FM now holds as planned. */
  applied: number[];
  failed: { playerUid: number; name: string; reason: string }[];
  /** The error that stopped the request early; FM may have taken some changes first. */
  error: string | null;
};
