import { Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button/button";
import { Panel } from "@/components/ui/panel/panel";
import type { AcademyClass } from "../types/academy";

type AcademyClassWorkspaceProps = {
  academyClass: AcademyClass;
  onDelete: () => void;
};

export function AcademyClassWorkspace({
  academyClass,
  onDelete,
}: AcademyClassWorkspaceProps) {
  return (
    <Panel
      title={`Class of ${academyClass.classYear}`}
      actions={
        <Button variant="destructive" icon={Trash2} onClick={onDelete}>
          Delete class
        </Button>
      }
    >
      <dl>
        <div>
          <dt className="text-label-md text-on-surface-variant">
            Tracked players
          </dt>
          <dd className="mt-1 text-headline-md text-on-surface">
            {academyClass.memberCount}
          </dd>
        </div>
      </dl>
      <p className="mt-6 text-body-md text-on-surface-variant">
        Player roster details are not available in this workspace yet.
      </p>
    </Panel>
  );
}
