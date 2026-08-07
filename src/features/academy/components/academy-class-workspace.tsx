import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Trash2, TriangleAlert, UserPlus } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button/button";
import { Panel } from "@/components/ui/panel/panel";
import { formatMissable, formatPreferredFoot } from "@/utils/format";
import { academyClassQueryOptions } from "../api/academy-class-query-options";
import { academyKeys } from "../api/academy-keys";
import { removeAcademyMember } from "../api/remove-academy-member";
import type { AcademyClass, AcademyMember } from "../types/academy";
import { AcademyAddPlayersModal } from "./academy-add-players-modal";

type AcademyClassWorkspaceProps = {
  academyClass: AcademyClass;
  onDelete: () => void;
};

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

export function AcademyClassWorkspace({
  academyClass,
  onDelete,
}: AcademyClassWorkspaceProps) {
  const queryClient = useQueryClient();
  const [addPlayersOpen, setAddPlayersOpen] = useState(false);
  const roster = useQuery(academyClassQueryOptions(academyClass.id));
  const remove = useMutation({
    mutationFn: (member: AcademyMember) =>
      removeAcademyMember(academyClass.id, member.playerUid),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: academyKeys.classes() }),
        queryClient.invalidateQueries({
          queryKey: academyKeys.academyClass(academyClass.id),
        }),
        queryClient.invalidateQueries({ queryKey: academyKeys.candidates() }),
      ]);
    },
  });
  const members = roster.data?.members ?? [];

  return (
    <>
      <Panel
        title={`Class of ${academyClass.classYear}`}
        actions={
          <div className="flex items-center gap-2">
            <Button icon={UserPlus} onClick={() => setAddPlayersOpen(true)}>
              Add players
            </Button>
            <Button variant="destructive" icon={Trash2} onClick={onDelete}>
              Delete class
            </Button>
          </div>
        }
      >
        <dl>
          <div>
            <dt className="text-label-md text-on-surface-variant">
              Tracked players
            </dt>
            <dd className="mt-1 text-headline-md text-on-surface">
              {roster.data?.members.length ?? academyClass.memberCount}
            </dd>
          </div>
        </dl>
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
        {!roster.isPending && !roster.isError && members.length === 0 ? (
          <p className="mt-6 text-body-md text-on-surface-variant">
            No players are tracked in this class yet.
          </p>
        ) : null}
        {members.length > 0 ? (
          <div className="mt-6 max-h-[min(55vh,560px)] overflow-auto rounded-lg border border-outline-variant">
            <table className="min-w-[960px] w-full border-collapse text-left">
              <caption className="sr-only">
                Players in Class of {academyClass.classYear}
              </caption>
              <thead className="sticky top-0 z-10 bg-surface-container-lowest">
                <tr>
                  {[
                    "Player",
                    "Age",
                    "Nationality",
                    "Positions",
                    "Club",
                    "Reported team",
                    "PA",
                    "Determination",
                    "Height",
                    "Foot",
                    "",
                  ].map((label) => (
                    <th
                      key={label || "actions"}
                      scope="col"
                      aria-label={label || "Actions"}
                      className={`h-table-header-height px-2 text-label-md text-on-surface-variant uppercase ${
                        ["Age", "PA", "Determination", "Height"].includes(label)
                          ? "text-right"
                          : "text-left"
                      }`}
                    >
                      {label}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {members.map((member) => (
                  <AcademyRosterRow
                    key={member.playerUid}
                    academyClass={academyClass}
                    member={member}
                    error={
                      remove.isError &&
                      remove.variables?.playerUid === member.playerUid
                        ? errorMessage(remove.error)
                        : undefined
                    }
                    removing={
                      remove.isPending &&
                      remove.variables?.playerUid === member.playerUid
                    }
                    removeDisabled={remove.isPending}
                    onRemove={() => remove.mutate(member)}
                  />
                ))}
              </tbody>
            </table>
          </div>
        ) : null}
      </Panel>
      <AcademyAddPlayersModal
        open={addPlayersOpen}
        academyClassId={academyClass.id}
        academyClassYear={academyClass.classYear}
        onClose={() => setAddPlayersOpen(false)}
      />
    </>
  );
}

type AcademyRosterRowProps = {
  academyClass: AcademyClass;
  member: AcademyMember;
  error?: string;
  removing: boolean;
  removeDisabled: boolean;
  onRemove: () => void;
};

function AcademyRosterRow({
  academyClass,
  member,
  error,
  removing,
  removeDisabled,
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

  return (
    <tr className="h-table-row-height-two-line border-t border-outline-variant">
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
      <td className="px-2 text-right">
        <div className="flex flex-col items-end gap-1">
          <Button
            variant="secondary"
            disabled={removeDisabled}
            loading={removing}
            loadingLabel="Removing…"
            aria-describedby={
              error ? `academy-remove-error-${member.playerUid}` : undefined
            }
            aria-label={`Remove ${name} from Class of ${academyClass.classYear}`}
            onClick={onRemove}
          >
            Remove
          </Button>
          {error ? (
            <p
              id={`academy-remove-error-${member.playerUid}`}
              className="max-w-48 text-right text-body-sm text-error"
              role="alert"
            >
              {error}
            </p>
          ) : null}
        </div>
      </td>
    </tr>
  );
}
