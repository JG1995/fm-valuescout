import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { fieldClasses } from "@/components/ui/field/field-styles";
import { cn } from "@/utils/cn";
import { savesQueryOptions } from "../api/saves-query-options";
import { setActiveSave } from "../api/set-active-save";
import { snapshotKeys } from "../api/snapshot-keys";

// Shell chrome, so this uses useQuery rather than the route loader's suspense
// pattern: the top bar renders on every route, including ones with no loader,
// and a failed save list must not blank the whole window.
export function ActiveSaveSelect({ className }: { className?: string }) {
  const queryClient = useQueryClient();
  const { data: saves } = useQuery(savesQueryOptions);
  const activeSave = saves?.find((save) => save.isActive) ?? saves?.[0];

  const switchSave = useMutation({
    mutationFn: setActiveSave,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: snapshotKeys.all });
    },
  });

  return (
    <div className={cn("flex items-center gap-2", className)}>
      <select
        aria-label="Active save"
        aria-invalid={switchSave.isError || undefined}
        className={cn(
          fieldClasses,
          "max-w-52",
          switchSave.isError && "border-error",
        )}
        disabled={!saves || switchSave.isPending}
        value={activeSave?.id ?? ""}
        onChange={(event) => {
          const saveId = Number(event.target.value);
          if (!Number.isNaN(saveId) && saveId !== activeSave?.id) {
            switchSave.mutate(saveId);
          }
        }}
      >
        {saves ? (
          saves.map((save) => (
            <option key={save.id} value={save.id}>
              {save.name}
            </option>
          ))
        ) : (
          <option value="">Loading saves…</option>
        )}
      </select>
      {/* A failed switch silently snaps the selection back, which reads as the
          click not registering. */}
      {switchSave.isError ? (
        <p className="text-body-sm text-error">
          Could not switch save. {switchSave.error.message}
        </p>
      ) : null}
    </div>
  );
}
