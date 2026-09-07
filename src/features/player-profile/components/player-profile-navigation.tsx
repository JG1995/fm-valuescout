import { type KeyboardEvent, useEffect } from "react";

export type PlayerProfileView = "general" | "moneyball";

const PLAYER_PROFILE_VIEWS = ["general", "moneyball"] as const;

export function parsePlayerProfileView(
  value: unknown,
): PlayerProfileView | undefined {
  if (value === "moneyball" || value === "general") return value;
  return undefined;
}

export function PlayerAnalysisTabs({
  view,
  onViewChange,
  restoreFocus,
  onFocusRestored,
}: {
  view: PlayerProfileView;
  onViewChange: (view: PlayerProfileView, restoreFocus?: boolean) => void;
  restoreFocus: boolean;
  onFocusRestored: () => void;
}) {
  useEffect(() => {
    if (!restoreFocus) return;
    document.getElementById(`player-analysis-tab-${view}`)?.focus();
    onFocusRestored();
  }, [onFocusRestored, restoreFocus, view]);

  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    const index = PLAYER_PROFILE_VIEWS.indexOf(view);
    let nextIndex = index;
    if (event.key === "ArrowRight" || event.key === "ArrowDown") {
      nextIndex = (index + 1) % PLAYER_PROFILE_VIEWS.length;
    } else if (event.key === "ArrowLeft" || event.key === "ArrowUp") {
      nextIndex =
        (index - 1 + PLAYER_PROFILE_VIEWS.length) % PLAYER_PROFILE_VIEWS.length;
    } else if (event.key === "Home") {
      nextIndex = 0;
    } else if (event.key === "End") {
      nextIndex = PLAYER_PROFILE_VIEWS.length - 1;
    } else {
      return;
    }

    event.preventDefault();
    const next = PLAYER_PROFILE_VIEWS[nextIndex];
    onViewChange(next, true);
  };

  return (
    <div
      role="tablist"
      aria-label="Player analysis view"
      className="inline-flex rounded-full bg-surface-container-high p-0.5"
      onKeyDown={onKeyDown}
    >
      {PLAYER_PROFILE_VIEWS.map((candidate) => {
        const selected = candidate === view;
        return (
          <button
            key={candidate}
            id={`player-analysis-tab-${candidate}`}
            type="button"
            role="tab"
            aria-selected={selected}
            aria-controls="player-analysis-panel"
            tabIndex={selected ? 0 : -1}
            className={
              selected
                ? "cursor-pointer rounded-full bg-primary px-3 py-1.5 text-label-md text-on-primary"
                : "cursor-pointer rounded-full px-3 py-1.5 text-label-md text-on-surface-variant hover:text-on-surface"
            }
            onClick={() => onViewChange(candidate, true)}
          >
            {candidate === "general" ? "General" : "Moneyball"}
          </button>
        );
      })}
    </div>
  );
}
