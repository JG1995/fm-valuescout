import { createFileRoute } from "@tanstack/react-router";
import {
  type MyClubSearch,
  type MyClubWorkspace,
  parseMyClubWorkspace,
} from "@/app/routes/my-club";
import type {
  SquadSortDir,
  SquadSortField,
} from "@/features/squad/types/squad-sort";
import {
  defaultDirForSquadSortField,
  isSquadSortDir,
  isSquadSortField,
} from "@/features/squad/types/squad-sort";

type PlannerLegacySearch = {
  view?: MyClubWorkspace;
  sort?: SquadSortField;
  dir?: SquadSortDir;
};

function toMyClubSearch(search: {
  view?: unknown;
  sort?: unknown;
  dir?: unknown;
}): MyClubSearch {
  const view = parseMyClubWorkspace(search.view) ?? "planner";
  const sort = isSquadSortField(search.sort) ? search.sort : undefined;
  const dir = isSquadSortDir(search.dir) ? search.dir : undefined;

  if (!sort) {
    return {
      view,
      ...(dir ? { squadDir: dir } : {}),
    };
  }

  return {
    view,
    squadSort: sort,
    squadDir: dir ?? defaultDirForSquadSortField(sort),
  };
}

export const Route = createFileRoute("/planner")({
  validateSearch: (search: Record<string, unknown>): PlannerLegacySearch => {
    const view = parseMyClubWorkspace(search.view);
    const sort = isSquadSortField(search.sort) ? search.sort : undefined;
    const dir = isSquadSortDir(search.dir) ? search.dir : undefined;
    return {
      ...(view ? { view } : {}),
      ...(sort ? { sort } : {}),
      ...(dir ? { dir } : {}),
    };
  },
  beforeLoad: ({ search }) => {
    throw Route.redirect({
      to: "/my-club",
      search: toMyClubSearch(search),
      replace: true,
    });
  },
  component: () => null,
});
