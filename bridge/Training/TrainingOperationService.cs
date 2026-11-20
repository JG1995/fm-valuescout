using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Mutations;
using FmDataBridge.Protocol;
using FmDataBridge.Scanning;
using FmDataBridge.StaffLimits;

namespace FmDataBridge.Training;

internal enum TrainingFailure
{
    None,
    InvalidRequest,
    UnsupportedGameBuild,
    NoLiveScan,
    SourceRequestMismatch,
    ClubNotFound,
    ManagerNotFound,
    TrainingUnavailable,
    DispatchFailed,
}

/// <summary>The outcome of one training request.</summary>
internal readonly record struct TrainingExecutionResult(
    bool Succeeded,
    TrainingFailure Failure,
    TrainingResult? Result,
    string? Detail = null);

/// <summary>One player's training in FM's IDs, named by his database index.</summary>
internal readonly record struct TrainingAction(
    int PersonIndex,
    int? IpPosition,
    ulong? IpRole,
    int? OopRole,
    int? Focus);

/// <summary>Sends FM's training events and reads the training back. The plugin's implementation calls into the game.</summary>
internal interface ITrainingDispatcher
{
    bool TrainingAvailable { get; }

    /// <summary>
    /// Sends each action's events as FM's Individual Training screen does, then reads the three
    /// training properties of each player. A property FM sent nothing for is missing.
    /// </summary>
    bool TryApplyTraining(
        int humanManagerIndex,
        IReadOnlyList<TrainingAction> actions,
        out IReadOnlyDictionary<(int PersonIndex, uint Property), FmValue> values,
        out string? error);
}

/// <summary>
/// Sets the managed club's players' individual training through FM's own training events, then
/// confirms each part from what FM holds afterwards. It writes no game memory itself.
/// </summary>
internal sealed class TrainingOperationService
{
    private readonly LayoutRegistry _layouts;
    private readonly PlayerMutationIndex _index;
    private readonly ITrainingDispatcher _dispatcher;

    public TrainingOperationService(LayoutRegistry layouts, PlayerMutationIndex index, ITrainingDispatcher dispatcher)
    {
        _layouts = layouts ?? throw new ArgumentNullException(nameof(layouts));
        _index = index ?? throw new ArgumentNullException(nameof(index));
        _dispatcher = dispatcher ?? throw new ArgumentNullException(nameof(dispatcher));
    }

    public TrainingExecutionResult Execute(BridgeRequest request, string gameVersion, IMemoryReader reader)
    {
        ArgumentNullException.ThrowIfNull(request);
        ArgumentNullException.ThrowIfNull(reader);

        if (request.Operation != BridgeProtocol.OperationApplyTraining
            || string.IsNullOrWhiteSpace(request.SourceRequestId)
            || request.ClubUid is not { } clubUid
            || request.TrainingChanges is not { Count: > 0 } changes)
        {
            return Failed(TrainingFailure.InvalidRequest);
        }

        var source = request.SourceRequestId;
        if (!_layouts.TryResolveFromGameVersion(gameVersion, out var layout))
        {
            return Failed(TrainingFailure.UnsupportedGameBuild);
        }

        if (!_index.HasCandidatesForGameVersion(gameVersion))
        {
            return Failed(TrainingFailure.NoLiveScan);
        }

        if (!_index.IsFrom(source))
        {
            return Failed(TrainingFailure.SourceRequestMismatch);
        }

        if (!_dispatcher.TrainingAvailable)
        {
            return Failed(TrainingFailure.TrainingUnavailable);
        }

        if (!_index.TryGetClub(source, clubUid, out var club)
            || !reader.TryReadUInt32(club + (ulong)layout.ObjectUidOffset, out var liveClubUid)
            || liveClubUid != clubUid)
        {
            return Failed(TrainingFailure.ClubNotFound);
        }

        var teams = ClubTeams.Read(reader, layout, club);
        if (teams.Count == 0)
        {
            return Failed(TrainingFailure.ClubNotFound);
        }

        if (!ClubTeams.TryReadHumanManagerIndex(reader, layout, teams, out var managerIndex, out var managerDetail))
        {
            return new TrainingExecutionResult(false, TrainingFailure.ManagerNotFound, null, managerDetail);
        }

        var outcomes = new TrainingOutcome[changes.Count];
        var pending = new List<(int Slot, TrainingAction Action)>();
        for (var slot = 0; slot < changes.Count; slot++)
        {
            var change = changes[slot];
            if (_index.TryGet(source, change.PlayerUid, out var indexed) != PlayerMutationLookup.Found
                || !reader.TryReadUInt32(indexed.Candidate.ObjectAddress + (ulong)layout.ObjectUidOffset, out var liveUid)
                || liveUid != change.PlayerUid
                || !ClubTeams.TryReadDatabaseIndex(reader, indexed.Candidate.ObjectAddress, layout, out var personIndex))
            {
                outcomes[slot] = new TrainingOutcome { Outcome = TrainingOutcome.NotFound };
                continue;
            }

            // Only the managed club's own players: FM's events would change anyone's training.
            if (!teams.Any(team => team.Contains(indexed.Candidate)))
            {
                outcomes[slot] = new TrainingOutcome { Outcome = TrainingOutcome.NotAtClub };
                continue;
            }

            pending.Add((slot, new TrainingAction(personIndex, change.IpPosition, change.IpRole, change.OopRole, change.Focus)));
        }

        if (pending.Count == 0)
        {
            return new TrainingExecutionResult(true, TrainingFailure.None, new TrainingResult { Players = outcomes });
        }

        if (!_dispatcher.TryApplyTraining(
                managerIndex,
                pending.Select(entry => entry.Action).ToArray(),
                out var values,
                out var error))
        {
            // FM may have taken some of the events before it failed; nothing was read back.
            foreach (var (slot, _) in pending)
            {
                outcomes[slot] = new TrainingOutcome { Outcome = TrainingOutcome.NotConfirmed };
            }

            return new TrainingExecutionResult(
                false,
                TrainingFailure.DispatchFailed,
                new TrainingResult { Players = outcomes },
                error);
        }

        foreach (var (slot, action) in pending)
        {
            outcomes[slot] = Confirm(action, values);
        }

        return new TrainingExecutionResult(true, TrainingFailure.None, new TrainingResult { Players = outcomes });
    }

    /// <summary>What FM holds for the player, and whether it is every part the request set.</summary>
    private static TrainingOutcome Confirm(
        TrainingAction action,
        IReadOnlyDictionary<(int PersonIndex, uint Property), FmValue> values)
    {
        FmValue? Value(uint property) =>
            values.TryGetValue((action.PersonIndex, property), out var value) ? value : null;

        var ip = TrainingParser.InPossession(Value(TrainingParser.InPossessionProperty));
        var oop = TrainingParser.Choice(Value(TrainingParser.OutOfPossessionProperty), TrainingParser.OutOfPossessionKey);
        var focus = TrainingParser.Choice(Value(TrainingParser.FocusProperty), TrainingParser.FocusKey);
        var applied = (action.IpPosition is null || (ip.Position == action.IpPosition && ip.Role == action.IpRole))
            && (action.OopRole is null || oop.Id == action.OopRole)
            && (action.Focus is null || focus.Id == action.Focus);

        return new TrainingOutcome
        {
            Outcome = applied ? TrainingOutcome.Applied : TrainingOutcome.NotConfirmed,
            IpPosition = ip.Position,
            IpRole = ip.Role,
            IpRoleName = ip.Name,
            OopRole = oop.Id,
            OopRoleName = oop.Name,
            Focus = focus.Id,
            FocusName = focus.Name,
        };
    }

    private static TrainingExecutionResult Failed(TrainingFailure failure) => new(false, failure, null);
}

/// <summary>
/// Reads a player's training as FM's Individual Training screen receives it. Each property is a
/// record whose selection sits under one key: an option with FM's ID (<c>ptvl</c>) and name
/// (<c>qvJ6</c>). The in-possession option also holds its position (<c>Posi</c>) beside the role.
/// </summary>
internal static class TrainingParser
{
    public const uint InPossessionProperty = 1347707238; // "PTaf", TrainingPosition
    public const uint OutOfPossessionProperty = 1180201071; // "FXpo", OOPTraining
    public const uint FocusProperty = 1347709286; // "PTif", AdditionalFocus

    public const uint InPossessionKey = 1484020313; // "XtZY"
    public const uint OutOfPossessionKey = 1177831020; // "F4Fl"
    public const uint FocusKey = 1916366157; // "r9mM"
    public const uint OptionValueKey = 1886680684; // "ptvl"
    public const uint OptionNameKey = 1903577654; // "qvJ6"
    public const uint PositionKey = 1349481321; // "Posi"

    public static (int? Position, ulong? Role, string? Name) InPossession(FmValue? value)
    {
        var option = (value as FmRecord)?.Get<FmRecord>(InPossessionKey);
        return (
            Int(option?.Get<FmNumber>(PositionKey)),
            option?.Get<FmNumber>(OptionValueKey)?.Value is { } role and >= 0 ? (ulong)role : null,
            option?.Get<FmText>(OptionNameKey)?.Value);
    }

    public static (int? Id, string? Name) Choice(FmValue? value, uint key)
    {
        var option = (value as FmRecord)?.Get<FmRecord>(key);
        return (Int(option?.Get<FmNumber>(OptionValueKey)), option?.Get<FmText>(OptionNameKey)?.Value);
    }

    private static int? Int(FmNumber? number) =>
        number?.Value is { } value and >= int.MinValue and <= int.MaxValue ? (int)value : null;
}
