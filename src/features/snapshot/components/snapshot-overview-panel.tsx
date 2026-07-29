import { useSuspenseQuery } from "@tanstack/react-query";
import { currentSnapshotQueryOptions } from "../api/current-snapshot-query-options";
import { sanityPlayersQueryOptions } from "../api/sanity-players-query-options";
import type { SnapshotSummary } from "../types/snapshot";

function formatSnapshotMetadata(snapshot: SnapshotSummary) {
  const gameDate = snapshot.gameDate ? ` · ${snapshot.gameDate}` : "";
  return `${snapshot.playerCount} players loaded${gameDate} · game ${snapshot.gameVersion}`;
}

export function SnapshotOverviewPanel() {
  const { data: snapshot } = useSuspenseQuery(currentSnapshotQueryOptions);
  const { data: players } = useSuspenseQuery(sanityPlayersQueryOptions);

  if (!snapshot) {
    return (
      <div className="space-y-3 rounded-md border border-on-background/20 p-4">
        <h2 className="text-lg font-medium text-on-background">Snapshot</h2>
        <p className="text-on-background/80">
          No snapshot loaded for the active save. Use Load Data to scan and
          ingest players into the database.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-3 rounded-md border border-on-background/20 p-4">
      <h2 className="text-lg font-medium text-on-background">Snapshot</h2>
      <p className="text-on-background/80">
        <strong className="text-on-background">In database:</strong>{" "}
        {formatSnapshotMetadata(snapshot)}
      </p>
      <p className="text-sm text-on-background/60">
        Loaded at {snapshot.loadedAtUtc}
      </p>
      {snapshot.scanTruncated && (
        <p className="rounded-md border border-warning/40 bg-warning-container px-3 py-2 text-on-warning-container">
          Incomplete snapshot: scan was capped at{" "}
          {snapshot.maxAccepted ?? "unknown"} players. Review results with care.
        </p>
      )}
      <div className="overflow-x-auto">
        <table className="min-w-full text-left text-sm text-on-background/80">
          <caption className="sr-only">Player sanity list</caption>
          <thead className="border-b border-on-background/20 text-on-background">
            <tr>
              <th className="px-2 py-2 font-medium">Name</th>
              <th className="px-2 py-2 font-medium">CA</th>
              <th className="px-2 py-2 font-medium">Club</th>
            </tr>
          </thead>
          <tbody>
            {players.length === 0 ? (
              <tr>
                <td className="px-2 py-2" colSpan={3}>
                  No players in the current snapshot.
                </td>
              </tr>
            ) : (
              players.map((player) => (
                <tr
                  key={`${player.name}-${player.ca}`}
                  className="border-b border-on-background/10"
                >
                  <td className="px-2 py-2">{player.name}</td>
                  <td className="px-2 py-2">{player.ca}</td>
                  <td className="px-2 py-2">{player.club ?? "—"}</td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
