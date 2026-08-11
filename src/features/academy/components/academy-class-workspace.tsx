import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  CircleCheck,
  CircleMinus,
  HandCoins,
  RotateCcw,
  Trash2,
  TriangleAlert,
  UserPlus,
  UserRoundMinus,
} from "lucide-react";
import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { SelectField } from "@/components/ui/field/select-field";
import { Modal } from "@/components/ui/modal/modal";
import { Panel } from "@/components/ui/panel/panel";
import {
  formatMissable,
  formatMoney,
  formatPreferredFoot,
} from "@/utils/format";
import { academyClassQueryOptions } from "../api/academy-class-query-options";
import { academyKeys } from "../api/academy-keys";
import { removeAcademyMember } from "../api/remove-academy-member";
import type { AcademyClass, AcademyMember } from "../types/academy";
import {
  summarizeAcademyMembers,
  unavailableAcademyStatistics,
} from "../utils/academy-statistics";
import { AcademyAddPlayersModal } from "./academy-add-players-modal";
import {
  AcademyMemberOutcomeModal,
  type AcademyMemberOutcomeMode,
} from "./academy-member-outcome-modal";
import {
  AcademyStatistics,
  type AcademyStatisticsStatus,
} from "./academy-statistics";

type AcademyClassWorkspaceProps = {
  academyClass: AcademyClass;
  academyClasses: AcademyClass[];
  clubOptions: string[];
  onSelectClass: (academyClass: AcademyClass) => void;
  onDelete: () => void;
};

type OutcomeTarget = {
  member: AcademyMember;
  mode: AcademyMemberOutcomeMode;
  returnFocusTo: HTMLButtonElement | null;
};

type RemovalTarget = {
  member: AcademyMember;
  returnFocusTo: HTMLButtonElement | null;
};

const rosterGroups = [
  {
    id: "still-at-club",
    title: "Still at club",
    includes: (member: AcademyMember) => member.outcome === null,
  },
  {
    id: "sold",
    title: "Sold",
    includes: (member: AcademyMember) => member.outcome?.status === "sold",
  },
  {
    id: "released",
    title: "Released",
    includes: (member: AcademyMember) => member.outcome?.status === "released",
  },
] as const;

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

export function AcademyClassWorkspace({
  academyClass,
  academyClasses,
  clubOptions,
  onSelectClass,
  onDelete,
}: AcademyClassWorkspaceProps) {
  const queryClient = useQueryClient();
  const [addPlayersOpen, setAddPlayersOpen] = useState(false);
  const [outcomeTarget, setOutcomeTarget] = useState<OutcomeTarget | null>(
    null,
  );
  const [removalTarget, setRemovalTarget] = useState<RemovalTarget | null>(
    null,
  );
  const roster = useQuery(academyClassQueryOptions(academyClass.id));
  const remove = useMutation({
    mutationFn: (target: RemovalTarget) =>
      removeAcademyMember(academyClass.id, target.member.playerUid),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: academyKeys.classes() }),
        queryClient.invalidateQueries({
          queryKey: academyKeys.academyClass(academyClass.id),
        }),
        queryClient.invalidateQueries({ queryKey: academyKeys.candidates() }),
      ]);
      setRemovalTarget(null);
    },
  });
  const members = roster.data?.members ?? [];
  const statistics = roster.data
    ? summarizeAcademyMembers(members)
    : unavailableAcademyStatistics();
  const statisticsStatus: AcademyStatisticsStatus = roster.isError
    ? "error"
    : roster.isPending
      ? "loading"
      : "ready";

  return (
    <>
      <Panel
        title={
          <div className="w-52">
            <h2 className="sr-only">Class workspace</h2>
            <SelectField
              label="Academy class"
              value={academyClass.id}
              className="[&_select]:text-headline-sm"
              onChange={(event) => {
                const selectedClass = academyClasses.find(
                  (item) => item.id === Number(event.target.value),
                );
                if (selectedClass) {
                  onSelectClass(selectedClass);
                }
              }}
            >
              {academyClasses.map((item) => (
                <option key={item.id} value={item.id}>
                  Class of {item.classYear}
                </option>
              ))}
            </SelectField>
          </div>
        }
        actions={
          <div className="flex items-center gap-2">
            <Button
              data-academy-add-players={academyClass.id}
              icon={UserPlus}
              onClick={() => setAddPlayersOpen(true)}
            >
              Add players
            </Button>
            {!academyClass.isAutomatic ? (
              <Button variant="destructive" icon={Trash2} onClick={onDelete}>
                Delete class
              </Button>
            ) : null}
          </div>
        }
      >
        <AcademyStatistics
          trackedPlayers={
            roster.data?.members.length ?? academyClass.memberCount
          }
          statistics={statistics}
          status={statisticsStatus}
        />
        {roster.isError ? (
          <p className="mt-6 text-body-sm text-error" role="alert">
            Could not load the class roster. {roster.error.message}
          </p>
        ) : null}
        {roster.isPending ? (
          <p className="mt-6 text-body-md text-on-surface-variant">
            Loading roster…
          </p>
        ) : null}
        {!roster.isPending && !roster.isError ? (
          <div className="mt-6 space-y-6">
            {rosterGroups.map((group) => (
              <AcademyRosterGroup
                key={group.id}
                academyClass={academyClass}
                groupId={group.id}
                title={group.title}
                members={members.filter(group.includes)}
                removePending={remove.isPending}
                onOpenOutcome={(member, mode, returnFocusTo) => {
                  setOutcomeTarget({ member, mode, returnFocusTo });
                }}
                onRemove={(member, returnFocusTo) => {
                  setRemovalTarget({ member, returnFocusTo });
                }}
              />
            ))}
          </div>
        ) : null}
      </Panel>
      <AcademyAddPlayersModal
        open={addPlayersOpen}
        academyClassId={academyClass.id}
        academyClassYear={academyClass.classYear}
        onClose={() => setAddPlayersOpen(false)}
      />
      <AcademyMemberOutcomeModal
        academyClassId={academyClass.id}
        target={outcomeTarget?.member ?? null}
        mode={outcomeTarget?.mode ?? null}
        clubOptions={clubOptions}
        returnFocusTo={outcomeTarget?.returnFocusTo}
        onClose={() => setOutcomeTarget(null)}
      />
      <AcademyMemberRemovalModal
        academyClass={academyClass}
        target={removalTarget}
        pending={remove.isPending}
        error={remove.isError ? errorMessage(remove.error) : undefined}
        onClose={() => {
          setRemovalTarget(null);
          remove.reset();
        }}
        onConfirm={() => {
          if (removalTarget) {
            remove.mutate(removalTarget);
          }
        }}
      />
    </>
  );
}

type AcademyRosterGroupProps = {
  academyClass: AcademyClass;
  groupId: string;
  title: string;
  members: AcademyMember[];
  removePending: boolean;
  onOpenOutcome: (
    member: AcademyMember,
    mode: AcademyMemberOutcomeMode,
    returnFocusTo: HTMLButtonElement | null,
  ) => void;
  onRemove: (
    member: AcademyMember,
    returnFocusTo: HTMLButtonElement | null,
  ) => void;
};

function AcademyRosterGroup({
  academyClass,
  groupId,
  title,
  members,
  removePending,
  onOpenOutcome,
  onRemove,
}: AcademyRosterGroupProps) {
  const headingId = `academy-roster-${groupId}`;

  return (
    <section aria-labelledby={headingId}>
      <h3 id={headingId} className="text-title-md text-on-surface">
        {title} ({members.length})
      </h3>
      <div className="mt-2 max-h-[min(55vh,560px)] overflow-auto rounded-lg border border-outline-variant">
        <table
          aria-labelledby={headingId}
          className="min-w-[1360px] w-full border-collapse text-left"
        >
          <caption className="sr-only">
            {title} players in Class of {academyClass.classYear}
          </caption>
          <AcademyRosterTableHeader />
          <tbody>
            {members.length > 0 ? (
              members.map((member) => (
                <AcademyRosterRow
                  key={member.playerUid}
                  academyClass={academyClass}
                  member={member}
                  removeDisabled={removePending}
                  onOpenOutcome={onOpenOutcome}
                  onRemove={onRemove}
                />
              ))
            ) : (
              <tr className="h-table-row border-t border-outline-variant">
                <td
                  colSpan={16}
                  className="px-2 text-body-sm text-on-surface-variant"
                >
                  No players are currently in this group.
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </section>
  );
}

function AcademyRosterTableHeader() {
  return (
    <thead className="sticky top-0 z-10 bg-surface-container-lowest">
      <tr>
        {[
          "Player",
          "Status",
          "Age",
          "Nationality",
          "Positions",
          "Club",
          "PA",
          "Determination",
          "Height",
          "Foot",
          "Career apps",
          "Goals",
          "Assists",
          "Caps",
          "Fee",
          "Actions",
        ].map((label) => (
          <th
            key={label || "actions"}
            scope="col"
            aria-label={label || "Actions"}
            className={`h-table-header-height px-2 text-label-md text-on-surface-variant uppercase ${
              [
                "Age",
                "PA",
                "Determination",
                "Height",
                "Career apps",
                "Goals",
                "Assists",
                "Caps",
                "Fee",
                "Actions",
              ].includes(label)
                ? "text-right"
                : "text-left"
            }`}
          >
            {label}
          </th>
        ))}
      </tr>
    </thead>
  );
}

type AcademyRosterRowProps = {
  academyClass: AcademyClass;
  member: AcademyMember;
  removeDisabled: boolean;
  onOpenOutcome: AcademyRosterGroupProps["onOpenOutcome"];
  onRemove: AcademyRosterGroupProps["onRemove"];
};

function AcademyRosterRow({
  academyClass,
  member,
  removeDisabled,
  onOpenOutcome,
  onRemove,
}: AcademyRosterRowProps) {
  const name = member.currentName ?? member.lastKnownName;
  const warning =
    member.state === "departed"
      ? "No longer in your club family"
      : member.state === "unresolved"
        ? "Unavailable in the current snapshot"
        : null;
  const positions = Object.keys(member.positions).join(", ");
  const rowTone =
    member.outcome?.status === "sold"
      ? "bg-success-container/20"
      : member.outcome?.status === "released"
        ? "bg-surface-container-low text-on-surface-variant"
        : "";

  return (
    <tr
      className={`h-table-row-height-two-line border-t border-outline-variant ${rowTone}`}
    >
      <td className="min-w-52 px-2 align-middle">
        <span
          className="block truncate text-body-sm text-on-surface"
          title={name}
        >
          {name}
        </span>
        {warning ? (
          <span
            className="flex items-center gap-1 whitespace-nowrap text-label-sm text-warning"
            role="status"
          >
            <TriangleAlert aria-hidden className="size-3.5 shrink-0" />
            {warning}
          </span>
        ) : null}
      </td>
      <td className="px-2 text-body-sm">
        <AcademyMemberStatus member={member} />
      </td>
      <td className="px-2 text-right text-body-sm tabular-nums">
        {formatMissable(member.age)}
      </td>
      <td
        className="max-w-36 truncate px-2 text-body-sm"
        title={member.nationalities.join(", ")}
      >
        {member.nationalities.join(", ") || "—"}
      </td>
      <td className="max-w-36 truncate px-2 text-body-sm" title={positions}>
        {positions || "—"}
      </td>
      <td
        className="max-w-44 truncate px-2 text-body-sm"
        title={member.currentClub ?? undefined}
      >
        {formatMissable(member.currentClub)}
      </td>
      <td className="px-2 text-right text-body-sm tabular-nums">
        {formatMissable(member.pa)}
      </td>
      <td className="px-2 text-right text-body-sm tabular-nums">
        {formatMissable(member.determination)}
      </td>
      <td className="px-2 text-right text-body-sm tabular-nums">
        {member.heightCm === null ? "—" : `${member.heightCm} cm`}
      </td>
      <td className="px-2 text-body-sm">
        {formatPreferredFoot(member.preferredFoot)}
      </td>
      <td className="px-2 text-right text-body-sm tabular-nums">
        {formatMissable(member.reportedCareerAppearances)}
      </td>
      <td className="px-2 text-right text-body-sm tabular-nums">
        {formatMissable(member.goals)}
      </td>
      <td className="px-2 text-right text-body-sm tabular-nums">
        {formatMissable(member.assists)}
      </td>
      <td className="px-2 text-right text-body-sm tabular-nums">
        {formatMissable(member.internationalCaps)}
      </td>
      <td className="px-2 text-right text-body-sm tabular-nums">
        {member.outcome?.status === "sold" && member.outcome.saleFeeEur !== null
          ? formatMoney(member.outcome.saleFeeEur)
          : "—"}
      </td>
      <td className="px-2 text-right">
        <AcademyMemberActions
          academyClass={academyClass}
          member={member}
          disabled={removeDisabled}
          onOpenOutcome={onOpenOutcome}
          onRemove={onRemove}
        />
      </td>
    </tr>
  );
}

function AcademyMemberStatus({ member }: { member: AcademyMember }) {
  if (member.outcome?.status === "sold") {
    return (
      <span className="inline-flex items-center gap-1 text-label-sm text-on-success-container">
        <CircleCheck aria-hidden size={14} strokeWidth={1.5} />
        Sold to {member.outcome.buyingClub}
      </span>
    );
  }
  if (member.outcome?.status === "released") {
    return (
      <span className="inline-flex items-center gap-1 text-label-sm text-on-surface-variant">
        <CircleMinus aria-hidden size={14} strokeWidth={1.5} />
        Released
      </span>
    );
  }
  return (
    <span className="text-label-sm text-on-surface-variant">Still at club</span>
  );
}

type AcademyMemberActionsProps = {
  academyClass: AcademyClass;
  member: AcademyMember;
  disabled: boolean;
  onOpenOutcome: AcademyRosterGroupProps["onOpenOutcome"];
  onRemove: AcademyRosterGroupProps["onRemove"];
};

function AcademyMemberActions({
  academyClass,
  member,
  disabled,
  onOpenOutcome,
  onRemove,
}: AcademyMemberActionsProps) {
  const released = member.outcome?.status === "released";

  return (
    <div className="inline-flex items-center justify-end gap-2 whitespace-nowrap">
      <Button
        data-academy-member-sell={`${academyClass.id}-${member.playerUid}`}
        disabled={disabled}
        icon={HandCoins}
        variant="secondary"
        className="h-7 border-success/60 px-3 text-success hover:bg-success/10 hover:text-success active:bg-success/15"
        onClick={(event) => onOpenOutcome(member, "sale", event.currentTarget)}
      >
        Sell
      </Button>
      <Button
        disabled={disabled}
        icon={released ? RotateCcw : UserRoundMinus}
        variant="secondary"
        className={
          released
            ? "h-7 px-3"
            : "h-7 border-warning/60 px-3 text-warning hover:bg-warning/10 hover:text-warning active:bg-warning/15"
        }
        onClick={(event) =>
          onOpenOutcome(
            member,
            released ? "clear" : "released",
            event.currentTarget,
          )
        }
      >
        {released ? "Restore" : "Release"}
      </Button>
      <Button
        disabled={disabled}
        icon={Trash2}
        variant="destructive"
        className="h-7 px-3"
        onClick={(event) => onRemove(member, event.currentTarget)}
      >
        Remove
      </Button>
    </div>
  );
}

function AcademyMemberRemovalModal({
  academyClass,
  target,
  pending,
  error,
  onClose,
  onConfirm,
}: {
  academyClass: AcademyClass;
  target: RemovalTarget | null;
  pending: boolean;
  error?: string;
  onClose: () => void;
  onConfirm: () => void;
}) {
  const [visibleTarget, setVisibleTarget] = useState(target);

  useEffect(() => {
    if (target) {
      setVisibleTarget(target);
    }
  }, [target]);

  if (!visibleTarget) {
    return null;
  }
  const name =
    visibleTarget.member.currentName ?? visibleTarget.member.lastKnownName;

  return (
    <Modal
      open={target !== null}
      title={`Remove ${name} from Class of ${academyClass.classYear}?`}
      variant="destructive"
      returnFocusTo={visibleTarget.returnFocusTo}
      fallbackFocusTo={() =>
        document.querySelector<HTMLButtonElement>(
          `[data-academy-add-players="${academyClass.id}"]`,
        )
      }
      onClose={() => {
        if (!pending) {
          onClose();
        }
      }}
      footer={
        <>
          <Button disabled={pending} variant="secondary" onClick={onClose}>
            Cancel
          </Button>
          <Button
            variant="destructive"
            loading={pending}
            loadingLabel="Removing…"
            onClick={onConfirm}
          >
            Remove from class
          </Button>
        </>
      }
    >
      <p className="text-body-md text-on-surface-variant">
        This removes {name} from the class and deletes any manual sale or
        release outcome. It does not affect other Academy classes.
      </p>
      {error ? (
        <p className="mt-3 text-body-sm text-error" role="alert">
          {error}
        </p>
      ) : null}
    </Modal>
  );
}
