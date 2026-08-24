import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useMemo, useRef, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { Modal } from "@/components/ui/modal/modal";
import {
  HIDDEN_ATTRIBUTE_KEYS,
  labelFromPascal,
  PERSONALITY_ATTRIBUTE_KEYS,
  VISIBLE_ATTRIBUTE_GROUPS,
} from "@/utils/player-attributes";
import { clubDnaKeys } from "../api/club-dna-keys";
import { clubDnaQueryOptions } from "../api/club-dna-query-options";
import { removeClubDna } from "../api/remove-club-dna";
import { setClubDna } from "../api/set-club-dna";
import type {
  ClubDnaContext,
  ClubDnaRemoveResult,
  ClubDnaUpsertResult,
} from "../types/club-dna";

type ClubDnaDefinitionProps = {
  context: ClubDnaContext;
  available: boolean;
  onSaved?: (result: ClubDnaUpsertResult, context: ClubDnaContext) => void;
  onRemoved?: (result: ClubDnaRemoveResult, context: ClubDnaContext) => void;
};

type AttributeOption = {
  id: string;
  label: string;
};

type AttributeSection = {
  title: string;
  options: readonly AttributeOption[];
};

function options(
  prefix: "attr" | "hidden" | "personality",
  keys: readonly string[],
) {
  return keys.map((key) => ({
    id: `${prefix}.${key}`,
    label: labelFromPascal(key),
  }));
}

const ATTRIBUTE_SECTIONS: readonly AttributeSection[] = [
  ...VISIBLE_ATTRIBUTE_GROUPS.flatMap((group) => [
    { title: group.title, options: options("attr", group.keys) },
    ...(group.subgroups?.map((subgroup) => ({
      title: subgroup.title,
      options: options("attr", subgroup.keys),
    })) ?? []),
  ]),
  { title: "Hidden", options: options("hidden", HIDDEN_ATTRIBUTE_KEYS) },
  {
    title: "Personality",
    options: options("personality", PERSONALITY_ATTRIBUTE_KEYS),
  },
] as const;

const ALL_OPTIONS = ATTRIBUTE_SECTIONS.flatMap((section) => section.options);
const FORMULA =
  "Club DNA scales each selected 1–20 value by 5, gives every selected attribute equal weight, averages the values, and rounds to a whole 0–100 score.";

function contextKey({ saveId, contextToken }: ClubDnaContext) {
  return `${saveId}:${contextToken}`;
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

function orderedSelection(attributeIds: readonly string[]) {
  const selected = new Set(attributeIds);
  return ALL_OPTIONS.filter((option) => selected.has(option.id)).map(
    (option) => option.id,
  );
}

export function ClubDnaDefinition({
  context,
  available,
  onSaved,
  onRemoved,
}: ClubDnaDefinitionProps) {
  const queryClient = useQueryClient();
  const [open, setOpen] = useState(false);
  const [confirmRemoval, setConfirmRemoval] = useState(false);
  const [draft, setDraft] = useState<string[]>([]);
  const [returnFocusToSave, setReturnFocusToSave] = useState(false);
  const key = contextKey(context);
  const currentKeyRef = useRef(key);
  const previousKeyRef = useRef(key);
  currentKeyRef.current = key;
  const definition = useQuery({
    ...clubDnaQueryOptions(context),
    enabled: open,
  });
  const setDefinition = useMutation({
    mutationFn: ({
      requestContext,
      attributeIds,
    }: {
      requestContext: ClubDnaContext;
      attributeIds: string[];
    }) => setClubDna(requestContext, attributeIds),
    onSuccess: (result, variables) => {
      if (currentKeyRef.current !== contextKey(variables.requestContext)) {
        return;
      }
      queryClient.setQueryData(
        clubDnaKeys.definition(variables.requestContext),
        result.definition,
      );
      setOpen(false);
      onSaved?.(result, variables.requestContext);
    },
  });
  const removeDefinition = useMutation({
    mutationFn: (requestContext: ClubDnaContext) =>
      removeClubDna(requestContext),
    onSuccess: (result, requestContext) => {
      if (currentKeyRef.current !== contextKey(requestContext)) {
        return;
      }
      queryClient.setQueryData(clubDnaKeys.definition(requestContext), null);
      setConfirmRemoval(false);
      setOpen(false);
      onRemoved?.(result, requestContext);
    },
  });

  const pending = setDefinition.isPending || removeDefinition.isPending;
  const selectedOptions = useMemo(() => {
    const selected = new Set(draft);
    return ALL_OPTIONS.filter((option) => selected.has(option.id));
  }, [draft]);

  useEffect(() => {
    if (previousKeyRef.current === key) {
      return;
    }
    previousKeyRef.current = key;
    setOpen(false);
    setConfirmRemoval(false);
    setDraft([]);
    setReturnFocusToSave(false);
    setDefinition.reset();
    removeDefinition.reset();
  }, [key, removeDefinition.reset, setDefinition.reset]);

  useEffect(() => {
    if (!open || !definition.isSuccess) {
      return;
    }
    setDraft(orderedSelection(definition.data?.attributeIds ?? []));
    setConfirmRemoval(false);
    setDefinition.reset();
    removeDefinition.reset();
  }, [
    definition.data,
    definition.isSuccess,
    open,
    removeDefinition.reset,
    setDefinition.reset,
  ]);

  useEffect(() => {
    if (!open || !confirmRemoval) {
      return;
    }
    document
      .querySelector<HTMLButtonElement>("[data-club-dna-confirm-cancel]")
      ?.focus();
  }, [confirmRemoval, open]);

  useEffect(() => {
    if (!open || confirmRemoval || !returnFocusToSave) {
      return;
    }
    document.querySelector<HTMLButtonElement>("[data-club-dna-save]")?.focus();
    setReturnFocusToSave(false);
  }, [confirmRemoval, open, returnFocusToSave]);

  const closeEdit = () => {
    if (pending) {
      return;
    }
    setOpen(false);
    setConfirmRemoval(false);
    setDraft([]);
    setReturnFocusToSave(false);
    setDefinition.reset();
    removeDefinition.reset();
  };

  const leaveConfirmation = () => {
    if (pending) {
      return;
    }
    setConfirmRemoval(false);
    setReturnFocusToSave(true);
  };

  const toggle = (attributeId: string) => {
    if (pending) {
      return;
    }
    setDefinition.reset();
    setDraft((current) =>
      current.includes(attributeId)
        ? current.filter((id) => id !== attributeId)
        : orderedSelection([...current, attributeId]),
    );
  };

  const save = () => {
    if (
      !available ||
      definition.isPending ||
      definition.isError ||
      draft.length === 0
    ) {
      return;
    }
    setDefinition.mutate({
      requestContext: { ...context },
      attributeIds: draft,
    });
  };

  const remove = () => {
    if (!available || pending) {
      return;
    }
    removeDefinition.mutate({ ...context });
  };

  const formError = definition.isError
    ? errorMessage(definition.error)
    : setDefinition.isError
      ? errorMessage(setDefinition.error)
      : null;
  const removeError = removeDefinition.isError
    ? errorMessage(removeDefinition.error)
    : null;
  const canSave =
    available &&
    !pending &&
    !definition.isPending &&
    !definition.isError &&
    draft.length > 0;

  return (
    <>
      <Button
        variant="secondary"
        disabled={!available || pending}
        onClick={() => setOpen(true)}
      >
        Define DNA
      </Button>
      <Modal
        open={open}
        title={confirmRemoval ? "Remove Club DNA?" : "Define Club DNA"}
        variant={confirmRemoval ? "destructive" : "form"}
        onClose={confirmRemoval ? leaveConfirmation : closeEdit}
        footer={
          confirmRemoval ? (
            <>
              <Button
                variant="secondary"
                disabled={pending}
                data-club-dna-confirm-cancel
                onClick={leaveConfirmation}
              >
                Cancel
              </Button>
              <Button
                variant="destructive"
                loading={removeDefinition.isPending}
                loadingLabel="Removing…"
                disabled={!available || pending}
                onClick={remove}
              >
                Remove definition
              </Button>
            </>
          ) : (
            <>
              <Button
                variant="secondary"
                disabled={pending}
                onClick={closeEdit}
              >
                Cancel
              </Button>
              {definition.data ? (
                <Button
                  variant="destructive"
                  disabled={
                    pending ||
                    !available ||
                    definition.isPending ||
                    definition.isError
                  }
                  onClick={() => setConfirmRemoval(true)}
                >
                  Remove Club DNA
                </Button>
              ) : null}
              <Button
                disabled={!canSave}
                loading={setDefinition.isPending}
                loadingLabel="Saving…"
                data-club-dna-save
                onClick={save}
              >
                Save Club DNA
              </Button>
            </>
          )
        }
      >
        {confirmRemoval ? (
          <div className="space-y-3">
            <p className="text-body-md text-on-surface-variant">
              This removes the Club DNA definition and its saved scores.
            </p>
            {removeError ? (
              <p className="text-body-sm text-error" role="alert">
                {removeError}
              </p>
            ) : null}
          </div>
        ) : (
          <form
            className="space-y-5"
            onSubmit={(event) => {
              event.preventDefault();
              save();
            }}
          >
            <p className="text-body-md text-on-surface-variant">{FORMULA}</p>
            {definition.isPending ? (
              <p className="text-body-md text-on-surface-variant" role="status">
                Loading Club DNA…
              </p>
            ) : null}
            {formError ? (
              <p className="text-body-sm text-error" role="alert">
                {formError}
              </p>
            ) : null}
            <fieldset
              className="space-y-4"
              disabled={pending || definition.isPending || definition.isError}
            >
              <legend className="text-label-lg text-on-surface">
                Attributes
              </legend>
              {ATTRIBUTE_SECTIONS.map((section) => (
                <fieldset key={section.title} className="space-y-2">
                  <legend className="text-label-md text-on-surface-variant">
                    {section.title}
                  </legend>
                  <div className="grid gap-2 sm:grid-cols-2">
                    {section.options.map((option) => (
                      <label
                        key={option.id}
                        className="flex items-center gap-2 text-body-md text-on-surface"
                      >
                        <input
                          type="checkbox"
                          checked={draft.includes(option.id)}
                          onChange={() => toggle(option.id)}
                        />
                        {option.label}
                      </label>
                    ))}
                  </div>
                </fieldset>
              ))}
            </fieldset>
            <section
              aria-labelledby="club-dna-selected-attributes"
              className="space-y-2"
            >
              <h3
                id="club-dna-selected-attributes"
                className="text-label-lg text-on-surface"
              >
                Selected attributes ({selectedOptions.length})
              </h3>
              {selectedOptions.length === 0 ? (
                <p className="text-body-sm text-on-surface-variant">
                  Select at least one attribute.
                </p>
              ) : (
                <ul className="list-disc space-y-1 pl-5 text-body-sm text-on-surface">
                  {selectedOptions.map((option) => (
                    <li key={option.id}>{option.label}</li>
                  ))}
                </ul>
              )}
            </section>
          </form>
        )}
      </Modal>
    </>
  );
}
