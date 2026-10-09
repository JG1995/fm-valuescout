import { FolderOpen, Plus } from "lucide-react";
import { Button } from "@/components/ui/button/button";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { formatCount, formatMoney } from "@/utils/format";
import type { AcademyClass, AcademyClassDetail } from "../types/academy";
import {
  academyDetailsAreComplete,
  summarizeAcademyMembers,
  unavailableAcademyStatistics,
} from "../utils/academy-statistics";
import {
  AcademyStatistics,
  type AcademyStatisticsStatus,
} from "./academy-statistics";

type AcademyOverviewProps = {
  classes: AcademyClass[];
  classDetails: AcademyClassDetail[];
  classDetailsReady: boolean;
  classDetailsPending: boolean;
  classDetailsError?: unknown;
  onCreate: () => void;
  onOpenClass: (academyClass: AcademyClass) => void;
};

export function AcademyOverview({
  classes,
  classDetails,
  classDetailsReady,
  classDetailsPending,
  classDetailsError,
  onCreate,
  onOpenClass,
}: AcademyOverviewProps) {
  const members = classDetails.flatMap((detail) => detail.members);
  const statistics = classDetailsReady
    ? summarizeAcademyMembers(members)
    : unavailableAcademyStatistics();
  const statisticsStatus: AcademyStatisticsStatus = classDetailsError
    ? "error"
    : classDetailsPending || !classDetailsReady
      ? "loading"
      : "ready";

  return (
    <div className="space-y-4">
      <AcademyStatistics
        classes={classes.length}
        trackedPlayers={classes.reduce(
          (total, academyClass) => total + academyClass.memberCount,
          0,
        )}
        statistics={statistics}
        status={statisticsStatus}
      />
      <Panel
        title="Classes"
        actions={
          <Button icon={Plus} onClick={onCreate}>
            Create class
          </Button>
        }
      >
        {classes.length === 0 ? (
          <EmptyState icon={FolderOpen} title="No academy classes yet">
            Create a class to start grouping players by the year they came
            through your club.
          </EmptyState>
        ) : (
          <ul className="divide-y divide-outline-variant">
            {classes.map((academyClass) => {
              const detail = classDetails.find(
                (candidate) => candidate.id === academyClass.id,
              );
              const classStatistics =
                classDetailsReady &&
                detail &&
                academyDetailsAreComplete([academyClass], [detail])
                  ? summarizeAcademyMembers(detail.members)
                  : null;

              return (
                <li key={academyClass.id}>
                  <button
                    type="button"
                    aria-label={`Open Class of ${academyClass.classYear}`}
                    className="flex min-h-16 w-full cursor-pointer flex-wrap items-center gap-x-4 gap-y-2 rounded-sm px-2 py-2 text-left transition-colors duration-150 ease-out hover:bg-surface-container-high focus-visible:outline-2 focus-visible:outline-offset-[-2px] focus-visible:outline-primary"
                    onClick={() => onOpenClass(academyClass)}
                  >
                    <span className="w-40 shrink-0 text-body-md font-semibold text-on-surface">
                      Class of {academyClass.classYear}
                    </span>
                    <span className="min-w-40 text-body-sm text-on-surface-variant">
                      {academyClass.memberCount} tracked player
                      {academyClass.memberCount === 1 ? "" : "s"}
                    </span>
                    <span className="min-w-40 text-body-sm text-on-surface-variant">
                      Reported senior:{" "}
                      {classStatistics?.reportedSeniorPlayers ?? "—"}
                    </span>
                    <span className="flex flex-wrap items-center gap-x-4 gap-y-1 text-body-sm text-on-surface-variant">
                      <span>
                        {formatClassCount(
                          classStatistics?.graduates,
                          "graduate",
                        )}
                      </span>
                      <span>
                        {formatClassIncome(classStatistics?.saleFeeEur)} sale
                        income
                      </span>
                    </span>
                    <span className="ml-auto inline-flex items-center gap-2 text-label-md text-on-surface">
                      <FolderOpen
                        aria-hidden
                        className="size-4"
                        strokeWidth={1.5}
                      />
                      Open
                    </span>
                  </button>
                </li>
              );
            })}
          </ul>
        )}
      </Panel>
    </div>
  );
}

function formatClassCount(value: number | null | undefined, singular: string) {
  if (value === null || value === undefined) {
    return `— ${singular}s`;
  }
  return `${formatCount(value)} ${value === 1 ? singular : `${singular}s`}`;
}

function formatClassIncome(value: number | null | undefined) {
  return value === null || value === undefined ? "—" : formatMoney(value);
}
