export const moneyballKeys = {
  all: ["moneyball"] as const,
  profile: (uid: number) => [...moneyballKeys.all, "profile", uid] as const,
};
