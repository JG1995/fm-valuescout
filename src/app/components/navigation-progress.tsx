import { useRouterState } from "@tanstack/react-router";

/**
 * A slim bar on the navigation's lower edge while a navigation waits for its
 * data. It stays invisible for the first 150ms, so the usual instant
 * navigation never shows it.
 */
export function NavigationProgress() {
  const isPending = useRouterState({
    select: (state) => state.status === "pending",
  });
  if (!isPending) return null;
  return (
    <div
      role="progressbar"
      aria-label="Loading page"
      data-testid="navigation-progress"
      className="pointer-events-none absolute inset-x-0 bottom-0 h-0.5 animate-enter-late overflow-hidden"
    >
      <div className="h-full w-1/3 animate-progress rounded-full bg-primary motion-reduce:w-full motion-reduce:animate-none" />
    </div>
  );
}
