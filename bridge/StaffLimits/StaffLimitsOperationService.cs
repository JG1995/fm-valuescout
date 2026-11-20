using FmDataBridge.Extraction;
using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Mutations;
using FmDataBridge.Protocol;

namespace FmDataBridge.StaffLimits;

internal enum StaffLimitsFailure
{
    None,
    InvalidRequest,
    UnsupportedGameBuild,
    NoLiveScan,
    SourceRequestMismatch,
    ClubNotFound,
    ReadUnavailable,
    ReadFailed,
    NoLimits,
}

internal readonly record struct StaffLimitsExecutionResult(
    bool Succeeded,
    StaffLimitsFailure Failure,
    StaffLimitsResult? Result,
    string? Detail = null);

/// <summary>Opens FM's staff-group channels on teams and copies what FM sends. The plugin's implementation calls into the game.</summary>
internal interface IStaffLimitsReader
{
    bool StaffLimitsAvailable { get; }

    /// <summary>
    /// Opens each property on each team, named by database index, and returns the last value FM
    /// sent on each channel. A channel FM sent nothing on is missing.
    /// </summary>
    bool TryReadStaffGroups(
        IReadOnlyList<int> teamIndexes,
        IReadOnlyList<uint> properties,
        out IReadOnlyDictionary<(int TeamIndex, uint Property), FmValue> values,
        out string? error);
}

/// <summary>
/// Reads the board's staff limits for the managed club's first, reserve, and youth teams from the
/// three staff groups FM's Staff overview shows for a team. It writes no game memory and sends no
/// action.
/// </summary>
internal sealed class StaffLimitsOperationService
{
    /// <summary>The team properties FM's Staff overview reads: StgC, StgR, and StgM.</summary>
    private static readonly (uint Property, string Group)[] Groups =
    {
        (1400137539, StaffLimitsGroup.Coaching),
        (1400137554, StaffLimitsGroup.Recruitment),
        (1400137549, StaffLimitsGroup.Medical),
    };

    private static readonly string[] Levels = { TeamLevelMap.Senior, TeamLevelMap.Reserve, TeamLevelMap.Youth };

    private readonly LayoutRegistry _layouts;
    private readonly PlayerMutationIndex _players;
    private readonly IStaffLimitsReader _reader;

    public StaffLimitsOperationService(LayoutRegistry layouts, PlayerMutationIndex players, IStaffLimitsReader reader)
    {
        _layouts = layouts ?? throw new ArgumentNullException(nameof(layouts));
        _players = players ?? throw new ArgumentNullException(nameof(players));
        _reader = reader ?? throw new ArgumentNullException(nameof(reader));
    }

    public StaffLimitsExecutionResult Execute(BridgeRequest request, string gameVersion, IMemoryReader memory)
    {
        ArgumentNullException.ThrowIfNull(request);
        ArgumentNullException.ThrowIfNull(memory);

        if (request.Operation != BridgeProtocol.OperationReadStaffLimits
            || string.IsNullOrWhiteSpace(request.SourceRequestId)
            || request.ClubUid is not { } clubUid)
        {
            return Failed(StaffLimitsFailure.InvalidRequest);
        }

        if (!_layouts.TryResolveFromGameVersion(gameVersion, out var layout))
        {
            return Failed(StaffLimitsFailure.UnsupportedGameBuild);
        }

        if (!_players.HasCandidatesForGameVersion(gameVersion))
        {
            return Failed(StaffLimitsFailure.NoLiveScan);
        }

        if (!_players.IsFrom(request.SourceRequestId))
        {
            return Failed(StaffLimitsFailure.SourceRequestMismatch);
        }

        if (!_reader.StaffLimitsAvailable)
        {
            return Failed(StaffLimitsFailure.ReadUnavailable);
        }

        if (!_players.TryGetClub(request.SourceRequestId, clubUid, out var club)
            || !memory.TryReadUInt32(club + (ulong)layout.ObjectUidOffset, out var liveClubUid)
            || liveClubUid != clubUid)
        {
            return Failed(StaffLimitsFailure.ClubNotFound);
        }

        var clubTeams = ClubTeams.Read(memory, layout, club);
        var teams = new List<(string Level, int Type, int Index)>();
        foreach (var level in Levels)
        {
            if (ClubTeams.AtLevel(clubTeams, level) is { } team
                && ClubTeams.TryReadDatabaseIndex(memory, team.Address, layout, out var index))
            {
                teams.Add((level, team.Type, index));
            }
        }

        if (teams.Count == 0 || teams[0].Level != TeamLevelMap.Senior)
        {
            return Failed(StaffLimitsFailure.ClubNotFound);
        }

        if (!_reader.TryReadStaffGroups(
                teams.Select(team => team.Index).ToArray(),
                Groups.Select(group => group.Property).ToArray(),
                out var values,
                out var error))
        {
            return Failed(StaffLimitsFailure.ReadFailed, error);
        }

        var result = new StaffLimitsResult
        {
            Teams = teams
                .Select(team => new StaffLimitsTeam
                {
                    Level = team.Level,
                    TeamType = team.Type,
                    Groups = Groups
                        .Select(group => StaffLimitsParser.ParseGroup(
                            group.Group,
                            values.TryGetValue((team.Index, group.Property), out var value) ? value : null))
                        .OfType<StaffLimitsGroup>()
                        .ToArray(),
                })
                .ToArray(),
        };

        // FM always shows the first team's groups, so none at all means the read saw nothing.
        return result.Teams[0].Groups.Count == 0
            ? Failed(StaffLimitsFailure.NoLimits)
            : new StaffLimitsExecutionResult(true, StaffLimitsFailure.None, result);
    }

    private static StaffLimitsExecutionResult Failed(StaffLimitsFailure failure, string? detail = null) =>
        new(false, failure, null, detail);
}
