import { Eye, EyeOff } from "lucide-react";
import { Button } from "@/components/ui/button/button";
import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import { formatMissable, formatMoney, formatPlayerDob } from "@/utils/format";
import type { StaffDetail } from "../types/staff-detail";
import { StaffCaBoost } from "./staff-ca-boost";

function SummaryFact({
  label,
  value,
  numeric = false,
}: {
  label: string;
  value: string | number;
  numeric?: boolean;
}) {
  return (
    <div className="min-w-0">
      <dt className="text-label-sm text-on-surface-variant uppercase tracking-[0.08em]">
        {label}
      </dt>
      <dd
        className={
          numeric
            ? "font-mono text-mono-md text-on-surface tabular-nums"
            : "truncate text-body-md text-on-surface"
        }
      >
        {value}
      </dd>
    </div>
  );
}

export function StaffOverviewPanel({
  staff,
  hiddenInformationPending,
  hiddenInformationError,
  onToggleHiddenInformation,
  boostPending,
  boostError,
  onBoost,
  onOpenBoostConfirmation,
  fallbackFocusTo,
}: {
  staff: StaffDetail;
  hiddenInformationPending: boolean;
  hiddenInformationError: Error | null;
  onToggleHiddenInformation: () => void;
  boostPending: boolean;
  boostError: Error | null;
  onBoost: () => Promise<unknown>;
  onOpenBoostConfirmation: () => void;
  fallbackFocusTo: () => HTMLElement | null;
}) {
  const VisibilityIcon = staff.hiddenInformationRevealed ? EyeOff : Eye;
  const bestRole = staff.roleScores.reduce<
    (typeof staff.roleScores)[number] | undefined
  >(
    (best, role) =>
      role.score !== null &&
      (best === undefined || best.score === null || role.score > best.score)
        ? role
        : best,
    undefined,
  );
  const dob =
    staff.birthYear !== null && staff.birthDayOfYear !== null
      ? formatPlayerDob(staff.birthYear, staff.birthDayOfYear, staff.age)
      : formatMissable(staff.age);

  return (
    <section
      aria-label={`${staff.name ?? "Staff"} summary`}
      className="rounded-lg border border-outline-variant bg-surface-container p-4"
    >
      <div className="grid gap-x-4 gap-y-2 lg:grid-cols-[minmax(260px,1.15fr)_minmax(300px,1fr)_minmax(260px,0.9fr)] lg:items-start">
        <div className="min-w-0">
          <h1
            className="truncate text-headline-lg text-on-surface"
            title={staff.name ?? undefined}
          >
            {formatMissable(staff.name)}
          </h1>
          <p className="mt-0.5 truncate text-body-md text-on-surface-variant">
            {formatMissable(staff.club)}
            {staff.division ? ` · ${staff.division}` : ""}
          </p>
        </div>

        <div className="min-w-0 lg:col-span-2 lg:flex lg:self-end lg:justify-end">
          <div className="space-y-2">
            <div className="flex flex-wrap items-start justify-end gap-2">
              {staff.hiddenInformationRevealed ? (
                <StaffCaBoost
                  staff={staff}
                  pending={boostPending}
                  error={boostError}
                  onBoost={onBoost}
                  onOpenConfirmation={onOpenBoostConfirmation}
                  fallbackFocusTo={fallbackFocusTo}
                />
              ) : null}
              <Button
                icon={VisibilityIcon}
                variant="secondary"
                aria-label={
                  staff.hiddenInformationRevealed
                    ? "Hide hidden info"
                    : "Reveal hidden info"
                }
                aria-pressed={staff.hiddenInformationRevealed}
                disabled={hiddenInformationPending}
                loading={hiddenInformationPending}
                loadingLabel="Updating…"
                onClick={onToggleHiddenInformation}
              >
                {staff.hiddenInformationRevealed
                  ? "Hide hidden info"
                  : "Reveal hidden info"}
              </Button>
            </div>
            {hiddenInformationError ? (
              <p className="text-right text-body-sm text-error" role="alert">
                Could not update hidden information.
              </p>
            ) : null}
          </div>
        </div>

        <dl className="grid grid-cols-2 gap-x-4 gap-y-2 sm:grid-cols-4 lg:grid-cols-2 xl:grid-cols-4">
          <SummaryFact label="Age / DOB" value={dob} />
          <SummaryFact
            label="Nationality"
            value={
              staff.nationalities.length > 0
                ? staff.nationalities.join(", ")
                : "—"
            }
          />
          <SummaryFact
            label="Wage"
            value={
              staff.weeklyWageGbp === null
                ? "—"
                : formatMoney(staff.weeklyWageGbp)
            }
            numeric
          />
          <SummaryFact
            label="Contract expiry"
            value={formatMissable(staff.contractExpiryYear)}
            numeric
          />
        </dl>

        <div className="grid min-w-0 grid-cols-2 gap-3 border-outline-variant lg:border-x lg:px-4">
          <div className="flex min-w-0 items-start gap-3">
            {bestRole?.score === null || bestRole === undefined ? (
              <span className="inline-flex size-12 items-center justify-center font-mono text-mono-lg text-on-surface-variant tabular-nums">
                —
              </span>
            ) : (
              <ScoreBadge
                score={bestRole.score}
                roleName="Best role fit"
                variant="hero"
              />
            )}
            <div className="min-w-0">
              <p className="text-label-sm text-on-surface-variant uppercase tracking-[0.08em]">
                Best role fit
              </p>
              <p className="truncate text-body-md text-on-surface">
                {bestRole?.displayName ?? "—"}
              </p>
            </div>
          </div>
        </div>

        <dl className="grid min-w-0 grid-cols-2 gap-3">
          <SummaryFact label="CA" value={staff.ca} numeric />
          {staff.hiddenInformationRevealed ? (
            <SummaryFact label="PA" value={staff.pa} numeric />
          ) : null}
        </dl>
      </div>
    </section>
  );
}
