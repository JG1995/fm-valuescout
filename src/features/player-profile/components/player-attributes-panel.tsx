import { Panel } from "@/components/ui/panel/panel";
import { formatMissable } from "@/utils/format";
import type { PlayerDetail } from "../types/player-detail";
import {
  type AttributeRow,
  attributeRows,
  HIDDEN_ATTRIBUTE_KEYS,
  PERSONALITY_ATTRIBUTE_KEYS,
  VISIBLE_ATTRIBUTE_GROUPS,
} from "../utils/attribute-groups";

type AttributeSectionProps = {
  title: string;
  rows: AttributeRow[];
};

function AttributeSection({ title, rows }: AttributeSectionProps) {
  const headingId = `attr-group-${title.toLowerCase()}`;

  return (
    <section aria-labelledby={headingId} className="space-y-3">
      <h3 id={headingId} className="text-label-lg text-on-surface">
        {title}
      </h3>
      <dl className="grid grid-cols-1 gap-x-6 gap-y-2 sm:grid-cols-2 lg:grid-cols-3">
        {rows.map((row) => (
          <div
            key={row.key}
            className="flex min-w-0 items-baseline justify-between gap-3"
          >
            <dt className="truncate text-body-md text-on-surface-variant">
              {row.label}
            </dt>
            <dd className="shrink-0 font-mono text-mono-sm text-on-surface tabular-nums">
              {row.potentialValue === undefined ? (
                formatMissable(row.value)
              ) : (
                <>
                  <span aria-hidden="true">
                    <span>{formatMissable(row.value)}</span>
                    <span className="px-1 text-on-surface-variant">→</span>
                    <span>{formatMissable(row.potentialValue)}</span>
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
};

export function PlayerAttributesPanel({ player }: PlayerAttributesPanelProps) {
  return (
    <Panel title="Attributes">
      <div className="space-y-6 divide-y divide-outline-variant">
        {VISIBLE_ATTRIBUTE_GROUPS.map((group, index) => (
          <div key={group.id} className={index === 0 ? undefined : "pt-6"}>
            <AttributeSection
              title={group.title}
              rows={attributeRows(
                group.keys,
                player.attributes,
                player.potentialAttributes,
              )}
            />
          </div>
        ))}
        <div className="pt-6">
          <AttributeSection
            title="Hidden"
            rows={attributeRows(HIDDEN_ATTRIBUTE_KEYS, player.hiddenAttributes)}
          />
        </div>
        <div className="pt-6">
          <AttributeSection
            title="Personality"
            rows={attributeRows(PERSONALITY_ATTRIBUTE_KEYS, player.personality)}
          />
        </div>
      </div>
    </Panel>
  );
}
