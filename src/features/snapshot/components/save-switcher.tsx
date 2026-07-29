import {
  useMutation,
  useQueryClient,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { Button } from "@/components/ui/button/button";
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
    <div className="space-y-3 rounded-md border border-on-background/20 p-4">
      <h2 className="text-lg font-medium text-on-background">Saves</h2>
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
        <label className="block text-on-background/80" htmlFor="rename-save">
          Rename active save
        </label>
        {/* Keyed to the save so a draft cannot survive a switch made from the top
            bar and then rename whichever save became active. */}
        <input
          key={activeSave?.id}
          id="rename-save"
          name="name"
          defaultValue={activeSave?.name ?? ""}
          className="w-full rounded-md border border-on-background/20 bg-background px-3 py-2 text-on-background"
        />
        <Button type="submit" variant="secondary" disabled={!activeSave}>
          Rename save
        </Button>
        {rename.isError && (
          <p className="text-on-background/80">{rename.error.message}</p>
        )}
      </form>
      <form
        className="space-y-2"
        onSubmit={(event) => {
          event.preventDefault();
          const form = event.currentTarget;
          create.mutate(readName(form), { onSuccess: () => form.reset() });
        }}
      >
        <label className="block text-on-background/80" htmlFor="new-save">
          New save
        </label>
        <input
          id="new-save"
          name="name"
          className="w-full rounded-md border border-on-background/20 bg-background px-3 py-2 text-on-background"
        />
        <Button type="submit" variant="secondary">
          Create save
        </Button>
        {create.isError && (
          <p className="text-on-background/80">{create.error.message}</p>
        )}
      </form>
    </div>
  );
}
