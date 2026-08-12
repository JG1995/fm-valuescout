import { Link } from "@tanstack/react-router";
import type { LucideIcon } from "lucide-react";
import {
  GraduationCap,
  LayoutDashboard,
  PanelLeftClose,
  PanelLeftOpen,
  Search,
  UsersRound,
} from "lucide-react";
import { Button } from "@/components/ui/button/button";
import { useLayoutStore } from "@/stores/use-layout-store";
import { cn } from "@/utils/cn";

type NavItem = {
  to: string;
  label: string;
  icon: LucideIcon;
};

const navItems: NavItem[] = [
  { to: "/", label: "Dashboard", icon: LayoutDashboard },
  { to: "/search", label: "Search", icon: Search },
  { to: "/planner", label: "Squad", icon: UsersRound },
  { to: "/academy", label: "Youth Academy", icon: GraduationCap },
];

export function AppNavRail() {
  const railExpanded = useLayoutStore((state) => state.railExpanded);
  const toggleRail = useLayoutStore((state) => state.toggleRail);

  return (
    <nav
      aria-label="Primary"
      data-testid="app-nav-rail"
      data-expanded={railExpanded ? "true" : "false"}
      className={cn(
        "flex shrink-0 flex-col gap-1 border-r border-outline-variant bg-surface-container-lowest p-2",
        "transition-[width] duration-150 ease-out",
        railExpanded ? "w-rail-width-expanded" : "w-rail-width",
      )}
    >
      <div className="flex h-10 items-center gap-2 px-1">
        <span
          aria-hidden="true"
          className="grid size-8 shrink-0 place-items-center rounded-md bg-primary font-mono text-mono-sm text-on-primary"
        >
          VS
        </span>
        {railExpanded ? (
          <span className="truncate text-label-lg text-on-surface">
            FM ValueScout
          </span>
        ) : null}
      </div>

      <ul className="flex flex-col gap-1">
        {navItems.map((item) => (
          <li key={item.to}>
            <Link
              to={item.to}
              activeOptions={{ exact: true }}
              title={railExpanded ? undefined : item.label}
              className={cn(
                "relative flex h-10 items-center gap-3 rounded-md px-3 text-label-lg text-on-surface-variant",
                "transition-colors duration-150 ease-out hover:bg-surface-container-high hover:text-on-surface",
                !railExpanded && "justify-center px-0",
              )}
              activeProps={{
                "aria-current": "page",
                className:
                  "bg-primary-container text-primary before:absolute before:inset-y-1 before:left-0 before:w-0.5 before:rounded-full before:bg-primary",
              }}
            >
              <item.icon
                aria-hidden="true"
                size={20}
                strokeWidth={1.5}
                className="shrink-0"
              />
              {railExpanded ? item.label : null}
            </Link>
          </li>
        ))}
      </ul>

      <Button
        size="icon"
        variant="ghost"
        icon={railExpanded ? PanelLeftClose : PanelLeftOpen}
        aria-label="Toggle navigation"
        aria-expanded={railExpanded}
        className={cn("mt-auto", railExpanded && "self-end")}
        onClick={toggleRail}
      />
    </nav>
  );
}
