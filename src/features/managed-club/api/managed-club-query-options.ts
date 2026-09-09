import { queryOptions } from "@tanstack/react-query";
import { invokeCommand } from "@/lib/tauri-client";
import type {
  ManagedClubOption,
  ManagedClubStatus,
} from "../types/managed-club";
import { managedClubKeys } from "./managed-club-keys";

export const managedClubQueryOptions = queryOptions({
  queryKey: managedClubKeys.status(),
  queryFn: () => invokeCommand<ManagedClubStatus>("get_managed_club"),
});

export const managedClubOptionsQueryOptions = queryOptions({
  queryKey: managedClubKeys.options(),
  queryFn: () =>
    invokeCommand<ManagedClubOption[]>("list_managed_club_options"),
});
