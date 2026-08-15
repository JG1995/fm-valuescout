import { Panel } from "@/components/ui/panel/panel";
import { formatMissable } from "@/utils/format";
import type { PlayerDetail } from "../types/player-detail";
import {
  type AttributeRow,
  attributeRows,
  attributeTierLabel,
  attributeValueTier,
  HIDDEN_ATTRIBUTE_KEYS,
  PERSONALITY_ATTRIBUTE_KEYS,
  VISIBLE_ATTRIBUTE_GROUPS,
} from "../utils/attribute-groups";
import { PROFILE_TABS, type ProfileTab } from "../utils/profile-tab";
import { PlayerProfileTabs, profileTabPanelProps } from "./player-profile-tabs";

type AttributeSectionProps = {
  title: string;
  rows: AttributeRow[];
};

function AttributeValue({ value }: { value: number | null | undefined }) {
  if (value === null || value === undefined) {
    return (
      <span className="text-on-surface-variant">{formatMissable(value)}</span>
    );
  }

  const tier = attributeValueTier(value);
  return (
    <span
      data-tier={tier}
      title={attributeTierLabel(tier)}
      className="inline-flex min-w-7 justify-center rounded-sm bg-surface-container-high px-1.5 py-0.5 data-[tier=1]:bg-score-1/10 data-[tier=1]:text-score-1 data-[tier=2]:bg-score-2/10 data-[tier=2]:text-score-2 data-[tier=3]:bg-score-3/10 data-[tier=3]:text-score-3 data-[tier=4]:bg-score-4/10 data-[tier=4]:text-score-4"
    >
      {value}
    </span>
  );
}

function AttributeSection({ title, rows }: AttributeSectionProps) {
  const headingId = `attr-group-${title.toLowerCase()}`;

  return (
    <section aria-labelledby={headingId} className="min-h-0">
      <h3 id={headingId} className="sr-only">
        {title}
      </h3>
      <dl className="grid grid-cols-1 gap-x-5 sm:grid-cols-2">
        {rows.map((row) => (
          <div
            key={row.key}
            className="flex min-h-9 min-w-0 items-center justify-between gap-3 border-b border-outline-variant/70"
          >
            <dt className="truncate text-body-md text-on-surface-variant">
              {row.label}
            </dt>
            <dd className="shrink-0 font-mono text-mono-sm tabular-nums">
              {row.potentialValue === undefined ? (
                <AttributeValue value={row.value} />
              ) : (
                <>
                  <span aria-hidden="true">
                    <AttributeValue value={row.value} />
                    <span className="px-1.5 text-on-surface-variant">→</span>
                    <AttributeValue value={row.potentialValue} />
                  </span>
                  <span className="sr-only">
                    {`Current ${formatMissable(row.value)}, Potential ${formatMissable(row.potentialValue)}`}
                  </span>
                </>
              )}
            </dd>
          </div>
        ))}
      </dl>
    </section>
  );
}

type PlayerAttributesPanelProps = {
  player: PlayerDetail;
  tab: ProfileTab;
  onTabChange: (tab: ProfileTab) => void;
  hiddenInformationRevealed: boolean;
};

function rowsForTab(
  player: PlayerDetail,
  tab: ProfileTab,
  hiddenInformationRevealed: boolean,
): AttributeRow[] {
  if (tab === "hidden") {
    if (!hiddenInformationRevealed) return [];
    return attributeRows(HIDDEN_ATTRIBUTE_KEYS, player.hiddenAttributes);
  }
  if (tab === "personality") {
    if (!hiddenInformationRevealed) return [];
    return attributeRows(PERSONALITY_ATTRIBUTE_KEYS, player.personality);
  }

  const group = VISIBLE_ATTRIBUTE_GROUPS.find(({ id }) => id === tab);
  return group
    ? attributeRows(
        group.keys,
        player.attributes,
        hiddenInformationRevealed ? player.potentialAttributes : undefined,
      )
    : [];
}

function titleForTab(tab: ProfileTab): string {
  if (tab === "hidden") return "Hidden";
  if (tab === "personality") return "Personality";
  return VISIBLE_ATTRIBUTE_GROUPS.find(({ id }) => id === tab)?.title ?? tab;
}

export function PlayerAttributesPanel({
  player,
  tab,
  onTabChange,
  hiddenInformationRevealed,
}: PlayerAttributesPanelProps) {
  const currentOnly =
    !hiddenInformationRevealed || tab === "hidden" || tab === "personality";

  return (
    <Panel
      title="Attributes"
      actions={
        <span className="text-label-sm text-on-surface-variant">
          {currentOnly ? "Current only" : "Current → Potential"}
        </span>
      }
      className="flex min-h-0 flex-col [&>div:last-child]:min-h-0 [&>div:last-child]:flex-1"
    >
      <div className="flex h-full min-h-0 flex-col gap-4">
        <div className="overflow-x-auto pb-0.5">
          <PlayerProfileTabs tab={tab} onTabChange={onTabChange} />
        </div>
        <div className="min-h-0 flex-1 overflow-y-auto pr-1">
          {PROFILE_TABS.map((id) => (
            <div key={id} {...profileTabPanelProps(id, tab)}>
              {!hiddenInformationRevealed &&
              (id === "hidden" || id === "personality") ? (
                <p
                  className="text-body-md text-on-surface-variant"
                  role="status"
                >
                  Hidden information is concealed.
                </p>
              ) : (
                <AttributeSection
                  title={titleForTab(id)}
                  rows={rowsForTab(player, id, hiddenInformationRevealed)}
                />
              )}
            </div>
          ))}
        </div>
      </div>
    </Panel>
  );
}
