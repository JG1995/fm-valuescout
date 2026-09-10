import {
  useMutation,
  useQueryClient,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { CircleCheck, CircleDashed, CircleX, RefreshCw } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button/button";
import { Panel } from "@/components/ui/panel/panel";
import { StatusChip } from "@/components/ui/status-chip/status-chip";
import { graphicsKeys } from "../api/graphics-keys";
import {
  chooseGraphicsRoot,
  clearGraphicsRoot,
  rescanGraphics,
} from "../api/graphics-mutations";
import { graphicsStatusQueryOptions } from "../api/graphics-query-options";
import type { GraphicsStatus } from "../types/graphics";

function useGraphicsMutation(
  mutationFn: () => Promise<GraphicsStatus>,
  onCancellation: () => void,
) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn,
    onMutate: () => ({
      previousGeneration: queryClient.getQueryData<GraphicsStatus>(
        graphicsKeys.status(),
      )?.generation,
    }),
    onSuccess: (status, _variables, context) => {
      if (
        context?.previousGeneration !== undefined &&
        status.generation === context.previousGeneration
      ) {
        onCancellation();
        return;
      }

      queryClient.removeQueries({
        queryKey: graphicsKeys.results(),
        predicate: (query) => {
          const generation = (
            query.queryKey[2] as { generation?: number } | undefined
          )?.generation;
          return (
            generation !== undefined &&
            generation <= (context?.previousGeneration ?? status.generation)
          );
        },
      });
      queryClient.setQueryData(graphicsKeys.status(), status);
      void queryClient.invalidateQueries({ queryKey: graphicsKeys.status() });
      void queryClient.invalidateQueries({ queryKey: graphicsKeys.all });
    },
  });
}

const diagnosticLabels: [
  keyof GraphicsStatus["summary"]["diagnostics"],
  string,
][] = [
  ["configLimit", "config-limit hits"],
  ["entryLimit", "entry-limit hits"],
  ["depthLimit", "depth-limit hits"],
  ["mappingLimit", "mapping-limit hits"],
  ["configTooLarge", "oversized configs"],
  ["configUnreadable", "unreadable configs"],
  ["malformedConfig", "malformed configs"],
  ["invalidMapping", "invalid mappings"],
  ["sourceUnreadable", "unreadable sources"],
];

type GraphicsAction = "choose" | "clear" | "rescan";

export function GraphicsSettingsSection() {
  const { data } = useSuspenseQuery(graphicsStatusQueryOptions);
  const [cancelledGeneration, setCancelledGeneration] = useState<number | null>(
    null,
  );
  const [activeAction, setActiveAction] = useState<GraphicsAction | null>(null);
  const choose = useGraphicsMutation(chooseGraphicsRoot, () =>
    setCancelledGeneration(data.generation),
  );
  const clear = useGraphicsMutation(clearGraphicsRoot, () =>
    setCancelledGeneration(null),
  );
  const rescan = useGraphicsMutation(rescanGraphics, () =>
    setCancelledGeneration(null),
  );
  const pending = choose.isPending || clear.isPending || rescan.isPending;
  const activeMutation =
    activeAction === "choose"
      ? choose
      : activeAction === "clear"
        ? clear
        : activeAction === "rescan"
          ? rescan
          : null;
  const error = activeMutation?.error ?? null;
  const diagnostics = diagnosticLabels.filter(
    ([key]) => data.summary.diagnostics[key] > 0,
  );
  const statusTone = data.summary.truncated
    ? "warning"
    : data.selected
      ? "success"
      : "neutral";

  return (
    <Panel title="Graphics">
      <div className="flex flex-wrap items-center gap-2">
        <StatusChip
          tone={statusTone}
          icon={
            data.summary.truncated
              ? CircleX
              : data.selected
                ? CircleCheck
                : CircleDashed
          }
        >
          {data.summary.truncated
            ? "Scan capped"
            : data.selected
              ? "Root selected"
              : "No root selected"}
        </StatusChip>
        {data.candidate.available ? (
          <StatusChip tone="info" icon={CircleCheck}>
            {`Detected ${data.candidate.source === "onedrive" ? "OneDrive" : "Documents"} candidate`}
          </StatusChip>
        ) : null}
        <StatusChip
          tone="neutral"
          icon={RefreshCw}
        >{`Generation ${data.generation}`}</StatusChip>
      </div>
      <p className="mt-3 max-w-prose text-body-sm text-on-surface-variant">
        Local FM26 graphics are used for portraits and club logos. The app keeps
        the selected root private and never displays its path.
      </p>
      <p className="mt-2 text-body-sm text-on-surface-variant">
        Indexed {data.summary.configs} configs and {data.summary.mappings}{" "}
        mappings.
      </p>
      {diagnostics.length > 0 ? (
        <p className="mt-2 max-w-prose text-body-sm text-warning">
          Diagnostics:{" "}
          {diagnostics
            .map(([key, label]) => `${data.summary.diagnostics[key]} ${label}`)
            .join(", ")}
          .
        </p>
      ) : null}
      <div className="mt-4 flex flex-wrap gap-2">
        <Button
          loading={choose.isPending}
          loadingLabel="Choosing…"
          disabled={pending}
          onClick={() => {
            setActiveAction("choose");
            setCancelledGeneration(null);
            choose.mutate();
          }}
        >
          Choose graphics folder
        </Button>
        <Button
          variant="secondary"
          loading={rescan.isPending}
          loadingLabel="Rescanning…"
          disabled={pending || !data.selected}
          onClick={() => {
            setActiveAction("rescan");
            setCancelledGeneration(null);
            rescan.mutate();
          }}
        >
          Rescan graphics
        </Button>
        <Button
          variant="ghost"
          loading={clear.isPending}
          loadingLabel="Clearing…"
          disabled={pending || !data.selected}
          onClick={() => {
            setActiveAction("clear");
            setCancelledGeneration(null);
            clear.mutate();
          }}
        >
          Clear graphics folder
        </Button>
      </div>
      {error ? (
        <p className="mt-3 text-body-sm text-error">
          Graphics update failed: {error.message}
        </p>
      ) : null}
      {activeAction === "choose" &&
      choose.isSuccess &&
      cancelledGeneration === data.generation ? (
        <p className="mt-3 text-body-sm text-on-surface-variant">
          Folder choice cancelled. Graphics settings are unchanged.
        </p>
      ) : activeMutation?.isSuccess ? (
        <p className="mt-3 text-body-sm text-success">
          Graphics settings updated. The index is rebuilding in the background.
        </p>
      ) : null}
    </Panel>
  );
}
