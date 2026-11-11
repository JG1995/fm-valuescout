import { useQuery } from "@tanstack/react-query";
import { CircleAlert, TrendingUp } from "lucide-react";
import { Button } from "@/components/ui/button/button";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { dashboardProspectsQueryOptions } from "../api/dashboard-query-options";
import type { DashboardContext, DashboardProspect } from "../types/dashboard";
import { slotLabel } from "../utils/slot-label";
import {
  DashboardTable,
  Gain,
  PlayerCell,
  rowClass,
  Score,
  SlotCell,
} from "./dashboard-table";

function ProspectRow({ prospect }: { prospect: DashboardProspect }) {
  const label = slotLabel(prospect);
  return (
    <tr className={rowClass}>
      <PlayerCell
        uid={prospect.uid}
        name={prospect.name}
        detail={`${prospect.age} years`}
      />
      <SlotCell label={label} />
      <td className="px-2">
        <span className="flex items-center justify-end gap-1">
          <Score
            score={prospect.currentScore}
            name={`${prospect.name} now at ${label.compact}`}
          />
          <span
            aria-hidden="true"
            className="text-label-sm text-on-surface-variant"
          >
            →
          </span>
          <Score
            score={prospect.potentialScore}
            name={`${prospect.name} potential at ${label.compact}`}
          />
        </span>
      </td>
      <td className="px-2 text-right">
        <Gain points={prospect.potentialScore - prospect.currentScore} />
      </td>
    </tr>
  );
}

/** Young club players with the highest potential in the tactic. */
export function DevelopmentProspectsPanel({
  context,
}: {
  context: DashboardContext;
}) {
  const query = useQuery(dashboardProspectsQueryOptions(context));
  const prospects = query.data ?? null;
  return (
    <Panel
      title="Development prospects"
      flush
      actions={
        <p className="text-body-sm text-on-surface-variant">
          Aged 21 or younger
        </p>
      }
    >
      {query.isError ? (
        <EmptyState
          icon={CircleAlert}
          title="Could not load development prospects"
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
          Loading development prospects…
        </p>
      ) : prospects === null || prospects.length === 0 ? (
        <EmptyState icon={TrendingUp} title="No prospects with room to grow">
          No player aged 21 or younger is projected to improve in a slot of your
          tactic.
        </EmptyState>
      ) : (
        <div className="max-h-[22.5rem] overflow-auto border-t border-outline-variant">
          <DashboardTable
            caption="Your young players by potential score in their best tactic slot"
            headings={[
              { label: "Player" },
              { label: "Best slot", width: "w-44" },
              { label: "Now → Potential", numeric: true, width: "w-36" },
              { label: "Gain", numeric: true, width: "w-14" },
            ]}
          >
            {prospects.map((prospect) => (
              <ProspectRow key={prospect.uid} prospect={prospect} />
            ))}
          </DashboardTable>
        </div>
      )}
    </Panel>
  );
}
