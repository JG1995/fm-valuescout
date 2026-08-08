using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Scanning;

namespace FmDataBridge.Extraction;

public sealed class SquadAssignment
{
    public string ClubName { get; init; } = "";

    public int TeamType { get; init; }

    /// <summary>Raw reputation of the selected squad team; null when unread or invalid.</summary>
    public int? TeamReputation { get; init; }

    public string? Division { get; init; }
}

/// <summary>One discovered team assignment for a pinned human-manager person.</summary>
public sealed class HumanManagerClubAssignment
{
    public string ClubName { get; init; } = "";

    public int? TeamType { get; init; }

    public int? TeamReputation { get; init; }
}

/// <summary>
/// Walk club → teams → squads and assign current club with deterministic multi-hit rules.
/// </summary>
public sealed class SquadClubIndex
{
    public const int MaxMultiClubSamples = 25;
    public const int MaxTeamsPerClub = 24;
    public const int MaxPlayersPerSquad = 60;

    private readonly Dictionary<uint, SquadAssignment> _assignments = new();
    private readonly Dictionary<ulong, HumanManagerHit> _humanManagerAssignments = new();

    public IReadOnlyDictionary<uint, int> DateVotes { get; private set; } =
        new Dictionary<uint, int>();

    public int ClubsWalked { get; private set; }

    public int PlayersLinked { get; private set; }

    public List<string> MultiClubSamples { get; } = new();

    public HashSet<uint> MultiClubUids { get; } = new();

    public static SquadClubIndex Build(
        IMemoryReader reader,
        IFmMemoryLayout layout,
        IReadOnlyDictionary<ulong, uint> personToUid,
        IReadOnlyDictionary<uint, string?> parentClubByUid,
        IEnumerable<ClubCandidate> discoveredClubs,
        IEnumerable<ulong> fallbackClubAddresses,
        IEnumerable<ulong>? humanManagerAddresses = null)
    {
        ArgumentNullException.ThrowIfNull(reader);
        ArgumentNullException.ThrowIfNull(layout);
        ArgumentNullException.ThrowIfNull(personToUid);
        ArgumentNullException.ThrowIfNull(parentClubByUid);
        ArgumentNullException.ThrowIfNull(discoveredClubs);
        ArgumentNullException.ThrowIfNull(fallbackClubAddresses);

        var index = new SquadClubIndex();
        var votes = new Dictionary<uint, int>();
        var seenClubs = new HashSet<ulong>();
        var managers = humanManagerAddresses is null
            ? new HashSet<ulong>()
            : humanManagerAddresses.Where(address => address != 0).ToHashSet();

        foreach (var discovered in discoveredClubs.OrderBy(candidate => candidate.Address))
        {
            if (discovered.Address == 0
                || !ClubNamePlausibility.IsPlausible(discovered.Name)
                || !seenClubs.Add(discovered.Address))
            {
                continue;
            }

            WalkClub(
                reader,
                layout,
                discovered.Address,
                discovered.Name,
                personToUid,
                parentClubByUid,
                managers,
                votes,
                index);
        }

        foreach (var club in fallbackClubAddresses.OrderBy(address => address))
        {
            if (club == 0 || !seenClubs.Add(club))
            {
                continue;
            }

            var clubName = ClubNameReader.TryRead(reader, club, layout);
            if (clubName is null)
            {
                continue;
            }

            WalkClub(
                reader,
                layout,
                club,
                clubName,
                personToUid,
                parentClubByUid,
                managers,
                votes,
                index);
        }

        index.DateVotes = votes;
        index.PlayersLinked = index._assignments.Count;
        return index;
    }

    public bool TryGet(uint uid, out SquadAssignment assignment) =>
        _assignments.TryGetValue(uid, out assignment!);

    public bool TryGetHumanManager(
        ulong personAddress,
        out HumanManagerClubAssignment assignment)
    {
        if (_humanManagerAssignments.TryGetValue(personAddress, out var hit))
        {
            assignment = new HumanManagerClubAssignment
            {
                ClubName = hit.ClubName,
                TeamType = hit.TeamType,
                TeamReputation = hit.TeamReputation,
            };
            return true;
        }

        assignment = null!;
        return false;
    }

    private static void WalkClub(
        IMemoryReader reader,
        IFmMemoryLayout layout,
        ulong club,
        string clubName,
        IReadOnlyDictionary<ulong, uint> personToUid,
        IReadOnlyDictionary<uint, string?> parentClubByUid,
        IReadOnlySet<ulong> humanManagerAddresses,
        Dictionary<uint, int> votes,
        SquadClubIndex index)
    {
        if (!TryReadPointerAt(reader, club, layout.ClubTeamsBeginOffset, out var teamsBegin)
            || !TryReadPointerAt(reader, club, layout.ClubTeamsEndOffset, out var teamsEnd)
            || teamsBegin == 0
            || teamsEnd <= teamsBegin
            || teamsBegin % sizeof(ulong) != 0
            || teamsEnd % sizeof(ulong) != 0
            || (teamsEnd - teamsBegin) % sizeof(ulong) != 0)
        {
            return;
        }

        var teamCount = (teamsEnd - teamsBegin) / sizeof(ulong);
        if (teamCount is 0 or > MaxTeamsPerClub)
        {
            return;
        }

        index.ClubsWalked++;
        for (ulong ti = 0; ti < teamCount; ti++)
        {
            if (!TryReadVectorEntry(reader, teamsBegin, ti, out var team) || team == 0)
            {
                continue;
            }

            int? discoveredTeamType = null;
            if (TryReadByteAt(reader, team, layout.TeamTypeOffset, out var tt))
            {
                discoveredTeamType = tt;
            }

            var teamType = discoveredTeamType ?? 0;

            int? teamReputation = null;
            if (TryReadUInt16At(reader, team, layout.TeamReputationOffset, out var reputation)
                && reputation <= 12000)
            {
                teamReputation = reputation;
            }

            if (TryReadPointerAt(reader, team, layout.TeamManagerPtrOffset, out var managerAddress)
                && humanManagerAddresses.Contains(managerAddress))
            {
                index.RecordHumanManager(
                    managerAddress,
                    new HumanManagerHit(
                        clubName,
                        discoveredTeamType,
                        teamReputation,
                        team));
            }

            RecordDateVote(reader, layout, team, votes);
            var division = CompetitionNameReader.TryRead(reader, team, layout);
            WalkSquad(
                reader,
                layout,
                team,
                clubName,
                teamType,
                teamReputation,
                division,
                personToUid,
                parentClubByUid,
                index);
        }
    }

    private static void WalkSquad(
        IMemoryReader reader,
        IFmMemoryLayout layout,
        ulong team,
        string clubName,
        int teamType,
        int? teamReputation,
        string? division,
        IReadOnlyDictionary<ulong, uint> personToUid,
        IReadOnlyDictionary<uint, string?> parentClubByUid,
        SquadClubIndex index)
    {
        if (!TryReadPointerAt(reader, team, layout.TeamSquadBeginOffset, out var squadBegin)
            || !TryReadPointerAt(reader, team, layout.TeamSquadEndOffset, out var squadEnd)
            || squadBegin == 0
            || squadEnd <= squadBegin
            || squadBegin % sizeof(ulong) != 0
            || squadEnd % sizeof(ulong) != 0
            || (squadEnd - squadBegin) % sizeof(ulong) != 0)
        {
            return;
        }

        var count = (squadEnd - squadBegin) / sizeof(ulong);
        if (count is 0 or > MaxPlayersPerSquad)
        {
            return;
        }

        for (ulong pi = 0; pi < count; pi++)
        {
            if (!TryReadVectorEntry(reader, squadBegin, pi, out var entry) || entry == 0)
            {
                continue;
            }

            if (!TryResolvePersonUid(reader, entry, personToUid, out var uid))
            {
                continue;
            }

            parentClubByUid.TryGetValue(uid, out var parent);
            var candidate = new SquadHit(clubName, teamType, division, teamReputation);
            if (!index._assignments.TryGetValue(uid, out var cur))
            {
                index._assignments[uid] = ToAssignment(candidate);
                continue;
            }

            var currentHit = new SquadHit(
                cur.ClubName,
                cur.TeamType,
                cur.Division,
                cur.TeamReputation);
            var chosen = SquadPick.Choose(currentHit, candidate, parent);
            if (currentHit.ClubName != candidate.ClubName
                && index.MultiClubUids.Add(uid)
                && index.MultiClubSamples.Count < MaxMultiClubSamples)
            {
                var won = chosen.ClubName;
                var lost = won == candidate.ClubName ? currentHit.ClubName : candidate.ClubName;
                index.MultiClubSamples.Add(
                    $"uid={uid} plays={won} alsoIn={lost} parent={parent ?? "-"}");
            }

            index._assignments[uid] = ToAssignment(chosen);
        }
    }

    private static SquadAssignment ToAssignment(SquadHit hit) =>
        new()
        {
            ClubName = hit.ClubName,
            TeamType = hit.TeamType,
            TeamReputation = hit.TeamReputation,
            Division = hit.Division,
        };

    private void RecordHumanManager(ulong personAddress, HumanManagerHit candidate)
    {
        if (_humanManagerAssignments.TryGetValue(personAddress, out var current)
            && !HumanManagerPick.Choose(candidate, current))
        {
            return;
        }

        _humanManagerAssignments[personAddress] = candidate;
    }

    private static bool TryResolvePersonUid(
        IMemoryReader reader,
        ulong entry,
        IReadOnlyDictionary<ulong, uint> personToUid,
        out uint uid)
    {
        if (personToUid.TryGetValue(entry, out uid))
        {
            return true;
        }

        // Squad wrappers: probe pointer-sized fields for a known person address.
        for (var off = 0; off <= 0x80; off += 8)
        {
            if (!TryReadPointerAt(reader, entry, off, out var q) || q == 0)
            {
                continue;
            }

            if (personToUid.TryGetValue(q, out uid))
            {
                return true;
            }
        }

        uid = 0;
        return false;
    }

    private static void RecordDateVote(
        IMemoryReader reader,
        IFmMemoryLayout layout,
        ulong team,
        Dictionary<uint, int> votes)
    {
        if (!TryReadPointerAt(reader, team, layout.TeamSchedulePtrOffset, out var schedule)
            || schedule == 0)
        {
            return;
        }

        foreach (var so in new[] { layout.ScheduleNextMatchOffset, layout.ScheduleNextMatchAltOffset })
        {
            if (!TryReadUInt32At(reader, schedule, so, out var raw))
            {
                continue;
            }

            var (year, doy) = FmDateDecoder.Decode(raw);
            if (year is < 2020 or > 2060 || !FmDateDecoder.IsPlausible(year, doy))
            {
                continue;
            }

            var norm = ((uint)year << 16) | (uint)doy;
            votes.TryGetValue(norm, out var n);
            votes[norm] = n + 1;
            return;
        }
    }

    private static bool TryReadPointerAt(
        IMemoryReader reader,
        ulong address,
        int offset,
        out ulong value)
    {
        value = 0;
        return TryAdd(address, offset, out var fieldAddress)
            && reader.TryReadUInt64(fieldAddress, out value);
    }

    private static bool TryReadByteAt(
        IMemoryReader reader,
        ulong address,
        int offset,
        out byte value)
    {
        value = 0;
        return TryAdd(address, offset, out var fieldAddress)
            && reader.TryReadByte(fieldAddress, out value);
    }

    private static bool TryReadUInt32At(
        IMemoryReader reader,
        ulong address,
        int offset,
        out uint value)
    {
        value = 0;
        return TryAdd(address, offset, out var fieldAddress)
            && reader.TryReadUInt32(fieldAddress, out value);
    }

    private static bool TryReadUInt16At(
        IMemoryReader reader,
        ulong address,
        int offset,
        out ushort value)
    {
        value = 0;
        return TryAdd(address, offset, out var fieldAddress)
            && reader.TryReadUInt16(fieldAddress, out value);
    }

    private static bool TryReadVectorEntry(
        IMemoryReader reader,
        ulong begin,
        ulong index,
        out ulong value)
    {
        value = 0;
        return TryMultiply(index, sizeof(ulong), out var byteOffset)
            && TryAdd(begin, byteOffset, out var entryAddress)
            && reader.TryReadUInt64(entryAddress, out value);
    }

    private static bool TryAdd(ulong address, int offset, out ulong result)
    {
        result = 0;
        return offset >= 0 && TryAdd(address, (ulong)offset, out result);
    }

    private static bool TryAdd(ulong address, ulong offset, out ulong result)
    {
        result = 0;
        if (offset > ulong.MaxValue - address)
        {
            return false;
        }

        result = address + offset;
        return true;
    }

    private static bool TryMultiply(ulong value, ulong multiplier, out ulong result)
    {
        result = 0;
        if (value != 0 && multiplier > ulong.MaxValue / value)
        {
            return false;
        }

        result = value * multiplier;
        return true;
    }

    private readonly record struct HumanManagerHit(
        string ClubName,
        int? TeamType,
        int? TeamReputation,
        ulong TeamAddress);

    private static class HumanManagerPick
    {
        public static bool Choose(HumanManagerHit candidate, HumanManagerHit current)
        {
            var candidateType = candidate.TeamType ?? int.MaxValue;
            var currentType = current.TeamType ?? int.MaxValue;
            if (candidateType != currentType)
            {
                return candidateType < currentType;
            }

            var clubOrder = string.CompareOrdinal(candidate.ClubName, current.ClubName);
            return clubOrder < 0
                || (clubOrder == 0 && candidate.TeamAddress < current.TeamAddress);
        }
    }
}
