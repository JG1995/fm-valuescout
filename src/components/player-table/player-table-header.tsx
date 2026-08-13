import { ChevronDown, ChevronUp, Ellipsis, Plus, Trash2 } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { PlayerMetricPicker } from "@/components/ui/player-metric-picker";
import {
  PLAYER_METRICS,
  PLAYER_TABLE_MAX_COLUMN_WIDTH,
  PLAYER_TABLE_MIN_COLUMN_WIDTH,
  type PlayerMetricAlignment,
} from "@/utils/player-metrics";

const KEYBOARD_RESIZE_STEP = 16;

export type PlayerTableColumn = {
  id: string;
  label: string;
  align: PlayerMetricAlignment;
  width: number;
};

type PlayerTableHeaderProps = {
  columns: readonly PlayerTableColumn[];
  sortBy: string;
  sortDir: "asc" | "desc";
  onSortChange: (metricId: string) => void;
  onAddColumn: (metricId: string) => void;
  onRemoveColumn: (metricId: string) => void;
  onResizeColumn: (metricId: string, width: number) => void;
};

function ColumnResizeHandle({
  label,
  width,
  onResize,
}: {
  label: string;
  width: number;
  onResize: (width: number) => void;
}) {
  const handleRef = useRef<HTMLHRElement>(null);
  const activePointerRef = useRef<{
    id: number;
    startX: number;
    startWidth: number;
  } | null>(null);
  const releasePointer = () => {
    const activePointer = activePointerRef.current;
    const handle = handleRef.current;
    if (activePointer && handle?.hasPointerCapture?.(activePointer.id)) {
      handle.releasePointerCapture(activePointer.id);
    }
    activePointerRef.current = null;
  };

  useEffect(
    () => () => {
      const activePointer = activePointerRef.current;
      const handle = handleRef.current;
      if (activePointer && handle?.hasPointerCapture?.(activePointer.id)) {
        handle.releasePointerCapture(activePointer.id);
      }
    },
    [],
  );

  return (
    <hr
      ref={handleRef}
      aria-label={`Resize ${label} column`}
      aria-orientation="vertical"
      aria-valuemin={PLAYER_TABLE_MIN_COLUMN_WIDTH}
      aria-valuemax={PLAYER_TABLE_MAX_COLUMN_WIDTH}
      aria-valuenow={width}
      aria-valuetext={`${width} pixels`}
      tabIndex={0}
      className="absolute inset-y-0 right-0 z-20 w-2 cursor-col-resize touch-none rounded-sm border-0 hover:bg-primary/40 focus-visible:bg-primary focus-visible:outline-2 focus-visible:outline-offset-[-2px] focus-visible:outline-primary"
      onPointerDown={(event) => {
        event.preventDefault();
        event.stopPropagation();
        activePointerRef.current = {
          id: event.pointerId,
          startX: event.clientX,
          startWidth: width,
        };
        event.currentTarget.setPointerCapture?.(event.pointerId);
      }}
      onPointerMove={(event) => {
        const activePointer = activePointerRef.current;
        if (!activePointer || activePointer.id !== event.pointerId) {
          return;
        }
        onResize(
          activePointer.startWidth + event.clientX - activePointer.startX,
        );
      }}
      onPointerUp={releasePointer}
      onPointerCancel={releasePointer}
      onLostPointerCapture={() => {
        activePointerRef.current = null;
      }}
      onKeyDown={(event) => {
        switch (event.key) {
          case "ArrowLeft":
            event.preventDefault();
            onResize(width - KEYBOARD_RESIZE_STEP);
            break;
          case "ArrowRight":
            event.preventDefault();
            onResize(width + KEYBOARD_RESIZE_STEP);
            break;
          case "Home":
            event.preventDefault();
            onResize(PLAYER_TABLE_MIN_COLUMN_WIDTH);
            break;
          case "End":
            event.preventDefault();
            onResize(PLAYER_TABLE_MAX_COLUMN_WIDTH);
            break;
        }
      }}
    />
  );
}

export function PlayerTableHeader({
  columns,
  sortBy,
  sortDir,
  onSortChange,
  onAddColumn,
  onRemoveColumn,
  onResizeColumn,
}: PlayerTableHeaderProps) {
  const [openColumnId, setOpenColumnId] = useState<string | null>(null);
  const [pickingColumnId, setPickingColumnId] = useState<string | null>(null);
  const triggerRefs = useRef(new Map<string, HTMLButtonElement>());
  const availableMetrics = useMemo(
    () =>
      PLAYER_METRICS.filter(
        (metric) =>
          metric.sortable && !columns.some((column) => column.id === metric.id),
      ),
    [columns],
  );

  const closeMenu = () => {
    const columnId = openColumnId;
    setOpenColumnId(null);
    setPickingColumnId(null);
    if (columnId) {
      triggerRefs.current.get(columnId)?.focus();
    }
  };

  return (
    <thead className="sticky top-0 z-10">
      <tr className="bg-surface-container-lowest">
        {columns.map((column) => {
          const active = column.id === sortBy;
          const open = openColumnId === column.id;
          const picking = pickingColumnId === column.id;
          const ariaSort = active
            ? sortDir === "asc"
              ? "ascending"
              : "descending"
            : "none";
          const Caret = sortDir === "asc" ? ChevronUp : ChevronDown;

          return (
            <th
              key={column.id}
              scope="col"
              aria-label={column.label}
              aria-sort={ariaSort}
              className={`relative h-table-header-height px-2 ${
                column.align === "right" ? "text-right" : "text-left"
              }`}
              onContextMenu={(event) => {
                event.preventDefault();
                setOpenColumnId(column.id);
                setPickingColumnId(null);
              }}
            >
              <div className="flex min-w-0 items-center justify-between gap-1 pr-1">
                <button
                  type="button"
                  className={`inline-flex min-w-0 items-center gap-1 truncate text-label-md uppercase ${
                    active ? "text-primary" : "text-on-surface-variant"
                  }`}
                  onClick={() => onSortChange(column.id)}
                >
                  <span className="truncate">{column.label}</span>
                  {active ? (
                    <Caret
                      aria-hidden
                      className="size-3.5 shrink-0"
                      strokeWidth={2}
                    />
                  ) : null}
                </button>
                <button
                  ref={(element) => {
                    if (element) {
                      triggerRefs.current.set(column.id, element);
                    } else {
                      triggerRefs.current.delete(column.id);
                    }
                  }}
                  type="button"
                  aria-label={`Manage ${column.label} column`}
                  aria-expanded={open}
                  aria-haspopup="menu"
                  className="inline-flex size-6 shrink-0 items-center justify-center rounded-sm text-on-surface-variant transition-colors duration-150 ease-out hover:bg-surface-container-high hover:text-on-surface focus-visible:outline-2 focus-visible:outline-offset-[-2px] focus-visible:outline-primary"
                  onClick={() => {
                    if (open) {
                      closeMenu();
                    } else {
                      setOpenColumnId(column.id);
                      setPickingColumnId(null);
                    }
                  }}
                  onKeyDown={(event) => {
                    if (open && event.key === "Escape") {
                      event.preventDefault();
                      closeMenu();
                    }
                  }}
                >
                  <Ellipsis aria-hidden size={16} strokeWidth={1.5} />
                </button>
              </div>

              {open && !picking ? (
                <div
                  role="menu"
                  aria-label={`${column.label} column actions`}
                  className="absolute right-1 top-full z-30 mt-1 w-44 rounded-md border border-outline-variant bg-surface-container-highest p-1 text-left shadow-overlay"
                  onKeyDown={(event) => {
                    if (event.key === "Escape") {
                      event.preventDefault();
                      closeMenu();
                    }
                  }}
                >
                  <button
                    type="button"
                    role="menuitem"
                    disabled={availableMetrics.length === 0}
                    className="flex w-full items-center gap-2 rounded-sm px-3 py-2 text-left text-label-md text-on-surface hover:bg-surface-container-high focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary disabled:cursor-not-allowed disabled:opacity-45"
                    onClick={() => setPickingColumnId(column.id)}
                  >
                    <Plus aria-hidden size={16} strokeWidth={1.5} />
                    Add column
                  </button>
                  <button
                    type="button"
                    role="menuitem"
                    disabled={columns.length === 1}
                    className="flex w-full items-center gap-2 rounded-sm px-3 py-2 text-left text-label-md text-error hover:bg-surface-container-high focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary disabled:cursor-not-allowed disabled:opacity-45"
                    onClick={() => {
                      onRemoveColumn(column.id);
                      closeMenu();
                    }}
                  >
                    <Trash2 aria-hidden size={16} strokeWidth={1.5} />
                    Remove {column.label}
                  </button>
                </div>
              ) : null}

              {open && picking ? (
                <div
                  role="dialog"
                  aria-label="Add a column"
                  className="absolute right-1 top-full z-30 mt-1 w-72 rounded-md border border-outline-variant bg-surface-container-highest p-3 text-left shadow-overlay"
                  onKeyDown={(event) => {
                    if (event.key === "Escape") {
                      event.preventDefault();
                      closeMenu();
                    }
                  }}
                >
                  <PlayerMetricPicker
                    label="Column"
                    metrics={availableMetrics}
                    value=""
                    onChange={(metricId) => {
                      onAddColumn(metricId);
                      closeMenu();
                    }}
                  />
                </div>
              ) : null}

              <ColumnResizeHandle
                label={column.label}
                width={column.width}
                onResize={(width) => onResizeColumn(column.id, width)}
              />
            </th>
          );
        })}
      </tr>
    </thead>
  );
}
