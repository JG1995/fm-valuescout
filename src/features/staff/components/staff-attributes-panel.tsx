import { Panel } from "@/components/ui/panel/panel";
import { formatMissable } from "@/utils/format";
import type { StaffDetail } from "../types/staff-detail";
import {
  STAFF_PROFILE_ATTRIBUTE_GROUPS,
  type StaffProfileAttributeGroup,
} from "../utils/staff-profile-attributes";
import type { StaffProfileTab } from "../utils/staff-profile-tab";
import {
  StaffProfileTabs,
  staffProfileTabPanelProps,
} from "./staff-profile-tabs";

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
          <div
            key={key}
            className="flex min-h-9 min-w-0 items-center justify-between gap-3 border-b border-outline-variant/70"
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

export function StaffAttributesPanel({
  staff,
  tab,
  onTabChange,
}: {
  staff: StaffDetail;
  tab: StaffProfileTab;
  onTabChange: (tab: StaffProfileTab) => void;
}) {
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
      <div className="flex h-full min-h-0 flex-col gap-4">
        <div className="overflow-x-auto pb-0.5">
          <StaffProfileTabs tab={tab} onTabChange={onTabChange} />
        </div>
        <div className="min-h-0 flex-1 overflow-y-auto pr-1">
          {STAFF_PROFILE_ATTRIBUTE_GROUPS.map((group) => (
            <div key={group.id} {...staffProfileTabPanelProps(group.id, tab)}>
              <div className="grid gap-5 lg:grid-cols-2">
                <AttributeSection group={group} staff={staff} />
              </div>
            </div>
          ))}
        </div>
      </div>
    </Panel>
  );
}
