import { queryOptions } from "@tanstack/react-query";
import { invokeCommand } from "@/lib/tauri-client";
import type { MoneyballLeagueAdjustment } from "../types/moneyball-profile";
import { moneyballKeys } from "./moneyball-keys";

export const leagueAdjustmentQueryOptions = queryOptions({
  queryKey: moneyballKeys.leagueAdjustment(),
  queryFn: () =>
    invokeCommand<MoneyballLeagueAdjustment>("get_moneyball_league_adjustment"),
});

export function setMoneyballLeagueAdjustment(enabled: boolean) {
  return invokeCommand<MoneyballLeagueAdjustment>(
    "set_moneyball_league_adjustment",
    { enabled },
  );
}
