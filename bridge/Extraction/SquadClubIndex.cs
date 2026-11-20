using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Scanning;

namespace FmDataBridge.Extraction;

public sealed class SquadAssignment
{
    public string ClubName { get; init; } = "";

    public uint? ClubUid { get; init; }

    public int TeamType { get; init; }

    /// <summary>Raw reputation of the selected squad team; null when unread or invalid.</summary>
    public int? TeamReputation { get; init; }

    public string? Division { get; init; }

    /// <summary>The squad belongs to a separate B club and counts as the parent's reserve team.</summary>
    public bool ReserveClub { get; init; }
}

/// <summary>One discovered team assignment for a pinned human-manager person.</summary>
public sealed class HumanManagerClubAssignment
{
    public string ClubName { get; init; } = "";

    public uint? ClubUid { get; init; }

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
    public const int MaxReserveClubsPerClub = 8;

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

        var clubs = new List<(ulong Address, string Name)>();
        foreach (var discovered in discoveredClubs.OrderBy(candidate => candidate.Address))
        {
            if (discovered.Address != 0
                && ClubNamePlausibility.IsPlausible(discovered.Name)
                && seenClubs.Add(discovered.Address))
            {
                clubs.Add((discovered.Address, discovered.Name));
            }
        }

        foreach (var club in fallbackClubAddresses.OrderBy(address => address))
        {
            if (club == 0 || !seenClubs.Add(club))
            {
                continue;
            }

            var clubName = ClubNameReader.TryRead(reader, club, layout);
            if (clubName is not null)
            {
                clubs.Add((club, clubName));
            }
        }

        // A B club's first team counts as its parent club's reserve team.
        var reserveParents = new Dictionary<ulong, SquadOwner>();
        foreach (var (club, clubName) in clubs)
        {
            var owner = new SquadOwner(clubName, TryReadPositiveUid(reader, club, layout.ObjectUidOffset), ReserveClub: true);
            foreach (var team in ReadPointerVector(
                         reader,
                         club,
                         layout.ClubReserveTeamsBeginOffset,
                         layout.ClubReserveTeamsEndOffset,
                         MaxReserveClubsPerClub) ?? new List<ulong>())
            {
                reserveParents.TryAdd(team, owner);
            }
        }

        var walkedTeams = new HashSet<ulong>();
        foreach (var (club, clubName) in clubs)
        {
            WalkClub(
                reader,
                layout,
                club,
                clubName,
                personToUid,
                parentClubByUid,
                managers,
                reserveParents,
                walkedTeams,
                votes,
                index);
        }

        // B clubs that no contract or discovery reached are still walked through their parent.
        foreach (var (team, owner) in reserveParents.OrderBy(entry => entry.Key))
        {
            if (walkedTeams.Add(team))
            {
                WalkTeam(
                    reader,
                    layout,
                    team,
                    owner,
                    personToUid,
                    parentClubByUid,
                    new HashSet<ulong>(),
                    votes,
                    index);
            }
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
                ClubUid = hit.ClubUid,
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
        IReadOnlyDictionary<ulong, SquadOwner> reserveParents,
        HashSet<ulong> walkedTeams,
        Dictionary<uint, int> votes,
        SquadClubIndex index)
    {
        var teams = ReadPointerVector(
            reader,
            club,
            layout.ClubTeamsBeginOffset,
            layout.ClubTeamsEndOffset,
            MaxTeamsPerClub);
        if (teams is null)
        {
            return;
        }

        index.ClubsWalked++;
        var owner = new SquadOwner(clubName, TryReadPositiveUid(reader, club, layout.ObjectUidOffset), ReserveClub: false);
        foreach (var team in teams)
        {
            if (walkedTeams.Add(team))
            {
                WalkTeam(
                    reader,
                    layout,
                    team,
                    reserveParents.TryGetValue(team, out var parent) ? parent : owner,
                    personToUid,
                    parentClubByUid,
                    humanManagerAddresses,
                    votes,
                    index);
            }
        }
    }

    private static void WalkTeam(
        IMemoryReader reader,
        IFmMemoryLayout layout,
        ulong team,
        SquadOwner owner,
        IReadOnlyDictionary<ulong, uint> personToUid,
        IReadOnlyDictionary<uint, string?> parentClubByUid,
        IReadOnlySet<ulong> humanManagerAddresses,
        Dictionary<uint, int> votes,
        SquadClubIndex index)
    {
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

        if (!owner.ReserveClub
            && TryReadPointerAt(reader, team, layout.TeamManagerPtrOffset, out var managerAddress)
            && humanManagerAddresses.Contains(managerAddress))
        {
            index.RecordHumanManager(
                managerAddress,
                new HumanManagerHit(
                    owner.ClubName,
                    owner.ClubUid,
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
            owner,
            teamType,
            teamReputation,
            division,
            personToUid,
            parentClubByUid,
            index);
    }

    /// <summary>A begin/end pointer vector of non-null object pointers, or null when invalid.</summary>
    private static List<ulong>? ReadPointerVector(
        IMemoryReader reader,
        ulong owner,
        int beginOffset,
        int endOffset,
        int maxCount)
    {
        if (!TryReadPointerAt(reader, owner, beginOffset, out var begin)
            || !TryReadPointerAt(reader, owner, endOffset, out var end)
            || begin == 0
            || end <= begin
            || begin % sizeof(ulong) != 0
            || end % sizeof(ulong) != 0)
        {
            return null;
        }

        var count = (end - begin) / sizeof(ulong);
        if (count > (ulong)maxCount)
        {
            return null;
        }

        var entries = new List<ulong>((int)count);
        for (ulong i = 0; i < count; i++)
        {
            if (TryReadVectorEntry(reader, begin, i, out var entry) && entry != 0)
            {
                entries.Add(entry);
            }
        }

        return entries;
    }

    private static void WalkSquad(
        IMemoryReader reader,
        IFmMemoryLayout layout,
        ulong team,
        SquadOwner owner,
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
            var candidate = new SquadHit(
                owner.ClubName,
                teamType,
                division,
                teamReputation,
                owner.ClubUid,
                owner.ReserveClub);
            if (!index._assignments.TryGetValue(uid, out var cur))
            {
                index._assignments[uid] = ToAssignment(candidate);
                continue;
            }

            var currentHit = new SquadHit(
                cur.ClubName,
                cur.TeamType,
                cur.Division,
                cur.TeamReputation,
                cur.ClubUid,
                cur.ReserveClub);
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
            ClubUid = hit.ClubUid,
            TeamType = hit.TeamType,
            TeamReputation = hit.TeamReputation,
            Division = hit.Division,
            ReserveClub = hit.ReserveClub,
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

    private static uint? TryReadPositiveUid(IMemoryReader reader, ulong address, int offset)
    {
        return TryAdd(address, offset, out var fieldAddress)
            && reader.TryReadUInt32(fieldAddress, out var uid)
            && uid > 0
            ? uid
            : null;
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

    /// <summary>The club a team's squad counts for; a B club's squad counts for its parent.</summary>
    private readonly record struct SquadOwner(string ClubName, uint? ClubUid, bool ReserveClub);

    private readonly record struct HumanManagerHit(
        string ClubName,
        uint? ClubUid,
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
