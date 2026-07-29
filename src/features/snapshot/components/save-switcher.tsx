import {
  useMutation,
  useQueryClient,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { useState } from "react";
import { Button } from "@/components/ui/button/button";
import { createSave } from "../api/create-save";
import { renameSave } from "../api/rename-save";
import { savesQueryOptions } from "../api/saves-query-options";
import { setActiveSave } from "../api/set-active-save";
import { snapshotKeys } from "../api/snapshot-keys";

export function SaveSwitcher() {
  const queryClient = useQueryClient();
  const { data: saves } = useSuspenseQuery(savesQueryOptions);
  const activeSave = saves.find((save) => save.isActive) ?? saves[0];
  const [newSaveName, setNewSaveName] = useState("");
  const [renameDraft, setRenameDraft] = useState(() => activeSave?.name ?? "");

  const invalidateSnapshotQueries = () => {
    void queryClient.invalidateQueries({ queryKey: snapshotKeys.current() });
    void queryClient.invalidateQueries({
      queryKey: snapshotKeys.sanityPlayers(),
    });
  };

  const create = useMutation({
    mutationFn: createSave,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: snapshotKeys.saves() });
      setNewSaveName("");
    },
  });

  const rename = useMutation({
    mutationFn: ({ saveId, name }: { saveId: number; name: string }) =>
      renameSave(saveId, name),
    onSuccess: (save) => {
      void queryClient.invalidateQueries({ queryKey: snapshotKeys.saves() });
      setRenameDraft(save.name);
    },
  });

  const switchSave = useMutation({
    mutationFn: setActiveSave,
    onSuccess: (save) => {
      void queryClient.invalidateQueries({ queryKey: snapshotKeys.saves() });
      invalidateSnapshotQueries();
      setRenameDraft(save.name);
    },
  });

  return (
    <div className="space-y-3 rounded-md border border-on-background/20 p-4">
      <h2 className="text-lg font-medium text-on-background">Saves</h2>
      <label className="block text-on-background/80" htmlFor="active-save">
        Active save
      </label>
      <select
        id="active-save"
        className="w-full rounded-md border border-on-background/20 bg-background px-3 py-2 text-on-background"
        value={activeSave?.id ?? ""}
        disabled={switchSave.isPending}
        onChange={(event) => {
          const saveId = Number(event.target.value);
          if (!Number.isNaN(saveId) && saveId !== activeSave?.id) {
            switchSave.mutate(saveId);
          }
        }}
      >
        {saves.map((save) => (
          <option key={save.id} value={save.id}>
            {save.name}
          </option>
        ))}
      </select>
      {switchSave.isError && (
        <p className="text-on-background/80">{switchSave.error.message}</p>
      )}
      <div className="space-y-2">
        <label className="block text-on-background/80" htmlFor="rename-save">
          Rename active save
        </label>
        <input
          id="rename-save"
          className="w-full rounded-md border border-on-background/20 bg-background px-3 py-2 text-on-background"
          value={renameDraft}
          onChange={(event) => setRenameDraft(event.target.value)}
        />
        <Button
          type="button"
          variant="secondary"
          disabled={rename.isPending || !activeSave}
          onClick={() => {
            if (!activeSave) {
              return;
            }
            rename.mutate({ saveId: activeSave.id, name: renameDraft });
          }}
        >
          Rename save
        </Button>
        {rename.isError && (
          <p className="text-on-background/80">{rename.error.message}</p>
        )}
      </div>
      <div className="space-y-2">
        <label className="block text-on-background/80" htmlFor="new-save">
          New save
        </label>
        <input
          id="new-save"
          className="w-full rounded-md border border-on-background/20 bg-background px-3 py-2 text-on-background"
          value={newSaveName}
          onChange={(event) => setNewSaveName(event.target.value)}
        />
        <Button
          type="button"
          variant="secondary"
          disabled={create.isPending}
          onClick={() => create.mutate(newSaveName)}
        >
          Create save
        </Button>
        {create.isError && (
          <p className="text-on-background/80">{create.error.message}</p>
        )}
      </div>
    </div>
  );
}
