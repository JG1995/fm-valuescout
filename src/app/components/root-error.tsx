import { useQueryErrorResetBoundary } from "@tanstack/react-query";
import {
  type ErrorComponentProps,
  Link,
  useRouter,
} from "@tanstack/react-router";
import { useEffect } from "react";
import { Button } from "@/components/ui/button/button";

// A plain path, like the nav's destinations: Search's validateSearch fills the
// sort, filters, and preferred analysis view instead of this link fixing them.
const playerSearchPath: string = "/search";

/** Replaces the whole shell when it fails, so it brings its own landmark and a
 *  route back into the app instead of the router's default debug panel. */
export function RootError({ error }: ErrorComponentProps) {
  const router = useRouter();
  const queryErrorResetBoundary = useQueryErrorResetBoundary();

  useEffect(() => {
    queryErrorResetBoundary.reset();
  }, [queryErrorResetBoundary]);

  return (
    <main className="space-y-4 p-gutter">
      <h1 className="text-headline-md text-on-surface">Something went wrong</h1>
      <p
        role="alert"
        className="max-w-prose break-words text-body-md text-on-surface-variant"
      >
        {error instanceof Error ? error.message : String(error)}
      </p>
      <div className="flex gap-2">
        <Button variant="secondary" onClick={() => router.invalidate()}>
          Retry
        </Button>
        <Link
          to={playerSearchPath}
          className="inline-flex h-8 items-center rounded-md px-4 text-label-lg text-on-surface-variant transition-colors duration-150 ease-out hover:bg-surface-container-high hover:text-on-surface"
        >
          Open Player Search
        </Link>
      </div>
    </main>
  );
}
