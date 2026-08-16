import { useVirtualizer } from "@tanstack/react-virtual";
import { useRef } from "react";
import { Panel } from "@/components/ui/panel/panel";
import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import { formatMissable } from "@/utils/format";
import type { StaffDetail, StaffRoleScore } from "../types/staff-detail";

const ROLE_ROW_HEIGHT = 48;

function orderedScores(scores: StaffRoleScore[]) {
  return scores
    .map((score, index) => ({ score, index }))
    .sort((left, right) => {
      if (left.score.score === right.score.score)
        return left.index - right.index;
      if (left.score.score === null) return 1;
      if (right.score.score === null) return -1;
      return right.score.score - left.score.score;
    })
    .map(({ score }) => score);
}

export function StaffRoleFitPanel({ staff }: { staff: StaffDetail }) {
  const scrollRef = useRef<HTMLElement>(null);
  const scores = orderedScores(staff.roleScores);
  const virtualizer = useVirtualizer({
    count: scores.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => ROLE_ROW_HEIGHT,
    overscan: 3,
    initialRect: { width: 600, height: 320 },
    observeElementRect: (instance, callback) => {
      const measure = () => {
        const element = instance.scrollElement;
        const width = element?.clientWidth ?? 0;
        const height = element?.clientHeight ?? 0;
        callback({
          width: width > 0 ? width : 600,
          height: height > 0 ? height : 320,
        });
      };
      measure();
      const element = instance.scrollElement;
      if (!element || typeof ResizeObserver === "undefined") return () => {};
      const observer = new ResizeObserver(measure);
      observer.observe(element);
      return () => observer.disconnect();
    },
  });
  const virtualRows = virtualizer.getVirtualItems();
  const paddingTop = virtualRows[0]?.start ?? 0;
  const paddingBottom = virtualRows.length
    ? virtualizer.getTotalSize() - virtualRows[virtualRows.length - 1].end
    : 0;

  return (
    <Panel
      title="Role fit"
      actions={
        <span className="text-label-sm text-on-surface-variant">
          Current score
        </span>
      }
      className="flex min-h-0 flex-col"
      contentClassName="flex min-h-0 flex-1 flex-col"
    >
      <section
        ref={scrollRef}
        aria-label="Staff role fit scores"
        data-testid="staff-role-fit-scroller"
        className="h-full min-h-0 overflow-y-auto pr-1"
      >
        <table
          aria-rowcount={scores.length + 1}
          className="w-full table-fixed border-collapse"
        >
          <caption className="sr-only">Current staff role scores</caption>
          <colgroup>
            <col />
            <col className="w-[84px]" />
          </colgroup>
          <thead className="sticky top-0 z-10 bg-surface-container">
            <tr className="border-b border-outline-variant">
              <th
                scope="col"
                className="pb-2 text-left text-label-md text-on-surface-variant"
              >
                Job title
              </th>
              <th
                scope="col"
                className="pb-2 text-center text-label-md text-on-surface-variant"
              >
                Score
              </th>
            </tr>
          </thead>
          <tbody>
            {paddingTop > 0 ? (
              // biome-ignore lint/a11y/noAriaHiddenOnFocusable: this virtual spacer contains no focusable content and is not a data row.
              <tr aria-hidden="true">
                <td colSpan={2} style={{ height: paddingTop }} />
              </tr>
            ) : null}
            {virtualRows.map((virtualRow) => {
              const role = scores[virtualRow.index];
              if (!role) return null;
              return (
                <tr
                  key={role.roleId}
                  data-index={virtualRow.index}
                  aria-rowindex={virtualRow.index + 2}
                  className="border-b border-outline-variant/70"
                  style={{ height: virtualRow.size }}
                >
                  <td className="min-w-0 pr-2 text-body-md text-on-surface">
                    {role.displayName}
                  </td>
                  <td className="text-center">
                    {role.score === null ? (
                      <span className="font-mono text-mono-sm text-on-surface-variant tabular-nums">
                        {formatMissable(null)}
                      </span>
                    ) : (
                      <ScoreBadge
                        score={role.score}
                        roleName={`${role.displayName} current score`}
                        variant="card"
                      />
                    )}
                  </td>
                </tr>
              );
            })}
            {paddingBottom > 0 ? (
              // biome-ignore lint/a11y/noAriaHiddenOnFocusable: this virtual spacer contains no focusable content and is not a data row.
              <tr aria-hidden="true">
                <td colSpan={2} style={{ height: paddingBottom }} />
              </tr>
            ) : null}
          </tbody>
        </table>
      </section>
    </Panel>
  );
}
