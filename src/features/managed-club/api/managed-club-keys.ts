export const managedClubKeys = {
  all: ["managed-club"] as const,
  status: () => [...managedClubKeys.all, "status"] as const,
  options: () => [...managedClubKeys.all, "options"] as const,
};
