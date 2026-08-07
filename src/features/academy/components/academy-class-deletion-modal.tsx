import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { Modal } from "@/components/ui/modal/modal";
import { academyKeys } from "../api/academy-keys";
import { deleteAcademyClass } from "../api/delete-academy-class";
import type { AcademyClass } from "../types/academy";

type AcademyClassDeletionModalProps = {
  target: AcademyClass | null;
  onClose: () => void;
  onDeleted: () => void;
};

export function AcademyClassDeletionModal({
  target,
  onClose,
  onDeleted,
}: AcademyClassDeletionModalProps) {
  const queryClient = useQueryClient();
  const [visibleTarget, setVisibleTarget] = useState<AcademyClass | null>(
    target,
  );
  const remove = useMutation({
    mutationFn: () => deleteAcademyClass(visibleTarget?.id ?? 0),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: academyKeys.classes() });
      onDeleted();
    },
  });
  const { reset } = remove;

  useEffect(() => {
    if (target) {
      setVisibleTarget(target);
      reset();
    }
  }, [reset, target]);

  if (!visibleTarget) {
    return null;
  }

  return (
    <Modal
      open={target !== null}
      title={`Delete Class of ${visibleTarget.classYear}?`}
      variant="destructive"
      onClose={() => {
        if (!remove.isPending) {
          onClose();
        }
      }}
      footer={
        <>
          <Button
            variant="secondary"
            disabled={remove.isPending}
            onClick={onClose}
          >
            Cancel
          </Button>
          <Button
            variant="destructive"
            loading={remove.isPending}
            loadingLabel="Deleting…"
            onClick={() => remove.mutate()}
          >
            Delete class
          </Button>
        </>
      }
    >
      <p className="text-body-md text-on-surface-variant">
        This removes the class and its {visibleTarget.memberCount} tracked
        player{visibleTarget.memberCount === 1 ? "" : "s"}.
      </p>
      {remove.isError ? (
        <p className="mt-3 text-body-sm text-error" role="alert">
          {remove.error.message}
        </p>
      ) : null}
    </Modal>
  );
}
