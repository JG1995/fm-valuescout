import { useQuery } from "@tanstack/react-query";
import { useEffect } from "react";
import type { router as appRouter } from "@/app/router";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";

/** The screens a session usually opens, most likely first. */
export const warmedDestinations = [
  { to: "/search" },
  { to: "/my-club", search: { view: "squad" } },
  { to: "/staff", search: { view: "search" } },
  { to: "/staff", search: { view: "my-staff" } },
  { to: "/academy" },
  { to: "/settings" },
  { to: "/" },
] as const;

type WarmedDestination = (typeof warmedDestinations)[number];

/** Resolves when the browser has nothing more urgent to do. */
function whenIdle(): Promise<void> {
  return new Promise((resolve) => {
    if (typeof requestIdleCallback === "function") {
      requestIdleCallback(() => resolve(), { timeout: 2_000 });
    } else {
      setTimeout(resolve, 200);
    }
  });
}

/**
 * Runs each destination's loader in turn, one at a time and only while the
 * app is idle, so the reads never compete with the screen the user is on.
 */
export async function warmDestinations(
  preload: (destination: WarmedDestination) => Promise<unknown>,
  isCancelled: () => boolean,
  pause: () => Promise<void> = whenIdle,
): Promise<void> {
  for (const destination of warmedDestinations) {
    await pause();
    if (isCancelled()) return;
    await preload(destination).catch(() => undefined);
  }
}

/**
 * Preloads the main screens once a snapshot is known, and again when the
 * snapshot changes (Load Data, a save switch), so the first visit to each
 * paints from cache. A route loader decides what is worth reading early.
 */
export function DestinationWarmer({ router }: { router: typeof appRouter }) {
  const { data: snapshot } = useQuery(currentSnapshotQueryOptions);
  const snapshotIdentity = snapshot
    ? `${snapshot.saveId}:${snapshot.id}`
    : null;

  useEffect(() => {
    if (snapshotIdentity === null) return;
    let cancelled = false;
    void warmDestinations(
      (destination) => router.preloadRoute(destination),
      () => cancelled,
    );
    return () => {
      cancelled = true;
    };
  }, [router, snapshotIdentity]);

  return null;
}
