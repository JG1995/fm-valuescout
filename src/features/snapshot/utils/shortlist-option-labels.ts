import type { PlayerShortlistSummary } from "../types/save";

/**
 * Dropdown labels: the list name, or the file name when the name is empty.
 * When another shortlist shares the name, the file name is added, or "FM save"
 * for a list read from the linked save.
 */
export function shortlistOptionLabels(
  shortlists: readonly PlayerShortlistSummary[],
) {
  return shortlists.map((shortlist) => {
    const shared = shortlists.some(
      (other) => other.id !== shortlist.id && other.name === shortlist.name,
    );
    const source = shortlist.fileName ?? "FM save";
    return {
      id: shortlist.id,
      label:
        shortlist.name === ""
          ? source
          : shared
            ? `${shortlist.name} (${source})`
            : shortlist.name,
    };
  });
}
