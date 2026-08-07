import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  CircleCheck,
  CircleMinus,
  Ellipsis,
  Trash2,
  TriangleAlert,
  UserPlus,
} from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button/button";
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
  clubOptions: string[];
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
  clubOptions,
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
        title={`Class of ${academyClass.classYear}`}
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
                  colSpan={17}
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
          "Reported team",
          "PA",
          "Determination",
          "Height",
          "Foot",
          "Senior league apps",
          "Goals",
          "Assists",
          "Caps",
          "Sale fee",
          "",
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
                "Senior league apps",
                "Goals",
                "Assists",
                "Caps",
                "Sale fee",
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
  const reportedTeam = [member.teamLevel, member.parentClub]
    .filter((value): value is string => Boolean(value))
    .join(" · ");
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
      <td
        className="max-w-44 truncate px-2 text-body-sm"
        title={reportedTeam || undefined}
      >
        {reportedTeam || "—"}
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
        {formatMissable(member.seniorLeagueAppearances)}
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
  const triggerRef = useRef<HTMLButtonElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const [open, setOpen] = useState(false);
  const name = member.currentName ?? member.lastKnownName;
  const closeAndRestoreFocus = () => {
    setOpen(false);
    triggerRef.current?.focus();
  };
  const moveMenuFocus = (direction: 1 | -1) => {
    const items = Array.from(
      menuRef.current?.querySelectorAll<HTMLButtonElement>(
        '[role="menuitem"]',
      ) ?? [],
    );
    const currentIndex = items.indexOf(
      document.activeElement as HTMLButtonElement,
    );
    if (currentIndex === -1) {
      items[direction === 1 ? 0 : items.length - 1]?.focus();
      return;
    }
    const nextIndex = (currentIndex + direction + items.length) % items.length;
    items[nextIndex]?.focus();
  };
  const openOutcome = (mode: AcademyMemberOutcomeMode) => {
    setOpen(false);
    onOpenOutcome(member, mode, triggerRef.current);
  };

  useEffect(() => {
    if (open) {
      menuRef.current
        ?.querySelector<HTMLButtonElement>('[role="menuitem"]')
        ?.focus();
    }
  }, [open]);

  return (
    <div className="relative inline-flex">
      <button
        ref={triggerRef}
        type="button"
        aria-label={`Manage ${name} in Class of ${academyClass.classYear}`}
        data-academy-member-actions={`${academyClass.id}-${member.playerUid}`}
        aria-expanded={open}
        aria-haspopup="menu"
        className="inline-flex size-8 cursor-pointer items-center justify-center rounded-md text-on-surface-variant transition-colors duration-150 ease-out hover:bg-surface-container-high hover:text-on-surface focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary disabled:cursor-not-allowed disabled:opacity-45"
        disabled={disabled}
        onContextMenu={(event) => {
          event.preventDefault();
          setOpen(true);
        }}
        onClick={() => setOpen((current) => !current)}
        onKeyDown={(event) => {
          if (event.key === "ArrowDown") {
            event.preventDefault();
            setOpen(true);
          } else if (open && event.key === "Escape") {
            event.preventDefault();
            closeAndRestoreFocus();
          }
        }}
      >
        <Ellipsis aria-hidden size={16} strokeWidth={1.5} />
      </button>
      {open ? (
        <div
          ref={menuRef}
          role="menu"
          aria-label={`${name} actions`}
          className="absolute right-0 top-full z-20 mt-1 w-48 rounded-md border border-outline-variant bg-surface-container-highest p-1 text-left shadow-overlay"
          onKeyDown={(event) => {
            if (event.key === "Escape") {
              event.preventDefault();
              closeAndRestoreFocus();
            } else if (event.key === "ArrowDown") {
              event.preventDefault();
              moveMenuFocus(1);
            } else if (event.key === "ArrowUp") {
              event.preventDefault();
              moveMenuFocus(-1);
            } else if (event.key === "Home") {
              event.preventDefault();
              menuRef.current
                ?.querySelector<HTMLButtonElement>('[role="menuitem"]')
                ?.focus();
            } else if (event.key === "End") {
              event.preventDefault();
              const items =
                menuRef.current?.querySelectorAll<HTMLButtonElement>(
                  '[role="menuitem"]',
                );
              items?.[items.length - 1]?.focus();
            }
          }}
        >
          {member.outcome?.status === "sold" ? (
            <AcademyActionMenuItem onClick={() => openOutcome("sale")}>
              Edit sale
            </AcademyActionMenuItem>
          ) : (
            <AcademyActionMenuItem onClick={() => openOutcome("sale")}>
              Record sale
            </AcademyActionMenuItem>
          )}
          {member.outcome?.status !== "released" ? (
            <AcademyActionMenuItem onClick={() => openOutcome("released")}>
              Mark released
            </AcademyActionMenuItem>
          ) : null}
          {member.outcome ? (
            <AcademyActionMenuItem onClick={() => openOutcome("clear")}>
              Restore to still at club
            </AcademyActionMenuItem>
          ) : null}
          <AcademyActionMenuItem
            destructive
            onClick={() => {
              setOpen(false);
              onRemove(member, triggerRef.current);
            }}
          >
            Remove from class
          </AcademyActionMenuItem>
        </div>
      ) : null}
    </div>
  );
}

function AcademyActionMenuItem({
  children,
  destructive = false,
  onClick,
}: {
  children: string;
  destructive?: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      role="menuitem"
      className={`flex w-full cursor-pointer items-center rounded-sm px-3 py-2 text-left text-label-md hover:bg-surface-container-high focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary ${
        destructive ? "text-error" : "text-on-surface"
      }`}
      onClick={onClick}
    >
      {children}
    </button>
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
