import { Link, useLocation } from "@tanstack/react-router";
import type { LucideIcon } from "lucide-react";
import {
  CalendarDays,
  ClipboardList,
  GraduationCap,
  LayoutDashboard,
  Search,
  Settings as SettingsIcon,
  Target,
  UserRoundCog,
  UserRoundSearch,
  UsersRound,
} from "lucide-react";
import { useMoneyballPreferences } from "@/stores/use-moneyball-preferences";
import { cn } from "@/utils/cn";

type DestinationId =
  | "dashboard"
  | "search"
  | "moneyball"
  | "staff-search"
  | "my-staff"
  | "squad"
  | "planner"
  | "tactic"
  | "youth"
  | "settings";

type Destination = {
  id: DestinationId;
  label: string;
  icon: LucideIcon;
  to: string;
  search?: Record<string, string>;
  /**
   * Same-route view transition. The plain `search` object replaces the
   * whole search state; the transition keeps `shortlistOnly` (and
   * `combine`, which the old tab patch never replaced) while every other
   * key falls back to the destination view defaults in validateSearch.
   */
  searchTransition?: (
    previous: Record<string, unknown>,
  ) => Record<string, unknown>;
};

function searchViewTransition(
  view: "general" | "moneyball",
): (previous: Record<string, unknown>) => Record<string, unknown> {
  return (previous) => ({
    view,
    combine: previous.combine,
    shortlistOnly: previous.shortlistOnly,
  });
}

function clubViewTransition(
  view: "squad" | "planner" | "tactic",
): (previous: Record<string, unknown>) => Record<string, unknown> {
  return (previous) => ({
    view,
    squadSort: previous.squadSort,
    squadDir: previous.squadDir,
  });
}

const destinations: Destination[] = [
  { id: "dashboard", label: "Dashboard", icon: LayoutDashboard, to: "/" },
  {
    id: "search",
    label: "Search",
    icon: Search,
    to: "/search",
    search: { view: "general" },
    searchTransition: searchViewTransition("general"),
  },
  {
    id: "moneyball",
    label: "Moneyball",
    icon: Target,
    to: "/search",
    search: { view: "moneyball" },
    searchTransition: searchViewTransition("moneyball"),
  },
  {
    id: "staff-search",
    label: "Staff Search",
    icon: UserRoundSearch,
    to: "/staff",
    search: { view: "search" },
  },
  {
    id: "my-staff",
    label: "My Staff",
    icon: UserRoundCog,
    to: "/staff",
    search: { view: "my-staff" },
  },
  {
    id: "squad",
    label: "Squad",
    icon: UsersRound,
    to: "/my-club",
    search: { view: "squad" },
    searchTransition: clubViewTransition("squad"),
  },
  {
    id: "planner",
    label: "Planner",
    icon: CalendarDays,
    to: "/my-club",
    search: { view: "planner" },
    searchTransition: clubViewTransition("planner"),
  },
  {
    id: "tactic",
    label: "Tactic",
    icon: ClipboardList,
    to: "/my-club",
    search: { view: "tactic" },
    searchTransition: clubViewTransition("tactic"),
  },
  { id: "youth", label: "Youth", icon: GraduationCap, to: "/academy" },
  { id: "settings", label: "Settings", icon: SettingsIcon, to: "/settings" },
];

const groups: DestinationId[][] = [
  ["dashboard"],
  ["search", "moneyball"],
  ["staff-search", "my-staff"],
  ["squad", "planner", "tactic", "youth"],
  ["settings"],
];

function currentDestinationId(
  pathname: string,
  search: Record<string, unknown>,
  defaultAnalysisView: "general" | "moneyball",
): DestinationId | null {
  if (pathname === "/") return "dashboard";
  if (pathname === "/settings") return "settings";
  if (pathname === "/search") {
    const view =
      typeof search.view === "string" ? search.view : defaultAnalysisView;
    return view === "moneyball" ? "moneyball" : "search";
  }
  if (pathname === "/staff") {
    return search.view === "my-staff" ? "my-staff" : "staff-search";
  }
  if (pathname === "/my-club") {
    if (search.view === undefined || search.view === "squad") return "squad";
    if (search.view === "planner") return "planner";
    if (search.view === "tactic") return "tactic";
    return null;
  }
  if (pathname === "/academy") return "youth";
  return null;
}

export function AppNavBar() {
  const { pathname, search } = useLocation();
  const defaultAnalysisView = useMoneyballPreferences(
    (state) => state.defaultAnalysisView,
  );
  const current = currentDestinationId(
    pathname,
    search as Record<string, unknown>,
    defaultAnalysisView,
  );
  const byId = new Map(destinations.map((item) => [item.id, item]));
  // Same-route view transitions keep the old tab contract: inside /search
  // shortlistOnly/combine survive, inside /my-club squadSort/squadDir
  // survive, and every other key falls back to the destination view
  // defaults in validateSearch. Links from other routes use the plain
  // search object so unrelated search state never leaks across routes.
  // Each transition only carries its own route's keys, so applying it on
  // the sibling route degrades to the plain object.

  return (
    <nav
      aria-label="Primary"
      data-testid="app-nav-bar"
      className="z-10 h-12 shrink-0 border-b border-outline-variant bg-surface-container"
    >
      <div className="flex h-full items-center justify-center gap-1 px-4">
        {groups.map((group, groupIndex) => (
          <div key={group[0]} className="flex items-center">
            {groupIndex > 0 ? (
              <div
                aria-hidden="true"
                data-nav-separator="true"
                className="h-8 w-px shrink-0 bg-outline-variant"
              />
            ) : null}
            <div className="flex items-center gap-2 px-2">
              <div className="flex items-center gap-1">
                {group.map((id) => {
                  const item = byId.get(id);
                  if (!item) return null;
                  const isActive = current === id;
                  return (
                    <Link
                      key={id}
                      to={item.to}
                      search={
                        item.searchTransition && pathname === item.to
                          ? item.searchTransition
                          : item.search
                      }
                      activeOptions={{ exact: true }}
                      aria-current={isActive ? "page" : undefined}
                      className={cn(
                        "flex h-9 items-center justify-center gap-1.5 rounded-md px-2 text-label-md font-normal tracking-normal",
                        "transition-colors duration-150 ease-out hover:text-on-surface focus-visible:outline-offset-[-2px]",
                        isActive
                          ? "bg-[color-mix(in_oklab,var(--color-primary)_10%,var(--color-surface-container))] text-on-surface"
                          : "text-on-surface-variant hover:bg-surface-container-high",
                      )}
                    >
                      <item.icon
                        aria-hidden="true"
                        size={16}
                        strokeWidth={isActive ? 2 : 1.5}
                        className={cn("shrink-0", isActive && "text-primary")}
                      />
                      <span className={cn(isActive && "font-bold")}>
                        {item.label}
                      </span>
                    </Link>
                  );
                })}
              </div>
            </div>
          </div>
        ))}
      </div>
    </nav>
  );
}
