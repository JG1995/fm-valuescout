export const searchKeys = {
  all: ["search"] as const,
  players: (offset: number, limit: number) =>
    [...searchKeys.all, "players", { offset, limit }] as const,
};
