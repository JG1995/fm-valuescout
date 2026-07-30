import { useSuspenseQuery } from "@tanstack/react-query";
import { DatabaseZap, TriangleAlert } from "lucide-react";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import {
  formatAbsoluteUtc,
  formatCount,
  formatMissable,
  formatRelativeAge,
} from "@/utils/format";
import { currentSnapshotQueryOptions } from "../api/current-snapshot-query-options";
import { sanityPlayersQueryOptions } from "../api/sanity-players-query-options";

export function SnapshotOverviewPanel() {
  const { data: snapshot } = useSuspenseQuery(currentSnapshotQueryOptions);
  const { data: players } = useSuspenseQuery(sanityPlayersQueryOptions);

  if (!snapshot) {
    return (
      <Panel title="Snapshot" flush>
        <EmptyState icon={DatabaseZap} title="No data loaded for this save">
          No snapshot loaded for the active save. Use Load Data to scan Football
          Manager and ingest players into the database.
        </EmptyState>
      </Panel>
    );
  }

  return (
    <Panel title="Snapshot" flush>
      <p className="px-4 text-body-md text-on-surface-variant">
        <span className="text-on-surface">In database:</span>{" "}
        {formatCount(snapshot.playerCount)} players ·{" "}
        {/* An unknown in-game date drops out rather than leaving a dangling dash. */}
        {snapshot.gameDate ? `${snapshot.gameDate} · ` : null}game{" "}
        {snapshot.gameVersion} · loaded{" "}
        <span title={formatAbsoluteUtc(snapshot.loadedAtUtc)}>
          {formatRelativeAge(snapshot.loadedAtUtc)}
        </span>
      </p>

      {snapshot.scanTruncated && (
        <p className="mx-4 mt-3 flex items-start gap-2 rounded-md border border-warning/40 bg-warning-container px-3 py-2 text-body-sm text-on-warning-container">
          <TriangleAlert
            aria-hidden="true"
            size={16}
            strokeWidth={1.5}
            className="mt-0.5 shrink-0"
          />
          Incomplete snapshot: scan was capped at{" "}
          {formatMissable(
            snapshot.maxAccepted === null
              ? null
              : formatCount(snapshot.maxAccepted),
          )}{" "}
          players. Review results with care.
        </p>
      )}

      <div className="mt-4 overflow-x-auto">
        <table className="w-full border-collapse text-left">
          <caption className="sr-only">Player sanity list</caption>
          <thead>
            <tr className="bg-surface-container-lowest">
              <th
                scope="col"
                className="h-table-header-height px-2 text-label-md text-on-surface-variant uppercase"
              >
                Name
              </th>
              <th
                scope="col"
                className="h-table-header-height px-2 text-right text-label-md text-on-surface-variant uppercase"
              >
                CA
              </th>
              <th
                scope="col"
                className="h-table-header-height px-2 text-right text-label-md text-on-surface-variant uppercase"
                title="Deep-Lying Playmaker (in possession) role score"
              >
                DLP IP
              </th>
              <th
                scope="col"
                className="h-table-header-height px-2 text-label-md text-on-surface-variant uppercase"
              >
                Club
              </th>
            </tr>
          </thead>
          <tbody>
            {players.length === 0 ? (
              <tr>
                <td colSpan={4}>
                  <EmptyState icon={DatabaseZap} title="No players in snapshot">
                    The snapshot exists but holds no player rows. Run Load Data
                    again with Football Manager in an active save.
                  </EmptyState>
                </td>
              </tr>
            ) : (
              players.map((player) => (
                <tr
                  key={`${player.name}-${player.ca}`}
                  className="border-t border-outline-variant transition-colors duration-150 ease-out hover:bg-surface-container-high"
                >
                  <td className="h-table-row-height px-2 text-body-sm text-on-surface">
                    {player.name}
                  </td>
                  <td className="h-table-row-height px-2 text-right font-mono text-mono-sm text-on-surface tabular-nums">
                    {player.ca}
                  </td>
                  <td className="h-table-row-height px-2 text-right font-mono text-mono-sm text-on-surface tabular-nums">
                    {formatMissable(player.proofRoleScore)}
                  </td>
                  <td className="h-table-row-height px-2 text-body-sm text-on-surface-variant">
                    {formatMissable(player.club)}
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </Panel>
  );
}
