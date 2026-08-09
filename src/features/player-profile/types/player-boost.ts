export type PlayerBoostResult = {
  snapshotId: number;
  operation: string;
  previousCurrentAbility: number | null;
  currentAbility: number | null;
  potentialAbility: number | null;
  previousAmbition: number | null;
  ambition: number | null;
  previousProfessionalism: number | null;
  professionalism: number | null;
  previousDetermination: number | null;
  determination: number | null;
};
