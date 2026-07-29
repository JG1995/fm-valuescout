import { Outlet } from "@tanstack/react-router";
import { AppNavRail } from "@/app/components/app-nav-rail";
import { AppTopBar } from "@/app/components/app-top-bar";

export function AppShellLayout() {
  return (
    // One window, not a page: the rail and top bar hold their place and only the
    // content area scrolls, so the rail's collapse toggle stays reachable.
    <div className="relative flex h-screen overflow-hidden bg-background text-on-surface">
      <a
        href="#main-content"
        className="sr-only rounded-md bg-primary px-4 py-2 text-label-lg text-on-primary focus:not-sr-only focus:absolute focus:top-2 focus:left-2 focus:z-50"
      >
        Skip to content
      </a>
      <AppNavRail />
      <div className="flex min-w-0 flex-1 flex-col">
        <AppTopBar />
        <main id="main-content" className="flex-1 overflow-y-auto p-4">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
