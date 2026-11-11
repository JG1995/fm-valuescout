import { useQuery } from "@tanstack/react-query";
import { Maximize2 } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button/button";
import { dashboardProspectsQueryOptions } from "../api/dashboard-query-options";
import type { DashboardContext, DashboardGraphics } from "../types/dashboard";
import { YouthXiModal } from "./youth-xi-modal";

/**
 * A button that opens the club's Youth XI, for a page outside the dashboard.
 * It stays disabled until a slot has a young player. Opening reads again: the
 * page may have changed the tactic since the last read.
 */
export function YouthXiAction({
  context,
  graphics,
}: {
  context: DashboardContext;
  graphics: DashboardGraphics;
}) {
  const [open, setOpen] = useState(false);
  const query = useQuery(dashboardProspectsQueryOptions(context));
  const slots = query.data?.youthXi ?? [];
  return (
    <>
      <Button
        variant="secondary"
        icon={Maximize2}
        disabled={!slots.some((slot) => slot.player !== null)}
        onClick={() => {
          setOpen(true);
          void query.refetch();
        }}
      >
        Youth XI
      </Button>
      <YouthXiModal
        open={open}
        onClose={() => setOpen(false)}
        slots={slots}
        graphics={graphics}
      />
    </>
  );
}
