import { AttributeRow } from "@/components/ui/attribute-row/attribute-row";
import { AttributeValue } from "@/components/ui/attribute-value/attribute-value";
import { Panel } from "@/components/ui/panel/panel";
import { formatMissable } from "@/utils/format";
import type { PlayerDetail } from "../types/player-detail";
import {
  type AttributeGroup,
  attributeRows,
  GOALKEEPER_OUTFIELD_ATTRIBUTE_GROUPS,
  GOALKEEPER_PRIMARY_ATTRIBUTE_GROUPS,
  GOALKEEPING_ATTRIBUTE_GROUP,
  HIDDEN_ATTRIBUTE_KEYS,
  OUTFIELD_ATTRIBUTE_GROUPS,
  PERSONALITY_ATTRIBUTE_KEYS,
  type AttributeRow as PlayerAttributeRow,
} from "../utils/attribute-groups";
import { isGoalkeeper } from "../utils/position-families";
import { type ProfileTab, profileTabsForPlayer } from "../utils/profile-tab";
import { PlayerProfileTabs, profileTabPanelProps } from "./player-profile-tabs";

type AttributeSectionProps = {
  group: AttributeGroup;
  player: PlayerDetail;
  hiddenInformationRevealed: boolean;
};

function AttributeRows({ rows }: { rows: PlayerAttributeRow[] }) {
  return (
    <dl className="grid min-w-0 grid-cols-1 gap-x-5">
      {rows.map((row) => (
        <AttributeRow key={row.key} label={row.label}>
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
        </AttributeRow>
      ))}
    </dl>
  );
}

function AttributeSection({
  group,
  player,
  hiddenInformationRevealed,
}: AttributeSectionProps) {
  const headingId = `attr-group-${group.id}`;
  const potentialValues = hiddenInformationRevealed
    ? player.potentialAttributes
    : undefined;

  return (
    <section aria-labelledby={headingId} className="min-h-0 min-w-0 space-y-3">
      <h3 id={headingId} className="text-label-lg text-on-surface">
        {group.title}
      </h3>
      <AttributeRows
        rows={attributeRows(group.keys, player.attributes, potentialValues)}
      />
      {group.subgroups?.map((subgroup) => {
        const subgroupId = `${headingId}-${subgroup.title
          .toLowerCase()
          .replaceAll(" ", "-")}`;
        return (
          <section
            key={subgroup.title}
            aria-labelledby={subgroupId}
            className="space-y-2"
          >
            <h4
              id={subgroupId}
              className="text-label-md text-on-surface-variant"
            >
              {subgroup.title}
            </h4>
            <AttributeRows
              rows={attributeRows(
                subgroup.keys,
                player.attributes,
                potentialValues,
              )}
            />
          </section>
        );
      })}
    </section>
  );
}

type PlayerAttributesPanelProps = {
  player: PlayerDetail;
  tab: ProfileTab;
  onTabChange: (tab: ProfileTab) => void;
  hiddenInformationRevealed: boolean;
};

export function PlayerAttributesPanel({
  player,
  tab,
  onTabChange,
  hiddenInformationRevealed,
}: PlayerAttributesPanelProps) {
  const goalkeeper = isGoalkeeper(player.positions);
  const tabs = profileTabsForPlayer(goalkeeper);
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
          <PlayerProfileTabs tab={tab} tabs={tabs} onTabChange={onTabChange} />
        </div>
        <div className="min-h-0 flex-1 overflow-y-auto pr-1">
          {tabs.map((id) => (
            <div key={id} {...profileTabPanelProps(id, tab)}>
              {!hiddenInformationRevealed &&
              (id === "hidden" || id === "personality") ? (
                <p
                  className="text-body-md text-on-surface-variant"
                  role="status"
                >
                  Hidden information is concealed.
                </p>
              ) : id === "outfield" || (id === "goalkeeping" && goalkeeper) ? (
                <div className="grid gap-5 lg:grid-cols-3">
                  {(id === "goalkeeping"
                    ? GOALKEEPER_PRIMARY_ATTRIBUTE_GROUPS
                    : goalkeeper
                      ? GOALKEEPER_OUTFIELD_ATTRIBUTE_GROUPS
                      : OUTFIELD_ATTRIBUTE_GROUPS
                  ).map((group) => (
                    <AttributeSection
                      key={group.id}
                      group={group}
                      player={player}
                      hiddenInformationRevealed={hiddenInformationRevealed}
                    />
                  ))}
                </div>
              ) : (
                <AttributeSection
                  group={
                    id === "goalkeeping"
                      ? GOALKEEPING_ATTRIBUTE_GROUP
                      : {
                          id,
                          title: id === "hidden" ? "Hidden" : "Personality",
                          keys:
                            id === "hidden"
                              ? HIDDEN_ATTRIBUTE_KEYS
                              : PERSONALITY_ATTRIBUTE_KEYS,
                        }
                  }
                  player={
                    id === "hidden" || id === "personality"
                      ? {
                          ...player,
                          attributes:
                            id === "hidden"
                              ? player.hiddenAttributes
                              : player.personality,
                        }
                      : player
                  }
                  hiddenInformationRevealed={
                    id === "goalkeeping" ? hiddenInformationRevealed : false
                  }
                />
              )}
            </div>
          ))}
        </div>
      </div>
    </Panel>
  );
}
