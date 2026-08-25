export const playerResultContextMutationKey = [
  "player-result-context",
] as const;

export type ClearPlayerResultContext = () => Promise<void>;
