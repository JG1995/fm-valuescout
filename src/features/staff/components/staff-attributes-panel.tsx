import { Panel } from "@/components/ui/panel/panel";
import { formatMissable } from "@/utils/format";
import type { StaffDetail } from "../types/staff-detail";
import {
  STAFF_PROFILE_ATTRIBUTE_GROUPS,
  type StaffProfileAttributeGroup,
} from "../utils/staff-profile-attributes";

function AttributeSection({
  group,
  staff,
}: {
  group: StaffProfileAttributeGroup;
  staff: StaffDetail;
}) {
  return (
    <section
      aria-labelledby={`staff-attribute-${group.id}`}
      className="space-y-3"
    >
      <h3
        id={`staff-attribute-${group.id}`}
        className="text-label-lg text-on-surface"
      >
        {group.title}
      </h3>
      <dl className="grid min-w-0 grid-cols-[minmax(0,max-content)_auto] gap-x-2">
        {group.keys.map((key) => (
          <div
            key={key}
            className="col-span-2 grid min-h-9 min-w-0 grid-cols-subgrid items-center border-b border-outline-variant/70"
          >
            <dt className="truncate text-body-md text-on-surface-variant">
              {key.replaceAll(/([a-z])([A-Z])/g, "$1 $2")}
            </dt>
            <dd className="shrink-0 font-mono text-mono-sm tabular-nums">
              {formatMissable(staff.attributes[key])}
            </dd>
          </div>
        ))}
      </dl>
    </section>
  );
}

export function StaffAttributesPanel({ staff }: { staff: StaffDetail }) {
  return (
    <Panel
      title="Attributes"
      actions={
        <span className="text-label-sm text-on-surface-variant">
          Current only
        </span>
      }
      className="flex min-h-0 flex-col [&>div:last-child]:min-h-0 [&>div:last-child]:flex-1"
    >
      <div className="grid h-full min-h-0 grid-cols-3 gap-5 overflow-y-auto pr-1">
        {STAFF_PROFILE_ATTRIBUTE_GROUPS.map((group) => (
          <AttributeSection key={group.id} group={group} staff={staff} />
        ))}
      </div>
    </Panel>
  );
}
