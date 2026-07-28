import {
  useMutation,
  useQueryClient,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { useState } from "react";
import { Button } from "@/components/ui/button/button";
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
    <div className="space-y-3">
      <p className="text-on-background/80">
        Status: <strong className="text-on-background">{data.status}</strong>
      </p>
      <div className="space-y-2">
        <label className="block text-on-background/80" htmlFor="demo-value">
          Demo value (SQLite):
        </label>
        <input
          id="demo-value"
          className="w-full rounded-md border border-on-background/20 bg-background px-3 py-2 text-on-background"
          value={draftValue}
          onChange={(event) => setDraftValue(event.target.value)}
        />
        <p className="text-on-background/80">
          Stored value:{" "}
          <strong className="text-on-background">{demoValue.value}</strong>
        </p>
        <Button
          type="button"
          variant="secondary"
          disabled={saveDemoValue.isPending}
          onClick={() => saveDemoValue.mutate(draftValue)}
        >
          Save demo value
        </Button>
        {saveDemoValue.isError && (
          <p className="text-on-background/80">
            Could not save demo value.{" "}
            <span className="text-on-background">
              {saveDemoValue.error.message}
            </span>
          </p>
        )}
      </div>
      <div className="flex flex-wrap gap-2">
        <Button
          type="button"
          variant="secondary"
          onClick={() =>
            queryClient.invalidateQueries({
              queryKey: healthQueryOptions.queryKey,
            })
          }
        >
          Refresh status
        </Button>
        <Button
          type="button"
          variant="secondary"
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
  );
}
