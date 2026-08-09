import { Panel } from "@/components/ui/panel/panel";
import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import { formatMissable } from "@/utils/format";
import type { PlayerDetail } from "../types/player-detail";
import { groupRolesByFamily } from "../utils/position-families";
import { rolePhaseLabel } from "../utils/role-phase";

type PlayerRolesPanelProps = {
  player: PlayerDetail;
};

type RoleScoreProps = {
  roleName: string;
  label: "Current" | "Potential";
  score: number | null;
};

function RoleScore({ roleName, label, score }: RoleScoreProps) {
  return (
    <div className="flex flex-col items-center gap-1">
      <span className="text-[11px] text-on-surface-variant">{label}</span>
      {score === null ? (
        <span
          role="img"
          aria-label={`${roleName} (${label}): unavailable`}
          className="font-mono text-mono-sm text-on-surface-variant tabular-nums"
        >
          {formatMissable(null)}
        </span>
      ) : (
        <ScoreBadge
          score={score}
          roleName={`${roleName} (${label})`}
          variant="card"
        />
      )}
    </div>
  );
}

export function PlayerRolesPanel({ player }: PlayerRolesPanelProps) {
  const groups = groupRolesByFamily(player.roleScores);

  return (
    <Panel title="Roles">
      <div className="space-y-6 divide-y divide-outline-variant">
        {groups.map((group, index) => {
          const headingId = `role-family-${group.family.id}`;
          return (
            <section
              key={group.family.id}
              aria-labelledby={headingId}
              className={index === 0 ? "space-y-3" : "space-y-3 pt-6"}
            >
              <h3 id={headingId} className="text-label-lg text-on-surface">
                {group.family.title}
              </h3>
              <ul className="space-y-2">
                {group.roles.map((role) => (
                  <li
                    key={role.roleId}
                    className="flex min-w-0 items-center justify-between gap-3"
                  >
                    <div className="min-w-0">
                      <p className="truncate text-body-md text-on-surface">
                        {role.displayName}
                      </p>
                      <p className="text-[11px] text-on-surface-variant">
                        {rolePhaseLabel(role.phase)}
                      </p>
                    </div>
                    <div className="flex shrink-0 items-center gap-3">
                      <RoleScore
                        roleName={role.displayName}
                        label="Current"
                        score={role.score}
                      />
                      <RoleScore
                        roleName={role.displayName}
                        label="Potential"
                        score={role.potentialScore}
                      />
                    </div>
                  </li>
                ))}
              </ul>
            </section>
          );
        })}
      </div>
    </Panel>
  );
}
