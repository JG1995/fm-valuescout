import type {
  AcademyClass,
  AcademyClassDetail,
  AcademyMember,
} from "../types/academy";

export type AcademyStatistics = {
  reportedSeniorPlayers: number | null;
  graduates: number | null;
  goals: number | null;
  assists: number | null;
  internationalCaps: number | null;
  saleFeeGbp: number | null;
  releasedPlayers: number | null;
};

export function unavailableAcademyStatistics(): AcademyStatistics {
  return {
    reportedSeniorPlayers: null,
    graduates: null,
    goals: null,
    assists: null,
    internationalCaps: null,
    saleFeeGbp: null,
    releasedPlayers: null,
  };
}

export function academyMemberIsGraduate(member: AcademyMember): boolean | null {
  return member.seniorLeagueAppearances === null
    ? null
    : member.seniorLeagueAppearances >= 1;
}

export function summarizeAcademyMembers(
  members: readonly AcademyMember[],
): AcademyStatistics {
  return {
    reportedSeniorPlayers: members.filter(
      (member) => member.state === "resolved" && member.teamLevel === "senior",
    ).length,
    graduates: completeCount(
      members,
      (member) => academyMemberIsGraduate(member) === true,
      (member) => member.seniorLeagueAppearances,
    ),
    goals: completeSum(members, (member) => member.goals),
    assists: completeSum(members, (member) => member.assists),
    internationalCaps: completeSum(
      members,
      (member) => member.internationalCaps,
    ),
    saleFeeGbp: completeSum(members, (member) => member.saleFeeGbp),
    releasedPlayers: completeCount(
      members,
      (member) => member.isReleased === true,
      (member) => member.isReleased,
    ),
  };
}

export function academyDetailsAreComplete(
  classes: readonly AcademyClass[],
  details: readonly AcademyClassDetail[],
): boolean {
  if (classes.length !== details.length) {
    return false;
  }

  return classes.every((academyClass) => {
    const detail = details.find(
      (candidate) => candidate.id === academyClass.id,
    );
    return detail?.members.length === academyClass.memberCount;
  });
}

function completeSum(
  members: readonly AcademyMember[],
  value: (member: AcademyMember) => number | null,
): number | null {
  if (members.length === 0) {
    return null;
  }

  let total = 0;
  for (const member of members) {
    const item = value(member);
    if (item === null) {
      return null;
    }
    total += item;
  }
  return total;
}

function completeCount(
  members: readonly AcademyMember[],
  predicate: (member: AcademyMember) => boolean,
  value: (member: AcademyMember) => number | boolean | null,
): number | null {
  if (
    members.length === 0 ||
    members.some((member) => value(member) === null)
  ) {
    return null;
  }

  return members.filter(predicate).length;
}
