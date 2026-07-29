import {
  useMutation,
  useQueryClient,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { CircleCheck, CircleX, RefreshCw } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button/button";
import { TextField } from "@/components/ui/field/text-field";
import { Panel } from "@/components/ui/panel/panel";
import { StatusChip } from "@/components/ui/status-chip/status-chip";
import { formatMissable } from "@/utils/format";
import { demoValueQueryOptions } from "../api/demo-value-query-options";
import { healthQueryOptions } from "../api/health-query-options";
import { setHealthSimulateError } from "../api/health-simulate-error";
import { setDemoValue } from "../api/set-demo-value";

export function HealthStatusPanel() {
  const queryClient = useQueryClient();
  const { data } = useSuspenseQuery(healthQueryOptions);
  const { data: demoValue } = useSuspenseQuery(demoValueQueryOptions);
  const [draftValue, setDraftValue] = useState(() => demoValue.value);

  const saveDemoValue = useMutation({
    mutationFn: setDemoValue,
    onSuccess: (savedValue) => {
      queryClient.setQueryData(demoValueQueryOptions.queryKey, savedValue);
      setDraftValue(savedValue.value);
    },
  });

  return (
    <Panel
      title="Health"
      actions={
        <Button
          size="icon"
          variant="ghost"
          icon={RefreshCw}
          aria-label="Refresh status"
          onClick={() =>
            queryClient.invalidateQueries({
              queryKey: healthQueryOptions.queryKey,
            })
          }
        />
      }
    >
      <StatusChip
        tone={data.status === "ok" ? "success" : "error"}
        icon={data.status === "ok" ? CircleCheck : CircleX}
      >
        {`Status: ${data.status}`}
      </StatusChip>

      <div className="mt-4 max-w-sm space-y-2">
        <TextField
          label="Demo value (SQLite):"
          value={draftValue}
          onChange={(event) => setDraftValue(event.target.value)}
          error={
            saveDemoValue.isError ? saveDemoValue.error.message : undefined
          }
        />
        <p className="text-body-sm text-on-surface-variant">
          Stored value:{" "}
          <span className="font-mono text-mono-sm text-on-surface">
            {formatMissable(demoValue.value)}
          </span>
        </p>
        <div className="flex flex-wrap gap-2">
          <Button
            variant="secondary"
            loading={saveDemoValue.isPending}
            loadingLabel="Saving…"
            onClick={() => saveDemoValue.mutate(draftValue)}
          >
            Save demo value
          </Button>
          <Button
            variant="ghost"
            onClick={() => {
              setHealthSimulateError(true);
              queryClient.resetQueries({
                queryKey: healthQueryOptions.queryKey,
              });
            }}
          >
            Simulate error
          </Button>
        </div>
      </div>
    </Panel>
  );
}
