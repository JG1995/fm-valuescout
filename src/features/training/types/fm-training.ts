/** What a save's individual training follows. */
export type TrainingBasis = "planner" | "development";

/** A slot position a player is not yet fully familiar with. */
export type FmTrainingLearning = {
  position: string;
  /** His familiarity there out of 20; 0 when he has none. */
  familiarity: number;
  /**
   * Whether it is the slot's out-of-possession position, which he trains in
   * place of the slot's own in-possession role and position.
   */
  outOfPossession: boolean;
};

/** Where the Planner has a player. */
export type FmTrainingSlot = {
  team: "senior" | "reserves" | "youth";
  teamName: string;
  stringName: string;
  laneId: string;
  /** The slot's placements in and out of possession, such as "AMR" and "MR". */
  ipPosition: string;
  oopPosition: string;
  /** The slot position he is not yet fully familiar with, whatever he trains. */
  learning: FmTrainingLearning | null;
};

/** The slot whose roles a player trains. */
export type FmTrainingTarget = {
  laneId: string;
  ipPosition: string;
  oopPosition: string;
  /**
   * The slot's own in-possession role, which he trains unless he is learning
   * the out-of-possession position.
   */
  inPossessionRole: string;
  /**
   * Whether it is his development slot and trains differently from his
   * Planner slot, if he has one.
   */
  development: boolean;
};

/** A slot a developing player could grow into. */
export type FmTrainingDevelopmentOption = {
  laneId: string;
  ipPosition: string;
  oopPosition: string;
  /**
   * The slot's in-possession role, which names the slot beside its position.
   * He trains its out-of-possession role too.
   */
  inPossessionRole: string;
  currentScore: number;
  potentialScore: number;
  /** Whether he can play the slot's positions today. */
  playable: boolean;
  /**
   * Where his potential stands among the slot's players in the save, as a top
   * percentage from 1 to 100; null for a slot the save's position rule does
   * not rank him in.
   */
  potentialTopPercent: number | null;
};

/** Where a player young enough to develop is growing. */
export type FmTrainingDevelopment = {
  /** His development slot; null when he suits no slot. */
  laneId: string | null;
  suggestedLaneId: string | null;
  /** Whether the user chose his development slot. */
  locked: boolean;
  /** The slots he is ranked in, best potential first, then the others. */
  options: FmTrainingDevelopmentOption[];
};

/**
 * The training Apply in FM sets for one player. The roles are those of
 * `target` and are null for a player whose roles FM keeps.
 */
export type FmTrainingPlayer = {
  playerUid: number;
  name: string;
  age: number | null;
  slot: FmTrainingSlot | null;
  target: FmTrainingTarget | null;
  /** The position his in-possession role is trained at, such as "DC". */
  position: string | null;
  inPossessionRole: string | null;
  outOfPossessionRole: string | null;
  /** The position of `target` he trains to become fully familiar with it. */
  learning: FmTrainingLearning | null;
  /** Null for a player too old to develop, or of unknown age. */
  development: FmTrainingDevelopment | null;
  /** His Suggested Training; null clears his focus in FM. */
  focus: string | null;
};

/** A slot of the tactic, as the plan names it. */
export type FmTrainingLane = {
  laneId: string;
  ipPosition: string;
  oopPosition: string;
  /** The first slot of the tactic that trains the same, which may be itself. */
  trainsAs: string;
};

export type FmTraining = {
  /** Players in a Planner slot by squad, string, and slot, then the rest by name. */
  players: FmTrainingPlayer[];
  /** Why the training cannot be set now; null when it can. */
  unavailableReason: string | null;
  basis: TrainingBasis;
  /** The oldest age at which a player still counts as developing. */
  developmentMaxAge: number;
  /**
   * Whether development slots are suggested only among the positions a player
   * can play, or in any position on his role scores and preferred foot alone.
   */
  developmentPositionFamiliarity: boolean;
  lanes: FmTrainingLane[];
};

export type FmTrainingResult = {
  /** Players whose training FM now holds as planned. */
  applied: number[];
  failed: { playerUid: number; name: string; reason: string }[];
  /** The error that stopped the request early; FM may have taken some changes first. */
  error: string | null;
};
