import { GraduationCap } from "lucide-react";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { formatMissable } from "@/utils/format";
import type { AcademyClassDetail, AcademyMember } from "../types/academy";
import { academyMemberIsGraduate } from "../utils/academy-statistics";

type AcademyGraduatesWorkspaceProps = {
  classDetails: AcademyClassDetail[];
  detailsReady: boolean;
  detailsPending: boolean;
  detailsError?: unknown;
};

export function AcademyGraduatesWorkspace({
  classDetails,
  detailsReady,
  detailsPending,
  detailsError,
}: AcademyGraduatesWorkspaceProps) {
  if (detailsError) {
    return (
      <Panel title="Graduates">
        <p className="text-body-sm text-error" role="alert">
          Could not load graduate data. {errorMessage(detailsError)}
        </p>
      </Panel>
    );
  }

  if (detailsPending) {
    return (
      <Panel title="Graduates">
        <p className="text-body-md text-on-surface-variant">
          Loading graduate data…
        </p>
      </Panel>
    );
  }

  if (!detailsReady) {
    return <AcademyGraduatesUnavailable />;
  }

  const members = classDetails.flatMap((detail) => detail.members);
  if (
    members.length === 0 ||
    members.some((member) => member.seniorLeagueAppearances === null)
  ) {
    return <AcademyGraduatesUnavailable />;
  }

  const graduates = classDetails.flatMap((detail) =>
    detail.members
      .filter((member) => academyMemberIsGraduate(member) === true)
      .map((member) => ({ member, academyClass: detail })),
  );

  if (graduates.length === 0) {
    return (
      <Panel title="Graduates">
        <EmptyState icon={GraduationCap} title="No graduates yet">
          A player becomes a graduate after one or more reported senior league
          appearances.
        </EmptyState>
      </Panel>
    );
  }

  return (
    <Panel title="Graduates">
      <div className="overflow-x-auto rounded-lg border border-outline-variant">
        <table className="min-w-[720px] w-full border-collapse text-left">
          <caption className="sr-only">Youth Academy graduates</caption>
          <thead className="bg-surface-container-lowest">
            <tr>
              {["Player", "Class", "Senior league apps", "Current club"].map(
                (label) => (
                  <th
                    key={label}
                    scope="col"
                    className={`h-table-header-height px-2 text-label-md text-on-surface-variant uppercase ${
                      label === "Senior league apps"
                        ? "text-right"
                        : "text-left"
                    }`}
                  >
                    {label}
                  </th>
                ),
              )}
            </tr>
          </thead>
          <tbody>
            {graduates.map(({ member, academyClass }) => (
              <GraduateRow
                key={`${academyClass.id}-${member.playerUid}`}
                academyClass={academyClass}
                member={member}
              />
            ))}
          </tbody>
        </table>
      </div>
    </Panel>
  );
}

function AcademyGraduatesUnavailable() {
  return (
    <Panel title="Graduates">
      <EmptyState icon={GraduationCap} title="Graduate data unavailable">
        Senior league appearances are not available from the current memory
        reader, so graduate status and totals remain unavailable.
      </EmptyState>
    </Panel>
  );
}

function GraduateRow({
  academyClass,
  member,
}: {
  academyClass: AcademyClassDetail;
  member: AcademyMember;
}) {
  const name = member.currentName ?? member.lastKnownName;

  return (
    <tr className="h-table-row-height border-t border-outline-variant">
      <td className="px-2 text-body-sm text-on-surface">{name}</td>
      <td className="px-2 text-body-sm text-on-surface-variant">
        Class of {academyClass.classYear}
      </td>
      <td className="px-2 text-right text-body-sm tabular-nums">
        {formatMissable(member.seniorLeagueAppearances)}
      </td>
      <td className="px-2 text-body-sm text-on-surface-variant">
        {formatMissable(member.currentClub)}
      </td>
    </tr>
  );
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}
