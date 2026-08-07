import { GraduationCap } from "lucide-react";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";

export function AcademyGraduatesWorkspace() {
  return (
    <Panel title="Graduates">
      <EmptyState icon={GraduationCap} title="Graduate data unavailable">
        Senior league appearances are not available from the current memory
        reader, so graduate status and totals remain unavailable.
      </EmptyState>
    </Panel>
  );
}
