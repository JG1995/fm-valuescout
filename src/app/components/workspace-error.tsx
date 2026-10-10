import { useQueryErrorResetBoundary } from "@tanstack/react-query";
import { type ErrorComponentProps, useRouter } from "@tanstack/react-router";
import { useEffect } from "react";
import { Button } from "@/components/ui/button/button";

export function WorkspaceError({ error }: ErrorComponentProps) {
  const router = useRouter();
  const queryErrorResetBoundary = useQueryErrorResetBoundary();

  useEffect(() => {
    queryErrorResetBoundary.reset();
  }, [queryErrorResetBoundary]);

  return (
    <section className="space-y-4">
      <h1 className="text-headline-md text-on-surface">
        Could not load workspace
      </h1>
      <p
        role="alert"
        className="max-w-prose break-words text-body-md text-on-surface-variant"
      >
        {error.message}
      </p>
      <Button variant="secondary" onClick={() => router.invalidate()}>
        Retry
      </Button>
    </section>
  );
}
