import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import { formatMissable, formatMoney, formatPlayerDob } from "@/utils/format";

export const TABLE_TEXT_CELL_CLASS =
  "h-table-row-height-two-line max-w-0 truncate px-2 align-middle text-body-sm";
export const TABLE_NUMERIC_CELL_CLASS =
  "h-table-row-height-two-line whitespace-nowrap px-2 align-middle text-right font-mono text-mono-sm text-on-surface tabular-nums";

type DynamicRow = {
  dynamicValues?: Record<string, unknown> | undefined;
};

/** Shared unavailable/loading text: `…` while the row is absent, `—` when the value is absent. */
export function formatTableDynamicCell(
  row: DynamicRow | undefined,
  fieldId: string,
): string {
  if (!row) {
    return "…";
  }
  const value = row.dynamicValues?.[fieldId];
  if (value === undefined || value === null) {
    return "—";
  }
  return String(value);
}

export type PlayerBasicCellKey =
  | "name"
  | "age"
  | "nationality"
  | "club"
  | "division"
  | "ca"
  | "pa"
  | "value";

/** Narrowest player shape shared by Search and Squad basic cells. */
export type PlayerBasicRow = {
  name: string;
  age: number | null;
  birthYear: number;
  birthDayOfYear: number;
  nationalities: string[];
  club: string | null;
  division: string | null;
  ca: number;
  pa: number;
  marketValueGbp: number | null;
};

/** Shared player basic-cell presentation: loading `…`, missing `—`, value via formatMoney. */
export function formatPlayerBasicCell(
  player: PlayerBasicRow | undefined,
  key: PlayerBasicCellKey,
): { text: string; title?: string; numeric: boolean } {
  if (!player) {
    return { text: "…", numeric: key !== "name" && key !== "age" };
  }
  switch (key) {
    case "name":
      return { text: player.name, title: player.name, numeric: false };
    case "age": {
      const dob = formatPlayerDob(
        player.birthYear,
        player.birthDayOfYear,
        player.age,
      );
      return { text: dob, title: dob, numeric: false };
    }
    case "nationality": {
      const nationalities = String(
        formatMissable(player.nationalities.join(", ")),
      );
      return { text: nationalities, title: nationalities, numeric: false };
    }
    case "club": {
      const club = String(formatMissable(player.club));
      return {
        text: club,
        title: club !== "—" ? club : undefined,
        numeric: false,
      };
    }
    case "division": {
      const division = String(formatMissable(player.division));
      return {
        text: division,
        title: division !== "—" ? division : undefined,
        numeric: false,
      };
    }
    case "ca":
      return { text: String(player.ca), numeric: true };
    case "pa":
      return { text: String(player.pa), numeric: true };
    case "value":
      return {
        text:
          player.marketValueGbp === null
            ? "—"
            : formatMoney(player.marketValueGbp),
        numeric: true,
      };
  }
}

type TableScoreContentProps = {
  score: unknown;
  roleName: string;
  isLoading?: boolean;
};

/** Shared score presentation: table ScoreBadge for numbers, `…` while loading, `—` when missing. */
export function TableScoreContent({
  score,
  roleName,
  isLoading = false,
}: TableScoreContentProps) {
  if (isLoading) {
    return <span className="text-on-surface-variant">…</span>;
  }
  if (typeof score === "number") {
    return <ScoreBadge score={score} roleName={roleName} />;
  }
  return <span className="text-on-surface-variant">—</span>;
}
