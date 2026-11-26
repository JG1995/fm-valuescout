import { Lock } from "lucide-react";
import { fieldClasses } from "@/components/ui/field/field-styles";
import { Tooltip } from "@/components/ui/tooltip/tooltip";
import { cn } from "@/utils/cn";
import type {
  FmTrainingLane,
  FmTrainingPlayer,
  TrainingBasis,
} from "../types/fm-training";

/** A slot's placements as the table names them: "DC", or "AMR/MR" when the phases differ. */
export function placement(lane: { ipPosition: string; oopPosition: string }) {
  return lane.ipPosition === lane.oopPosition
    ? lane.ipPosition
    : `${lane.ipPosition}/${lane.oopPosition}`;
}

/**
 * A slot as the Develops as column names it: its placement and the role the
 * tactic has there in possession, such as "STC · Channel Forward". The slot's
 * out-of-possession role is trained with it.
 */
function slotName(option: {
  ipPosition: string;
  oopPosition: string;
  inPossessionRole: string;
}) {
  return `${placement(option)} · ${option.inPossessionRole}`;
}

/** One slot to grow into: the tactic's slots that train the same. */
export type SlotGroup = {
  id: string;
  label: string;
  laneIds: string[];
  /** The players developing towards it. */
  count: number;
};

/**
 * The tactic's slots, with those that train the same as one group, each with
 * the number of players developing towards it.
 */
export function slotGroups(
  lanes: readonly FmTrainingLane[],
  players: readonly FmTrainingPlayer[],
) {
  const groups: SlotGroup[] = [];
  for (const lane of lanes) {
    const group = groups.find((candidate) => candidate.id === lane.trainsAs);
    if (group) {
      group.laneIds.push(lane.laneId);
      group.label += `, ${placement(lane)}`;
    } else {
      groups.push({
        id: lane.trainsAs,
        label: placement(lane),
        laneIds: [lane.laneId],
        count: 0,
      });
    }
  }
  for (const player of players) {
    const group = groupOf(groups, player.development?.laneId ?? null);
    if (group) {
      group.count += 1;
    }
  }
  return groups;
}

export function groupOf(groups: readonly SlotGroup[], laneId: string | null) {
  return laneId === null
    ? undefined
    : groups.find((group) => group.laneIds.includes(laneId));
}

const BASES: readonly { basis: TrainingBasis; label: string }[] = [
  { basis: "planner", label: "Planner slot" },
  { basis: "development", label: "Development slot" },
];

// From the youngest limit a save can set to the last age with growth left.
const MAX_AGES = Array.from({ length: 14 }, (_, index) => 15 + index);

/**
 * What training follows, and how a development slot is found: up to what age
 * a player has one, and whether his positions count. The last two also decide
 * what the Planner optimizes for development, so they show whatever training
 * follows. The parts sit in the caller's control row.
 */
export function TrainForControl({
  basis,
  maxAge,
  positionFamiliarity,
  disabled,
  onBasisChange,
  onMaxAgeChange,
  onPositionFamiliarityChange,
}: {
  basis: TrainingBasis;
  maxAge: number;
  positionFamiliarity: boolean;
  disabled: boolean;
  onBasisChange: (basis: TrainingBasis) => void;
  onMaxAgeChange: (maxAge: number) => void;
  onPositionFamiliarityChange: (positionFamiliarity: boolean) => void;
}) {
  const basisHints: Record<TrainingBasis, string> = {
    planner: "Every player trains the roles of the slot he plays.",
    development: `Players aged ${maxAge} or under train the slot of your tactic where their potential ranks highest in the save. Older players train the slot they play.`,
  };
  return (
    <>
      <div className="flex items-center gap-2">
        <span className="text-label-md text-on-surface-variant">Train for</span>
        <fieldset className="inline-flex rounded-md border border-outline bg-surface-container-high p-0.5">
          <legend className="sr-only">Train for</legend>
          {BASES.map((option) => (
            <Tooltip key={option.basis} content={basisHints[option.basis]}>
              <button
                type="button"
                aria-pressed={basis === option.basis}
                disabled={disabled}
                className={
                  basis === option.basis
                    ? "h-8 rounded-sm bg-primary px-3 text-label-md text-on-primary"
                    : "h-8 rounded-sm px-3 text-label-md text-on-surface-variant hover:text-on-surface disabled:hover:text-on-surface-variant"
                }
                onClick={() => {
                  if (basis !== option.basis) {
                    onBasisChange(option.basis);
                  }
                }}
              >
                {option.label}
              </button>
            </Tooltip>
          ))}
        </fieldset>
      </div>
      <label className="flex items-center gap-2 text-label-md text-on-surface-variant">
        Develop players aged up to
        <select
          className={cn(fieldClasses, "tabular-nums")}
          value={maxAge}
          disabled={disabled}
          onChange={(event) => onMaxAgeChange(Number(event.target.value))}
        >
          {MAX_AGES.map((age) => (
            <option key={age} value={age}>
              {age}
            </option>
          ))}
        </select>
      </label>
      {/* The Planner's familiarity option, for development slots. */}
      <Tooltip
        content={
          positionFamiliarity
            ? "A development slot is suggested only among the positions a player can play."
            : "A development slot is suggested in any position, by role scores and preferred foot alone. Goalkeepers and outfield players never swap."
        }
      >
        {(tooltipId) => (
          <label
            className={cn(
              "flex items-center gap-2 text-body-md",
              disabled ? "text-on-surface-variant" : "text-on-surface",
            )}
          >
            <input
              type="checkbox"
              checked={positionFamiliarity}
              disabled={disabled}
              aria-describedby={tooltipId}
              onChange={(event) =>
                onPositionFamiliarityChange(event.target.checked)
              }
            />
            Position familiarity
          </label>
        )}
      </Tooltip>
    </>
  );
}

/** How many players are growing into each slot, and the slots nobody is growing into. */
export function DevelopmentPipeline({
  groups,
  selectedId,
  onSelect,
}: {
  groups: readonly SlotGroup[];
  selectedId: string | null;
  onSelect: (groupId: string | null) => void;
}) {
  return (
    <div className="flex flex-wrap items-center gap-x-3 gap-y-2 border-t border-outline-variant px-4 py-2">
      <span className="text-label-md text-on-surface-variant">
        Developing towards
      </span>
      <fieldset className="m-0 flex min-w-0 flex-wrap gap-1.5 border-0 p-0">
        <legend className="sr-only">Show the players developing towards</legend>
        {groups.map((group) => (
          <button
            key={group.id}
            type="button"
            aria-pressed={selectedId === group.id}
            aria-label={`${group.label}: ${group.count === 0 ? "no players" : group.count === 1 ? "1 player" : `${group.count} players`}`}
            className={cn(
              "inline-flex h-7 items-center gap-1.5 rounded-sm border px-2 font-mono text-mono-sm",
              group.count === 0
                ? "border-warning/40 bg-warning-container text-on-warning-container"
                : "border-outline-variant text-on-surface hover:bg-surface-container-highest",
              selectedId === group.id &&
                "outline-2 outline-offset-1 outline-primary",
            )}
            onClick={() => onSelect(selectedId === group.id ? null : group.id)}
          >
            {group.label}
            <span
              className={cn(
                "tabular-nums",
                group.count === 0 ? null : "text-primary",
              )}
            >
              {group.count}
            </span>
          </button>
        ))}
      </fieldset>
    </div>
  );
}

function Scores({
  current,
  potential,
}: {
  current: number;
  potential: number;
}) {
  return (
    <span
      className="font-mono text-mono-sm whitespace-nowrap tabular-nums"
      title="His score in the slot now, and at his potential"
    >
      {current}
      <span aria-hidden="true" className="px-1 text-on-surface-variant">
        →
      </span>
      <span className="sr-only"> now, </span>
      {potential}
      <span className="sr-only"> at potential</span>
    </span>
  );
}

function Detail({ children }: { children: React.ReactNode }) {
  return (
    <span className="block pt-0.5 text-[11px] text-on-surface-variant">
      {children}
    </span>
  );
}

/**
 * The slot a player is growing into, with his score there now and at his
 * potential. While training follows development slots, it is a choice.
 */
export function DevelopsAsCell({
  player,
  basis,
  maxAge,
  sameAsSlot,
  disabled,
  onChoose,
}: {
  player: FmTrainingPlayer;
  basis: TrainingBasis;
  maxAge: number;
  /** His development slot trains as the slot the Planner has him in. */
  sameAsSlot: boolean;
  disabled: boolean;
  /** A null lane returns him to the suggested slot. */
  onChoose: (laneId: string | null) => void;
}) {
  const { development } = player;
  if (development === null) {
    return (
      <span className="text-on-surface-variant">
        {player.age === null ? "Age unknown" : `Over ${maxAge}`}
      </span>
    );
  }
  if (development.options.length === 0) {
    return <span className="text-on-surface-variant">Suits no slot</span>;
  }

  const chosen = development.options.find(
    (option) => option.laneId === development.laneId,
  );
  const suggested = development.options.find(
    (option) => option.laneId === development.suggestedLaneId,
  );
  if (basis === "planner") {
    return (
      <span className="flex items-center gap-2 text-on-surface-variant">
        <span>{chosen ? slotName(chosen) : "No slot"}</span>
        {chosen ? (
          <Scores
            current={chosen.currentScore}
            potential={chosen.potentialScore}
          />
        ) : null}
      </span>
    );
  }

  const playable = development.options.filter((option) => option.playable);
  const learnable = development.options.filter((option) => !option.playable);
  const optionLabel = (option: (typeof development.options)[number]) =>
    option.laneId === development.suggestedLaneId
      ? `${slotName(option)} · suggested`
      : slotName(option);
  return (
    <>
      <span className="flex items-center gap-2">
        <select
          aria-label={`Development slot for ${player.name}`}
          // Smaller than a form field, to sit in a table row.
          // The list shows each name in full; the closed box may cut the
          // suggested mark off a long one, which the line below still says.
          className="h-7 max-w-56 rounded-sm border border-outline bg-surface-container-high px-1.5 text-body-sm text-on-surface hover:border-on-surface-variant focus-visible:outline-offset-0 disabled:cursor-not-allowed disabled:opacity-45"
          value={development.laneId ?? ""}
          disabled={disabled}
          onChange={(event) =>
            onChoose(
              event.target.value === development.suggestedLaneId
                ? null
                : event.target.value,
            )
          }
        >
          {chosen ? null : (
            <option value="" disabled>
              No slot
            </option>
          )}
          {playable.length > 0 ? (
            <optgroup label="Can play">
              {playable.map((option) => (
                <option key={option.laneId} value={option.laneId}>
                  {optionLabel(option)}
                </option>
              ))}
            </optgroup>
          ) : null}
          {learnable.length > 0 ? (
            <optgroup label="To learn">
              {learnable.map((option) => (
                <option key={option.laneId} value={option.laneId}>
                  {optionLabel(option)}
                </option>
              ))}
            </optgroup>
          ) : null}
        </select>
        {chosen ? (
          <Scores
            current={chosen.currentScore}
            potential={chosen.potentialScore}
          />
        ) : null}
      </span>
      <Detail>
        {development.locked ? (
          <span className="inline-flex flex-wrap items-center gap-x-1.5">
            <span className="inline-flex items-center gap-1 text-on-surface">
              <Lock aria-hidden="true" className="size-3" />
              Your choice
            </span>
            <button
              type="button"
              disabled={disabled}
              className="underline underline-offset-2 hover:text-on-surface"
              onClick={() => onChoose(null)}
            >
              {suggested
                ? `Use suggested ${placement(suggested)}`
                : "Clear the choice"}
            </button>
          </span>
        ) : chosen ? (
          [
            chosen.potentialTopPercent === null
              ? null
              : `Top ${chosen.potentialTopPercent}% at potential`,
            chosen.playable ? null : "a position to learn",
            sameAsSlot ? "same as his slot" : null,
          ]
            .filter((part) => part !== null)
            .join(" · ")
            .replace(/^a /, "A ")
        ) : (
          "No playable slot yet: choose one to learn"
        )}
      </Detail>
    </>
  );
}
