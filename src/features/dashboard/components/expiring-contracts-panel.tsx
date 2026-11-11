import { useQuery } from "@tanstack/react-query";
import { CircleAlert, FileClock } from "lucide-react";
import { Button } from "@/components/ui/button/button";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { cn } from "@/utils/cn";
import {
  formatDaysRemaining,
  formatMoney,
  formatShortDate,
} from "@/utils/format";
import { dashboardExpiringContractsQueryOptions } from "../api/dashboard-query-options";
import type {
  DashboardContext,
  DashboardExpiringContract,
  DashboardGraphics,
} from "../types/dashboard";
import {
  dashboardArea,
  dashboardPanelClass,
  dashboardPanelContentClass,
} from "./dashboard-grid";
import {
  DashboardTable,
  numericCellClass,
  PlayerIdentity,
  rowClass,
} from "./dashboard-table";

function playerDetail(player: DashboardExpiringContract) {
  return [
    player.age === null ? null : `${player.age} years`,
    player.marketValueGbp === null ? null : formatMoney(player.marketValueGbp),
  ]
    .filter((part) => part !== null)
    .join(" · ");
}

function ContractRow({
  player,
  graphics,
}: {
  player: DashboardExpiringContract;
  graphics: DashboardGraphics;
}) {
  return (
    <tr className={rowClass}>
      <th scope="row" className="min-w-0 px-2 text-left font-normal">
        <PlayerIdentity
          uid={player.uid}
          name={player.name}
          detail={playerDetail(player)}
          portrait={graphics.portrait(player.uid, player.name)}
        />
      </th>
      <td className="min-w-0 px-2">
        <span className="block truncate text-body-sm text-on-surface">
          {formatShortDate(player.expiryDate)}
        </span>
        {player.daysRemaining === null ? null : (
          <span className="block truncate text-[11px] leading-4 text-on-surface-variant">
            {formatDaysRemaining(player.daysRemaining)}
          </span>
        )}
      </td>
      <td className={numericCellClass}>
        {player.weeklyWageGbp === null ? (
          <span className="text-on-surface-variant">—</span>
        ) : (
          formatMoney(player.weeklyWageGbp)
        )}
      </td>
    </tr>
  );
}

/** The club's own contracts that end soonest: the renew-or-sell list. */
export function ExpiringContractsPanel({
  context,
  graphics,
}: {
  context: DashboardContext;
  graphics: DashboardGraphics;
}) {
  const query = useQuery(dashboardExpiringContractsQueryOptions(context));
  const contracts = query.data ?? null;
  const window =
    contracts?.windowMonths == null
      ? "Soonest first"
      : `Next ${contracts.windowMonths} months`;
  return (
    <Panel
      title="Expiring contracts"
      flush
      actions={
        contracts ? (
          <p className="text-body-sm text-on-surface-variant">{window}</p>
        ) : null
      }
      className={cn(dashboardArea.contracts, dashboardPanelClass)}
      contentClassName={dashboardPanelContentClass}
    >
      {query.isError ? (
        <EmptyState
          icon={CircleAlert}
          title="Could not load expiring contracts"
          action={
            <Button
              variant="secondary"
              className="mt-2"
              onClick={() => void query.refetch()}
            >
              Retry
            </Button>
          }
        >
          {query.error.message}
        </EmptyState>
      ) : query.isPending ? (
        <p className="p-4 text-body-md text-on-surface-variant" role="status">
          Loading expiring contracts…
        </p>
      ) : contracts === null || contracts.players.length === 0 ? (
        <div className="flex flex-1 items-center justify-center overflow-y-auto">
          <EmptyState icon={FileClock} title="No contracts end soon">
            {contracts?.windowMonths == null
              ? "No contract end dates were read for your players."
              : `None of your players' contracts end within ${contracts.windowMonths} months of the in-game date.`}
          </EmptyState>
        </div>
      ) : (
        <>
          {/* The one list that can outgrow its panel: it scrolls in place, out of
              flow so its length never stretches the grid. */}
          <div className="relative min-h-40 flex-1 border-t border-outline-variant">
            <div className="absolute inset-0 overflow-auto">
              <DashboardTable
                caption="Your players by contract end date, soonest first"
                headings={[
                  { label: "Player" },
                  { label: "Ends", width: "w-24" },
                  { label: "Wage p/w", numeric: true, width: "w-20" },
                ]}
              >
                {contracts.players.map((player) => (
                  <ContractRow
                    key={player.uid}
                    player={player}
                    graphics={graphics}
                  />
                ))}
              </DashboardTable>
            </div>
          </div>
          {contracts.total > contracts.players.length ? (
            <p className="shrink-0 border-t border-outline-variant px-4 py-2 text-body-sm text-on-surface-variant">
              {contracts.total - contracts.players.length} more end later in
              this period.
            </p>
          ) : null}
        </>
      )}
    </Panel>
  );
}
