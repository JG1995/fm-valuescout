import { Outlet } from "@tanstack/react-router";
import { useId } from "react";
import { useLayoutStore } from "@/stores/use-layout-store";
import { cn } from "@/utils/cn";

export function AppShellLayout() {
  const sidebarId = useId();
  const sidebarOpen = useLayoutStore((state) => state.sidebarOpen);
  const toggleSidebar = useLayoutStore((state) => state.toggleSidebar);

  return (
    <div className="flex min-h-screen bg-background text-on-background">
      <aside
        id={sidebarId}
        data-testid="app-sidebar"
        data-open={sidebarOpen ? "true" : "false"}
        className={cn(
          "border-r border-outline bg-surface text-on-surface transition-[width] duration-200",
          sidebarOpen ? "w-48" : "w-0 overflow-hidden border-r-0",
        )}
        aria-hidden={!sidebarOpen}
      >
        <nav className="p-4 text-sm" aria-label="Primary">
          Navigation
        </nav>
      </aside>
      <div className="flex min-h-screen flex-1 flex-col">
        <header
          className="border-b border-outline px-4 py-3"
          data-testid="app-header"
        >
          <button
            type="button"
            className="rounded bg-primary px-3 py-1.5 text-sm text-on-primary"
            onClick={toggleSidebar}
            aria-expanded={sidebarOpen}
            aria-controls={sidebarId}
          >
            Toggle sidebar
          </button>
        </header>
        <main className="flex-1 p-4">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
