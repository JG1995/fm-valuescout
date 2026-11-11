import { useQuery } from "@tanstack/react-query";
import { CircleAlert, FileClock } from "lucide-react";
import { Button } from "@/components/ui/button/button";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import {
  formatDaysRemaining,
  formatMoney,
  formatShortDate,
} from "@/utils/format";
import { dashboardExpiringContractsQueryOptions } from "../api/dashboard-query-options";
import type {
  DashboardContext,
  DashboardExpiringContract,
} from "../types/dashboard";
import {
  DashboardTable,
  numericCellClass,
  PlayerCell,
  rowClass,
} from "./dashboard-table";

function Money({ value }: { value: number | null }) {
  return value === null ? (
    <span className="text-on-surface-variant">—</span>
  ) : (
    formatMoney(value)
  );
}

function ContractRow({ player }: { player: DashboardExpiringContract }) {
  return (
    <tr className={rowClass}>
      <PlayerCell
        uid={player.uid}
        name={player.name}
        detail={player.age === null ? "Age unknown" : `${player.age} years`}
      />
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
      <td className={numericCellClass}>{player.ca}</td>
      <td className={numericCellClass}>
        <Money value={player.weeklyWageGbp} />
      </td>
      <td className={numericCellClass}>
        <Money value={player.marketValueGbp} />
      </td>
    </tr>
  );
}

/** The club's own contracts that end soonest: the renew-or-sell list. */
export function ExpiringContractsPanel({
  context,
}: {
  context: DashboardContext;
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
        <EmptyState icon={FileClock} title="No contracts end soon">
          {contracts?.windowMonths == null
            ? "No contract end dates were read for your players."
            : `None of your players' contracts end within ${contracts.windowMonths} months of the in-game date.`}
        </EmptyState>
      ) : (
        <>
          <div className="max-h-[22.5rem] overflow-auto border-t border-outline-variant">
            <DashboardTable
              caption="Your players by contract end date, soonest first"
              headings={[
                { label: "Player" },
                { label: "Ends", width: "w-32" },
                { label: "CA", numeric: true, width: "w-14" },
                { label: "Wage p/w", numeric: true, width: "w-24" },
                { label: "Value", numeric: true, width: "w-20" },
              ]}
            >
              {contracts.players.map((player) => (
                <ContractRow key={player.uid} player={player} />
              ))}
            </DashboardTable>
          </div>
          {contracts.total > contracts.players.length ? (
            <p className="border-t border-outline-variant px-4 py-2 text-body-sm text-on-surface-variant">
              {contracts.total - contracts.players.length} more end later in
              this period.
            </p>
          ) : null}
        </>
      )}
    </Panel>
  );
}
