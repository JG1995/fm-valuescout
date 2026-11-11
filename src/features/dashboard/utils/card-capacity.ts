export type CardSize = { width: number; height: number };

export type CardCapacity = { columns: number; rows: number };

/**
 * How many cards of at least `min` fit in `box` as columns and rows separated
 * by 1px rules. A box smaller than one card still holds one.
 */
export function cardCapacity(box: CardSize, min: CardSize): CardCapacity {
  return {
    columns: Math.max(1, Math.floor((box.width + 1) / (min.width + 1))),
    rows: Math.max(1, Math.floor((box.height + 1) / (min.height + 1))),
  };
}
