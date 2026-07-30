import { useQueries, useSuspenseQuery } from "@tanstack/react-query";
import { useVirtualizer } from "@tanstack/react-virtual";
import { SearchX } from "lucide-react";
import { useRef } from "react";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import {
  formatCount,
  formatMissable,
  formatMoney,
  formatPlayerDob,
} from "@/utils/format";
import {
  SEARCH_PAGE_SIZE,
  searchPlayersQueryOptions,
} from "../api/search-players-query-options";
import type { PlayerSummary } from "../types/player-summary";

/** Must match `--spacing-table-row-height-two-line` / `h-table-row-height-two-line`. */
const ROW_HEIGHT = 40;
const COLUMN_COUNT = 8;
const TEXT_CELL =
  "h-table-row-height-two-line max-w-0 truncate px-2 align-middle text-body-sm";
const NUM_CELL =
  "h-table-row-height-two-line whitespace-nowrap px-2 align-middle text-right font-mono text-mono-sm text-on-surface tabular-nums";

const COLUMNS = [
  { key: "name", label: "Name", align: "left" as const },
  { key: "age", label: "Age / DOB", align: "left" as const },
  { key: "nationality", label: "Nationality", align: "left" as const },
  { key: "club", label: "Club", align: "left" as const },
  { key: "division", label: "Division", align: "left" as const },
  { key: "ca", label: "CA", align: "right" as const },
  { key: "pa", label: "PA", align: "right" as const },
  { key: "value", label: "Value", align: "right" as const },
] as const;

function pageIndexesForRange(startIndex: number, endIndex: number): number[] {
  const startPage = Math.floor(startIndex / SEARCH_PAGE_SIZE);
  const endPage = Math.floor(endIndex / SEARCH_PAGE_SIZE);
  const pages: number[] = [];
  for (let page = startPage; page <= endPage; page += 1) {
    pages.push(page);
  }
  return pages;
}

function playerAtIndex(
  pages: Array<{ page: number; players: PlayerSummary[] | undefined }>,
  index: number,
): PlayerSummary | undefined {
  const page = Math.floor(index / SEARCH_PAGE_SIZE);
  const entry = pages.find((item) => item.page === page);
  return entry?.players?.[index % SEARCH_PAGE_SIZE];
}

function SearchResultsVirtualTable({ total }: { total: number }) {
  const parentRef = useRef<HTMLDivElement>(null);
  const virtualizer = useVirtualizer({
    count: total,
    getScrollElement: () => parentRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 8,
    // jsdom reports 0×0; fall back so tests and first paint still open a window.
    initialRect: { width: 1200, height: 600 },
    observeElementRect: (instance, cb) => {
      const measure = () => {
        const el = instance.scrollElement;
        if (!el) {
          cb({ width: 1200, height: 600 });
          return;
        }
        const height = el.clientHeight;
        const width = el.clientWidth;
        cb({
          width: width > 0 ? width : 1200,
          height: height > 0 ? height : 600,
        });
      };
      measure();
      const el = instance.scrollElement;
      if (!el || typeof ResizeObserver === "undefined") {
        return () => {};
      }
      const observer = new ResizeObserver(measure);
      observer.observe(el);
      return () => {
        observer.disconnect();
      };
    },
  });

  const virtualRows = virtualizer.getVirtualItems();
  const rangeStart = virtualRows[0]?.index ?? 0;
  const rangeEnd = virtualRows[virtualRows.length - 1]?.index ?? 0;
  const pages = pageIndexesForRange(rangeStart, rangeEnd);

  const pageQueries = useQueries({
    queries: pages.map((page) =>
      searchPlayersQueryOptions(page * SEARCH_PAGE_SIZE, SEARCH_PAGE_SIZE),
    ),
  });

  const pageData = pages.map((page, index) => ({
    page,
    players: pageQueries[index]?.data?.players,
  }));

  const paddingTop = virtualRows.length > 0 ? virtualRows[0].start : 0;
  const paddingBottom =
    virtualRows.length > 0
      ? virtualizer.getTotalSize() - virtualRows[virtualRows.length - 1].end
      : 0;

  return (
    <div
      ref={parentRef}
      data-testid="search-results-scroller"
      className="max-h-[min(70vh,720px)] overflow-auto"
    >
      <table className="w-full border-collapse text-left">
        <caption className="sr-only">Player search results</caption>
        <thead className="sticky top-0 z-10">
          <tr className="bg-surface-container-lowest">
            {COLUMNS.map((column) => (
              <th
                key={column.key}
                scope="col"
                className={
                  column.align === "right"
                    ? "h-table-header-height px-2 text-right text-label-md text-on-surface-variant uppercase"
                    : "h-table-header-height px-2 text-label-md text-on-surface-variant uppercase"
                }
              >
                {column.label}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {paddingTop > 0 ? (
            <tr>
              <td colSpan={COLUMN_COUNT} style={{ height: paddingTop }} />
            </tr>
          ) : null}
          {virtualRows.map((virtualRow) => {
            const player = playerAtIndex(pageData, virtualRow.index);
            const dob = player
              ? formatPlayerDob(
                  player.birthYear,
                  player.birthDayOfYear,
                  player.age,
                )
              : "…";
            const nationalities = player
              ? String(formatMissable(player.nationalities.join(", ")))
              : "…";
            const club = player ? String(formatMissable(player.club)) : "…";
            const division = player
              ? String(formatMissable(player.division))
              : "…";
            const value = player
              ? player.marketValueGbp === null
                ? "—"
                : formatMoney(player.marketValueGbp)
              : "…";

            return (
              <tr
                key={virtualRow.key}
                data-index={virtualRow.index}
                className="border-t border-outline-variant transition-colors duration-150 ease-out hover:bg-surface-container-high"
                style={{ height: `${virtualRow.size}px` }}
              >
                <td
                  className={`${TEXT_CELL} text-on-surface`}
                  title={player?.name}
                >
                  {player?.name ?? "…"}
                </td>
                <td
                  className={`${TEXT_CELL} text-on-surface-variant`}
                  title={player ? dob : undefined}
                >
                  {dob}
                </td>
                <td
                  className={`${TEXT_CELL} text-on-surface`}
                  title={player ? nationalities : undefined}
                >
                  {nationalities}
                </td>
                <td
                  className={`${TEXT_CELL} text-on-surface`}
                  title={player && club !== "—" ? club : undefined}
                >
                  {club}
                </td>
                <td
                  className={`${TEXT_CELL} text-on-surface-variant`}
                  title={player && division !== "—" ? division : undefined}
                >
                  {division}
                </td>
                <td className={NUM_CELL}>{player?.ca ?? "…"}</td>
                <td className={NUM_CELL}>{player?.pa ?? "…"}</td>
                <td className={NUM_CELL}>{value}</td>
              </tr>
            );
          })}
          {paddingBottom > 0 ? (
            <tr>
              <td colSpan={COLUMN_COUNT} style={{ height: paddingBottom }} />
            </tr>
          ) : null}
        </tbody>
      </table>
    </div>
  );
}

/** Assumes a current snapshot exists — the route handles the no-snapshot empty. */
export function SearchResultsPanel() {
  const { data: page } = useSuspenseQuery(searchPlayersQueryOptions(0));

  if (page.total === 0) {
    return (
      <Panel title="Results" flush>
        <EmptyState icon={SearchX} title="No players in snapshot">
          The snapshot exists but holds no player rows. Run Load Data again with
          Football Manager in an active save.
        </EmptyState>
      </Panel>
    );
  }

  return (
    <Panel title="Results" flush>
      <p className="px-4 pb-3 text-body-md text-on-surface-variant">
        <span className="text-on-surface">{formatCount(page.total)}</span>{" "}
        players · sorted by CA
      </p>
      <SearchResultsVirtualTable total={page.total} />
    </Panel>
  );
}
