import { useQuery } from "@tanstack/react-query";
import { CircleCheck, CircleDashed, Clock, TriangleAlert } from "lucide-react";
import { StatusChip } from "@/components/ui/status-chip/status-chip";
import { formatAbsoluteUtc, formatRelativeAge } from "@/utils/format";
import { currentSnapshotQueryOptions } from "../api/current-snapshot-query-options";

const FRESH_MS = 30 * 60_000;
const STALE_MS = 6 * 60 * 60_000;

export function SnapshotFreshnessChip() {
  const { data: snapshot, isPending } = useQuery(currentSnapshotQueryOptions);

  if (isPending) {
    return (
      <StatusChip tone="neutral" icon={CircleDashed}>
        Checking snapshot…
      </StatusChip>
    );
  }

  if (!snapshot) {
    return (
      <StatusChip tone="neutral" icon={CircleDashed}>
        No data loaded
      </StatusChip>
    );
  }

  const age = Date.now() - Date.parse(snapshot.loadedAtUtc);
  const stale = Number.isNaN(age) || age >= STALE_MS;
  const tone =
    snapshot.scanTruncated || stale
      ? "warning"
      : age < FRESH_MS
        ? "success"
        : "neutral";
  const icon =
    tone === "warning"
      ? TriangleAlert
      : tone === "success"
        ? CircleCheck
        : Clock;

  return (
    <span title={`Loaded ${formatAbsoluteUtc(snapshot.loadedAtUtc)}`}>
      <StatusChip tone={tone} icon={icon}>
        {snapshot.scanTruncated
          ? `Capped · ${formatRelativeAge(snapshot.loadedAtUtc)}`
          : `Loaded ${formatRelativeAge(snapshot.loadedAtUtc)}`}
      </StatusChip>
    </span>
  );
}
