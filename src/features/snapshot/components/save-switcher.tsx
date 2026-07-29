import {
  useMutation,
  useQueryClient,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { Button } from "@/components/ui/button/button";
import { TextField } from "@/components/ui/field/text-field";
import { Panel } from "@/components/ui/panel/panel";
import { createSave } from "../api/create-save";
import { renameSave } from "../api/rename-save";
import { savesQueryOptions } from "../api/saves-query-options";
import { snapshotKeys } from "../api/snapshot-keys";

function readName(form: HTMLFormElement) {
  const name = new FormData(form).get("name");
  return typeof name === "string" ? name : "";
}

// Switching the active save lives in the top bar, where it stays reachable from
// every screen. This panel keeps the rarer management actions.
export function SaveSwitcher() {
  const queryClient = useQueryClient();
  const { data: saves } = useSuspenseQuery(savesQueryOptions);
  const activeSave = saves.find((save) => save.isActive) ?? saves[0];

  const create = useMutation({
    mutationFn: createSave,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: snapshotKeys.saves() });
    },
  });

  const rename = useMutation({
    mutationFn: ({ saveId, name }: { saveId: number; name: string }) =>
      renameSave(saveId, name),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: snapshotKeys.saves() });
    },
  });

  return (
    <Panel title="Saves">
      <div className="grid gap-4 sm:grid-cols-2">
        <form
          className="space-y-2"
          onSubmit={(event) => {
            event.preventDefault();
            if (activeSave) {
              rename.mutate({
                saveId: activeSave.id,
                name: readName(event.currentTarget),
              });
            }
          }}
        >
          {/* Keyed to the save so a draft cannot survive a switch made from the
              top bar and then rename whichever save became active. */}
          <TextField
            key={activeSave?.id}
            label="Rename active save"
            name="name"
            defaultValue={activeSave?.name ?? ""}
            error={rename.isError ? rename.error.message : undefined}
          />
          <Button
            type="submit"
            variant="secondary"
            disabled={!activeSave}
            loading={rename.isPending}
            loadingLabel="Renaming…"
          >
            Rename save
          </Button>
        </form>
        <form
          className="space-y-2"
          onSubmit={(event) => {
            event.preventDefault();
            const form = event.currentTarget;
            create.mutate(readName(form), {
              onSuccess: () => form.reset(),
            });
          }}
        >
          <TextField
            label="New save"
            name="name"
            error={create.isError ? create.error.message : undefined}
          />
          <Button
            type="submit"
            variant="secondary"
            loading={create.isPending}
            loadingLabel="Creating…"
          >
            Create save
          </Button>
        </form>
      </div>
    </Panel>
  );
}
