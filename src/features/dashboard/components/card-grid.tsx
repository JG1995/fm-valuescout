import { type ReactNode, useLayoutEffect, useRef, useState } from "react";
import { cn } from "@/utils/cn";
import {
  type CardCapacity,
  type CardSize,
  cardCapacity,
} from "../utils/card-capacity";

// Shown until the grid is measured, and where it cannot be.
const UNMEASURED: CardCapacity = { columns: 3, rows: 1 };

/**
 * The best-first cards that fit the space the grid is given: more columns on a
 * wider window and a second row on a taller one. Cards share the space evenly
 * behind 1px rules, and the grid never scrolls. `className` sets the least
 * height the grid accepts, which should hold one card.
 */
export function CardGrid<Item>({
  label,
  items,
  min,
  maxColumns,
  className,
  itemKey,
  children,
}: {
  /** Names the list, such as "Recommended signings, best first". */
  label: string;
  /** Every candidate, best first. */
  items: readonly Item[];
  /** The smallest a card may be, in pixels. Pass a stable object. */
  min: CardSize;
  /** The widest the grid grows, so its rows fill evenly with the longest list. */
  maxColumns: number;
  className: string;
  itemKey: (item: Item) => string | number;
  /** Renders one card as an `li`. */
  children: (item: Item) => ReactNode;
}) {
  const ref = useRef<HTMLOListElement>(null);
  const [capacity, setCapacity] = useState(UNMEASURED);

  useLayoutEffect(() => {
    const element = ref.current;
    if (!element || typeof ResizeObserver === "undefined") return;
    const measure = () => {
      const box = { width: element.clientWidth, height: element.clientHeight };
      if (box.width === 0 || box.height === 0) return;
      const next = cardCapacity(box, min);
      setCapacity((current) =>
        current.columns === next.columns && current.rows === next.rows
          ? current
          : next,
      );
    };
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(element);
    return () => observer.disconnect();
  }, [min]);

  const columns = Math.min(capacity.columns, maxColumns);
  const shown = items.slice(0, columns * capacity.rows);
  const rows = Math.max(1, Math.ceil(shown.length / columns));
  // Empty cells keep a short list at the same card width as a full one.
  const emptyCells = rows * columns - shown.length;
  return (
    <ol
      ref={ref}
      aria-label={label}
      className={cn(
        "grid flex-1 gap-px overflow-hidden bg-outline-variant",
        className,
      )}
      style={{
        gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))`,
        gridTemplateRows: `repeat(${rows}, minmax(0, 1fr))`,
      }}
    >
      {shown.map((item) => (
        <li
          key={itemKey(item)}
          className="flex min-h-0 min-w-0 overflow-hidden bg-surface-container [container-type:size]"
        >
          {children(item)}
        </li>
      ))}
      {Array.from({ length: emptyCells }, (_, index) => (
        <li
          // biome-ignore lint/suspicious/noArrayIndexKey: identical fillers
          key={`empty-${index}`}
          aria-hidden="true"
          className="bg-surface-container"
        />
      ))}
    </ol>
  );
}
