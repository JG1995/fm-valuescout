import { FolderOpen, Plus } from "lucide-react";
import { Button } from "@/components/ui/button/button";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import type { AcademyClass } from "../types/academy";

type AcademyOverviewProps = {
  classes: AcademyClass[];
  onCreate: () => void;
  onOpenClass: (academyClass: AcademyClass) => void;
};

export function AcademyOverview({
  classes,
  onCreate,
  onOpenClass,
}: AcademyOverviewProps) {
  return (
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
          Create a class to start grouping players by the year they came through
          your club.
        </EmptyState>
      ) : (
        <ul className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {classes.map((academyClass) => (
            <li key={academyClass.id}>
              <button
                type="button"
                aria-label={`Open Class of ${academyClass.classYear}`}
                className="flex min-h-28 w-full cursor-pointer flex-col justify-between rounded-lg border border-outline-variant bg-surface-container-high p-4 text-left transition-colors duration-150 ease-out hover:bg-surface-container-highest focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
                onClick={() => onOpenClass(academyClass)}
              >
                <span className="text-headline-sm text-on-surface">
                  Class of {academyClass.classYear}
                </span>
                <span className="text-body-sm text-on-surface-variant">
                  {academyClass.memberCount} tracked player
                  {academyClass.memberCount === 1 ? "" : "s"}
                </span>
              </button>
            </li>
          ))}
        </ul>
      )}
    </Panel>
  );
}
