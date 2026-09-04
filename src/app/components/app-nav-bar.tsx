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
  },
  {
    id: "planner",
    label: "Planner",
    icon: CalendarDays,
    to: "/my-club",
    search: { view: "planner" },
  },
  {
    id: "tactic",
    label: "Tactic",
    icon: ClipboardList,
    to: "/my-club",
    search: { view: "tactic" },
  },
  { id: "youth", label: "Youth", icon: GraduationCap, to: "/academy" },
  { id: "settings", label: "Settings", icon: SettingsIcon, to: "/settings" },
];

const groups: { caption: string; ids: DestinationId[] }[] = [
  { caption: "Home", ids: ["dashboard"] },
  { caption: "Players", ids: ["search", "moneyball"] },
  { caption: "Staff", ids: ["staff-search", "my-staff"] },
  { caption: "Club", ids: ["squad", "planner", "tactic", "youth"] },
  { caption: "Settings", ids: ["settings"] },
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
    // Interim: Commit 4 owns reversing the retained /staff?view=my-staff
    // replace-redirect to /my-club?view=staff, so map that route state to
    // the My Staff destination until the canonical Staff URL lands directly.
    if (search.view === "staff") return "my-staff";
    return null;
  }
  if (pathname === "/academy") return "youth";
  return null;
}

function currentGroupCaption(pathname: string): string | null {
  if (/^\/players\/[^/]+\/?$/.test(pathname)) return "Players";
  if (/^\/staff\/[^/]+\/?$/.test(pathname)) return "Staff";
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
  const groupContext = current === null ? currentGroupCaption(pathname) : null;
  const byId = new Map(destinations.map((item) => [item.id, item]));
  // View switching inside /search keeps the old tab transition contract
  // (shortlistOnly/combine survive, everything else resets to the
  // destination view defaults). Links from other routes use the plain
  // search object so staff or club state never leaks into Search.
  const onSearchRoute = pathname === "/search";

  return (
    <nav
      aria-label="Primary"
      data-testid="app-nav-bar"
      className="z-10 shrink-0 border-b border-outline-variant bg-surface-container"
    >
      <div className="flex items-stretch gap-1 px-4">
        {groups.map((group, groupIndex) => (
          <div key={group.caption} className="flex items-stretch">
            {groupIndex > 0 ? (
              <div
                aria-hidden="true"
                data-nav-separator="true"
                className="my-2 w-px shrink-0 bg-outline-variant"
              />
            ) : null}
            <div className="flex flex-col justify-center gap-0.5 px-2 py-1.5">
              <div className="flex items-center gap-1">
                {group.ids.map((id) => {
                  const item = byId.get(id);
                  if (!item) return null;
                  const isActive = current === id;
                  return (
                    <Link
                      key={id}
                      to={item.to}
                      search={
                        item.searchTransition && onSearchRoute
                          ? item.searchTransition
                          : item.search
                      }
                      activeOptions={{ exact: true }}
                      aria-current={isActive ? "page" : undefined}
                      className={cn(
                        "flex h-8 items-center gap-1.5 rounded-md px-2 text-label-md text-on-surface-variant",
                        "transition-colors duration-150 ease-out hover:bg-surface-container-high hover:text-on-surface",
                        isActive && "bg-primary-container text-primary",
                      )}
                    >
                      <item.icon
                        aria-hidden="true"
                        size={16}
                        strokeWidth={isActive ? 2 : 1.5}
                        className="shrink-0"
                      />
                      <span className={cn(isActive && "font-bold")}>
                        {item.label}
                      </span>
                    </Link>
                  );
                })}
              </div>
              <span
                data-nav-caption={group.caption}
                aria-current={
                  groupContext === group.caption ? "location" : undefined
                }
                className="px-2 text-label-sm text-on-surface-variant"
              >
                {group.caption}
              </span>
            </div>
          </div>
        ))}
      </div>
    </nav>
  );
}
