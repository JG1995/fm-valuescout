import { AttributeRow } from "@/components/ui/attribute-row/attribute-row";
import { AttributeValue } from "@/components/ui/attribute-value/attribute-value";
import { Panel } from "@/components/ui/panel/panel";
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
      <dl className="grid min-w-0 grid-cols-1 gap-x-5">
        {group.keys.map((key) => (
          <AttributeRow
            key={key}
            label={key.replaceAll(/([a-z])([A-Z])/g, "$1 $2")}
          >
            <AttributeValue value={staff.attributes[key]} />
          </AttributeRow>
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
