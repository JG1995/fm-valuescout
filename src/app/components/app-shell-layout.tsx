import { Outlet } from "@tanstack/react-router";
import { AppNavBar } from "@/app/components/app-nav-bar";
import { AppTopBar } from "@/app/components/app-top-bar";

export function AppShellLayout() {
  return (
    // One window, not a page: the utility bar and navigation bar hold their
    // place and only the content area scrolls.
    <div className="relative flex h-screen flex-col overflow-hidden bg-background text-on-surface">
      <a
        href="#main-content"
        className="sr-only rounded-md bg-primary px-4 py-2 text-label-lg text-on-primary focus:not-sr-only focus:absolute focus:top-2 focus:left-2 focus:z-50"
      >
        Skip to content
      </a>
      <AppTopBar />
      <AppNavBar />
      <main id="main-content" className="min-h-0 flex-1 overflow-y-auto p-4">
        <Outlet />
      </main>
    </div>
  );
}
