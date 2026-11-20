using FmDataBridge.Extraction;
using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Scanning;

namespace FmDataBridge.Mutations;

/// <summary>One of a club's own teams: its object, FM team type, and squad list.</summary>
internal sealed record ClubTeam(ulong Address, int Type, HashSet<ulong> Squad)
{
    // FM27 squad lists hold the player block; accept the object address too.
    public bool Contains(PersonCandidate player) =>
        Squad.Contains(player.BlockAddress) || Squad.Contains(player.ObjectAddress);
}

/// <summary>Reads a club's teams and the database indexes FM's actions name people, clubs and teams by.</summary>
internal static class ClubTeams
{
    /// <summary>FM's database index fills 24 bits; an action's top byte carries the table.</summary>
    private const int MaxDatabaseIndex = (1 << 24) - 1;

    public static bool TryReadDatabaseIndex(IMemoryReader reader, ulong address, IFmMemoryLayout layout, out int index)
    {
        index = 0;
        if (!reader.TryReadUInt32(address + (ulong)layout.ObjectDatabaseIndexOffset, out var raw)
            || raw > MaxDatabaseIndex)
        {
            return false;
        }

        index = (int)raw;
        return true;
    }

    public static List<ClubTeam> Read(IMemoryReader reader, IFmMemoryLayout layout, ulong club)
    {
        var teams = new List<ClubTeam>();
        foreach (var team in SquadClubIndex.ReadPointerVector(
                     reader,
                     club,
                     layout.ClubTeamsBeginOffset,
                     layout.ClubTeamsEndOffset,
                     SquadClubIndex.MaxTeamsPerClub) ?? new List<ulong>())
        {
            if (!reader.TryReadByte(team + (ulong)layout.TeamTypeOffset, out var type))
            {
                continue;
            }

            var squad = SquadClubIndex.ReadPointerVector(
                reader,
                team,
                layout.TeamSquadBeginOffset,
                layout.TeamSquadEndOffset,
                SquadClubIndex.MaxPlayersPerSquad) ?? new List<ulong>();
            teams.Add(new ClubTeam(team, type, squad.ToHashSet()));
        }

        return teams;
    }

    /// <summary>
    /// The club's verified squad team at a level; with two, the one with more players, then the
    /// lower type. Unverified types such as Youth Candidates never receive anyone.
    /// </summary>
    public static ClubTeam? AtLevel(IReadOnlyList<ClubTeam> teams, string level) =>
        teams
            .Where(team => TeamLevelMap.IsSquadTeam(team.Type) && TeamLevelMap.FromTeamType(team.Type) == level)
            .OrderByDescending(team => team.Squad.Count)
            .ThenBy(team => team.Type)
            .FirstOrDefault();

    /// <summary>
    /// The human manager manages the club's first team, and FM's actions name him as the one acting.
    /// The team's manager pointer may name any part of his object: the vtable's type information
    /// says how far that part sits from the object's start, and his person record sits at one of the
    /// layout's person offsets, which its own type information confirms.
    /// </summary>
    public static bool TryReadHumanManagerIndex(
        IMemoryReader reader,
        IFmMemoryLayout layout,
        IReadOnlyList<ClubTeam> teams,
        out int managerIndex,
        out string detail)
    {
        managerIndex = 0;
        var firstTeam = teams.FirstOrDefault(team => team.Type == 0);
        if (firstTeam is null
            || !reader.TryReadUInt64(firstTeam.Address + (ulong)layout.TeamManagerPtrOffset, out var pointer)
            || pointer == 0)
        {
            detail = "the first team has no manager pointer";
            return false;
        }

        var start = pointer;
        var pointerOffset = 0;
        if (reader.TryReadUInt64(pointer, out var vtable)
            && PersonScanner.TryResolveDynamicOffset(reader, vtable, out pointerOffset)
            && pointerOffset > 0
            && (ulong)pointerOffset <= pointer)
        {
            start = pointer - (ulong)pointerOffset;
        }

        foreach (var recordOffset in layout.HumanManagerClassOffsets
                     .Concat(layout.StaffClassOffsets)
                     .Concat(layout.PlayerClassOffsets))
        {
            var record = start + (ulong)recordOffset;
            if (reader.TryReadUInt64(record, out var recordVtable)
                && PersonScanner.TryResolveDynamicOffset(reader, recordVtable, out var recordClassOffset)
                && recordClassOffset == recordOffset
                && reader.TryReadUInt32(record + (ulong)layout.ObjectUidOffset, out var uid)
                && PersonScanner.IsValidUid(uid)
                && TryReadDatabaseIndex(reader, record, layout, out managerIndex))
            {
                detail = "";
                return true;
            }
        }

        detail = $"no person record behind the manager pointer (pointer sub-object offset 0x{pointerOffset:X})";
        return false;
    }
}
