import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Button } from "@/components/ui/button/button";
import { Modal } from "@/components/ui/modal/modal";
import { formatMoney } from "@/utils/format";
import {
  applyStaffFmChanges,
  staffFmChangesQueryOptions,
} from "../api/staff-fm-changes";
import { staffKeys } from "../api/staff-keys";
import type { StaffAssignmentContext } from "../types/staff-assignment";
import {
  STAFF_FM_TEAM_LEVEL_NAMES,
  type StaffFmChange,
  type StaffFmChangeKind,
  type StaffFmChangeResult,
} from "../types/staff-fm-changes";

const headingClass =
  "sticky top-0 bg-surface-container-lowest px-3 py-2 text-label-md text-on-surface-variant";

const KIND_LABELS: Record<StaffFmChangeKind, string> = {
  sack: "Sack",
  change: "New job",
  hire: "Hire",
};

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

function staff(count: number) {
  return count === 1 ? "1 change" : `${count} changes`;
}

function personName(person: { uid: number; name: string | null }) {
  return person.name ?? `Staff ${person.uid}`;
}

function wage(value: number | null) {
  return value === null ? "–" : `${formatMoney(value)} a week`;
}

/** What the person does now: their job and team, or their club for a hire. */
function nowDetail(change: StaffFmChange) {
  if (change.kind === "hire") {
    return change.currentClub ?? "Unemployed";
  }
  const job = change.currentJob ?? "Unknown job";
  return change.currentTeamName ? `${change.currentTeamName} ${job}` : job;
}

function becomesDetail(change: StaffFmChange) {
  if (!change.target) {
    return "Leaves the club";
  }
  return change.target.scope === "club"
    ? change.target.jobLabel
    : `${change.target.scopeDisplayName} ${change.target.jobLabel}`;
}

/**
 * Lists the sacks, job changes, and hires that put the optimizer's result into
 * FM, and makes the ticked ones there.
 */
export function StaffFmChanges({
  context,
  contextKey,
  open,
  onClose,
  onApplied,
}: {
  context: StaffAssignmentContext;
  contextKey: string;
  open: boolean;
  onClose: () => void;
  /** Called once FM has been asked, because the recommendations no longer match the club. */
  onApplied: () => void;
}) {
  const [unticked, setUnticked] = useState<ReadonlySet<number>>(new Set());
  const queryClient = useQueryClient();
  const plan = useQuery({
    ...staffFmChangesQueryOptions(context, contextKey),
    enabled: open,
  });
  const apply = useMutation({
    mutationFn: (uids: number[]) => applyStaffFmChanges(context, uids),
    // FM may have made some changes even when the request failed.
    onSettled: () => {
      onApplied();
      return queryClient.invalidateQueries({ queryKey: staffKeys.all });
    },
  });

  const changes = plan.data?.changes ?? [];
  const chosen = changes.filter((change) => !unticked.has(change.uid));
  const unavailableReason = plan.data?.unavailableReason ?? null;
  const result = apply.data;
  const allTicked = changes.length > 0 && chosen.length === changes.length;

  const toggle = (uid: number) =>
    setUnticked((previous) => {
      const next = new Set(previous);
      if (!next.delete(uid)) {
        next.add(uid);
      }
      return next;
    });
  const close = () => {
    if (apply.isPending) {
      return;
    }
    setUnticked(new Set());
    apply.reset();
    onClose();
  };

  return (
    <Modal
      open={open}
      onClose={close}
      title="Apply staff changes in FM"
      className="max-w-[60rem]!"
      footer={
        result ? (
          <Button onClick={close}>Close</Button>
        ) : (
          <>
            <Button
              variant="secondary"
              disabled={apply.isPending}
              onClick={close}
            >
              Cancel
            </Button>
            <Button
              disabled={
                chosen.length === 0 || unavailableReason !== null || !plan.data
              }
              loading={apply.isPending}
              loadingLabel="Applying…"
              onClick={() => apply.mutate(chosen.map((change) => change.uid))}
            >
              {`Apply ${staff(chosen.length)}`}
            </Button>
          </>
        )
      }
    >
      <div className="space-y-2 pb-3 text-body-md text-on-surface-variant">
        <p>
          Sacks the staff the recommendations leave without a job and hires the
          recommended people through FM's editor, as its Move to my club does,
          so FM sets each new contract and wage for the job. A new job or team
          for your own staff means sacking and hiring them again; if FM refuses
          the second step, they end up unemployed.
        </p>
        <p>
          FM does not hold these changes to the board's staff limits. Check FM's
          staff overview afterwards.
        </p>
      </div>
      {result ? (
        <ChangeResult result={result} />
      ) : plan.error ? (
        <p className="text-body-sm text-error" role="alert">
          Could not plan the changes: {errorMessage(plan.error)}
        </p>
      ) : !plan.data ? (
        <p className="text-body-sm text-on-surface-variant">
          Checking your staff…
        </p>
      ) : changes.length === 0 ? (
        <p className="text-body-md text-on-surface">
          FM already matches the recommendations.
        </p>
      ) : (
        <div className="max-h-[24rem] overflow-auto rounded-md border border-outline-variant">
          <table className="w-full border-collapse text-left text-body-sm text-on-surface">
            <caption className="sr-only">Planned staff changes in FM</caption>
            <thead>
              <tr>
                <th scope="col" className={`${headingClass} w-10`}>
                  <input
                    type="checkbox"
                    aria-label="Apply every change"
                    checked={allTicked}
                    disabled={apply.isPending}
                    onChange={() =>
                      setUnticked(
                        allTicked
                          ? new Set(changes.map((change) => change.uid))
                          : new Set(),
                      )
                    }
                  />
                </th>
                <th scope="col" className={headingClass}>
                  Person
                </th>
                <th scope="col" className={headingClass}>
                  Change
                </th>
                <th scope="col" className={headingClass}>
                  Now
                </th>
                <th scope="col" className={headingClass}>
                  Becomes
                </th>
                <th scope="col" className={`${headingClass} text-right`}>
                  Wage now
                </th>
              </tr>
            </thead>
            <tbody>
              {changes.map((change) => (
                <tr
                  key={change.uid}
                  className="border-t border-outline-variant"
                >
                  <td className="px-3 py-2">
                    <input
                      type="checkbox"
                      aria-label={`${KIND_LABELS[change.kind]} ${personName(change)}`}
                      checked={!unticked.has(change.uid)}
                      disabled={apply.isPending}
                      onChange={() => toggle(change.uid)}
                    />
                  </td>
                  <th scope="row" className="px-3 py-2 font-normal">
                    {personName(change)}
                  </th>
                  <td className="px-3 py-2">{KIND_LABELS[change.kind]}</td>
                  <td className="px-3 py-2 text-on-surface-variant">
                    {nowDetail(change)}
                  </td>
                  <td className="px-3 py-2">{becomesDetail(change)}</td>
                  <td className="px-3 py-2 text-right text-on-surface-variant">
                    {wage(change.weeklyWageGbp)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
      {!result && unavailableReason ? (
        <p className="pt-3 text-body-sm text-error" role="alert">
          {unavailableReason}
        </p>
      ) : null}
      {apply.error ? (
        <p className="pt-3 text-body-sm text-error" role="alert">
          The changes failed: {errorMessage(apply.error)}. Load Data to see FM's
          staff.
        </p>
      ) : null}
    </Modal>
  );
}

function ChangeResult({ result }: { result: StaffFmChangeResult }) {
  return (
    <div className="space-y-2" role="status">
      <p className="text-body-md text-on-surface">
        {result.done.length === 0
          ? "FM made no changes."
          : `FM made ${staff(result.done.length)}.`}
      </p>
      {result.done.length > 0 ? (
        <ul className="list-disc space-y-1 pl-5 text-body-sm text-on-surface">
          {result.done.map((done) => (
            <li key={done.uid}>
              {personName(done)}:{" "}
              {done.kind === "sack"
                ? "left the club"
                : `${done.jobLabel ?? "new job"}, ${
                    done.teamLevel
                      ? STAFF_FM_TEAM_LEVEL_NAMES[done.teamLevel]
                      : "team unknown"
                  }, ${wage(done.weeklyWageGbp)}`}
            </li>
          ))}
        </ul>
      ) : null}
      {result.failed.length > 0 ? (
        <ul className="list-disc space-y-1 pl-5 text-body-sm text-on-surface-variant">
          {result.failed.map((failure) => (
            <li key={failure.uid}>
              {personName(failure)}: {failure.reason}
            </li>
          ))}
        </ul>
      ) : null}
      {result.error ? (
        <p className="text-body-sm text-error">
          {result.error}. Load Data to see FM's staff.
        </p>
      ) : null}
    </div>
  );
}
