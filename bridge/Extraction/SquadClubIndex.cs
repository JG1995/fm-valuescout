using FmDataBridge.Layouts;
using FmDataBridge.Memory;

namespace FmDataBridge.Extraction;

public sealed class SquadAssignment
{
    public string ClubName { get; init; } = "";

    public int TeamType { get; init; }

    public string? Division { get; init; }
}

/// <summary>
/// Walk club → teams → squads and assign current club with deterministic multi-hit rules.
/// </summary>
public sealed class SquadClubIndex
{
    public const int MaxMultiClubSamples = 25;

    private readonly Dictionary<uint, SquadAssignment> _assignments = new();

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
        IEnumerable<ulong> clubAddresses)
    {
        ArgumentNullException.ThrowIfNull(reader);
        ArgumentNullException.ThrowIfNull(layout);
        ArgumentNullException.ThrowIfNull(personToUid);
        ArgumentNullException.ThrowIfNull(parentClubByUid);

        var index = new SquadClubIndex();
        var votes = new Dictionary<uint, int>();
        var seenClubs = new HashSet<ulong>();

        foreach (var club in clubAddresses)
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

            if (!reader.TryReadUInt64(club + (ulong)layout.ClubTeamsBeginOffset, out var teamsBegin)
                || !reader.TryReadUInt64(club + (ulong)layout.ClubTeamsEndOffset, out var teamsEnd)
                || teamsBegin == 0
                || teamsEnd <= teamsBegin
                || (teamsEnd - teamsBegin) % 8 != 0)
            {
                continue;
            }

            var teamCount = (long)((teamsEnd - teamsBegin) / 8);
            if (teamCount is <= 0 or > 24)
            {
                continue;
            }

            index.ClubsWalked++;

            for (long ti = 0; ti < teamCount; ti++)
            {
                if (!reader.TryReadUInt64(teamsBegin + (ulong)(ti * 8), out var team) || team == 0)
                {
                    continue;
                }

                var teamType = 0;
                if (reader.TryReadByte(team + (ulong)layout.TeamTypeOffset, out var tt))
                {
                    teamType = tt;
                }

                RecordDateVote(reader, layout, team, votes);

                var division = CompetitionNameReader.TryRead(reader, team, layout);
                WalkSquad(
                    reader,
                    layout,
                    team,
                    clubName,
                    teamType,
                    division,
                    personToUid,
                    parentClubByUid,
                    index);
            }
        }

        index.DateVotes = votes;
        index.PlayersLinked = index._assignments.Count;
        return index;
    }

    public bool TryGet(uint uid, out SquadAssignment assignment) =>
        _assignments.TryGetValue(uid, out assignment!);

    private static void WalkSquad(
        IMemoryReader reader,
        IFmMemoryLayout layout,
        ulong team,
        string clubName,
        int teamType,
        string? division,
        IReadOnlyDictionary<ulong, uint> personToUid,
        IReadOnlyDictionary<uint, string?> parentClubByUid,
        SquadClubIndex index)
    {
        if (!reader.TryReadUInt64(team + (ulong)layout.TeamSquadBeginOffset, out var squadBegin)
            || !reader.TryReadUInt64(team + (ulong)layout.TeamSquadEndOffset, out var squadEnd)
            || squadBegin == 0
            || squadEnd <= squadBegin
            || (squadEnd - squadBegin) % 8 != 0)
        {
            return;
        }

        var count = (long)((squadEnd - squadBegin) / 8);
        if (count is <= 0 or > 60)
        {
            return;
        }

        for (long pi = 0; pi < count; pi++)
        {
            if (!reader.TryReadUInt64(squadBegin + (ulong)(pi * 8), out var entry) || entry == 0)
            {
                continue;
            }

            if (!TryResolvePersonUid(reader, entry, personToUid, out var uid))
            {
                continue;
            }

            parentClubByUid.TryGetValue(uid, out var parent);
            var candidate = new SquadHit(clubName, teamType, division);
            if (!index._assignments.TryGetValue(uid, out var cur))
            {
                index._assignments[uid] = ToAssignment(candidate);
                continue;
            }

            var currentHit = new SquadHit(cur.ClubName, cur.TeamType, cur.Division);
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
            Division = hit.Division,
        };

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
            if (!reader.TryReadUInt64(entry + (ulong)off, out var q) || q == 0)
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
        if (!reader.TryReadUInt64(team + (ulong)layout.TeamSchedulePtrOffset, out var schedule)
            || schedule == 0)
        {
            return;
        }

        foreach (var so in new[] { layout.ScheduleNextMatchOffset, layout.ScheduleNextMatchAltOffset })
        {
            if (!reader.TryReadUInt32(schedule + (ulong)so, out var raw))
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
}
