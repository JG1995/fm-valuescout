import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { TextField } from "@/components/ui/field/text-field";
import { Modal } from "@/components/ui/modal/modal";
import { academyKeys } from "../api/academy-keys";
import { createAcademyClass } from "../api/create-academy-class";
import type { AcademyClass } from "../types/academy";

type AcademyClassCreationModalProps = {
  open: boolean;
  prefillYear: number | null;
  onClose: () => void;
  onCreated: (academyClass: AcademyClass) => void;
};

export function AcademyClassCreationModal({
  open,
  prefillYear,
  onClose,
  onCreated,
}: AcademyClassCreationModalProps) {
  const queryClient = useQueryClient();
  const [year, setYear] = useState("");
  const [validationError, setValidationError] = useState<string | undefined>();
  const create = useMutation({
    mutationFn: () => createAcademyClass(Number(year)),
    onSuccess: (academyClass) => {
      void queryClient.invalidateQueries({ queryKey: academyKeys.classes() });
      onCreated(academyClass);
    },
  });
  const { reset } = create;

  useEffect(() => {
    if (open) {
      setYear(prefillYear?.toString() ?? "");
      setValidationError(undefined);
      reset();
    }
  }, [open, prefillYear, reset]);

  const error =
    validationError ?? (create.isError ? create.error.message : undefined);

  return (
    <Modal
      open={open}
      title="Create academy class"
      onClose={onClose}
      footer={
        <>
          <Button
            variant="secondary"
            disabled={create.isPending}
            onClick={onClose}
          >
            Cancel
          </Button>
          <Button
            type="submit"
            form="academy-class-create-form"
            loading={create.isPending}
            loadingLabel="Creating…"
          >
            Create class
          </Button>
        </>
      }
    >
      <form
        id="academy-class-create-form"
        className="space-y-4"
        onSubmit={(event) => {
          event.preventDefault();
          const parsedYear = Number(year);
          if (!Number.isInteger(parsedYear) || parsedYear <= 0) {
            setValidationError("Enter a positive whole number");
            return;
          }
          setValidationError(undefined);
          create.mutate();
        }}
      >
        <p className="text-body-md text-on-surface-variant">
          Group players by the year they came through your club.
        </p>
        <p className="text-headline-sm text-on-surface" aria-live="polite">
          Class of {year || "YYYY"}
        </p>
        <TextField
          label="Class year"
          min={1}
          name="classYear"
          step={1}
          type="number"
          value={year}
          error={error}
          onChange={(event) => {
            setYear(event.target.value);
            setValidationError(undefined);
            reset();
          }}
        />
      </form>
    </Modal>
  );
}
