/**
 * Owner of the Squad-only Suggested Training column ID.
 *
 * The label, metric definition, and presentation list live in
 * Squad-owned `src/features/squad/utils/squad-columns.ts`; this neutral
 * module carries only the ID literal and its predicate so the global
 * player-table store and Squad can share one owner without a
 * shared-to-feature import.
 */
export const SUGGESTED_TRAINING_COLUMN_ID = "suggested_training";

export function isSuggestedTrainingColumnId(
  value: unknown,
): value is typeof SUGGESTED_TRAINING_COLUMN_ID {
  return value === SUGGESTED_TRAINING_COLUMN_ID;
}
