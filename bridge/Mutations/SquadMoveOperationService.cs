using System.Diagnostics;
using FmDataBridge.Extraction;
using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Protocol;
using FmDataBridge.Scanning;

namespace FmDataBridge.Mutations;

internal enum SquadMoveFailure
{
    None,
    InvalidRequest,
    UnsupportedGameBuild,
    NoLiveScan,
    SourceRequestMismatch,
    ClubNotFound,
    ManagerNotFound,
    GameActionsUnavailable,
    DispatchFailed,
}

/// <summary>
/// The outcome of one squad-move request. A dispatch failure still carries the outcomes it
/// confirmed, because FM may have made some of the moves.
/// </summary>
internal readonly record struct SquadMoveExecutionResult(
    bool Succeeded,
    SquadMoveFailure Failure,
    SquadMoveResult? Result,
    string? Detail = null);

/// <summary>
/// One FM squad-change action: the player's, the club's, and the human manager's database
/// indexes, and the team types he moves from and to.
/// </summary>
internal readonly record struct SquadMoveAction(
    int PersonIndex,
    int ClubIndex,
    int HumanManagerIndex,
    int FromTeamType,
    int ToTeamType);

/// <summary>Runs FM's own squad-change action. The plugin's implementation calls into the game.</summary>
internal interface ISquadMoveDispatcher
{
    bool IsAvailable { get; }

    /// <summary>Sends every action to FM, or returns false with a reason when it could not.</summary>
    bool TryDispatch(IReadOnlyList<SquadMoveAction> actions, out string? error);
}

/// <summary>
/// Moves the managed club's players between the club's own teams through FM's own squad action,
/// then confirms each move from the club's squad lists. It writes no game memory itself.
/// </summary>
internal sealed class SquadMoveOperationService
{
    private static readonly TimeSpan DefaultConfirmTimeout = TimeSpan.FromSeconds(5);
    private static readonly TimeSpan DefaultPollInterval = TimeSpan.FromMilliseconds(100);

    private readonly LayoutRegistry _layouts;
    private readonly PlayerMutationIndex _index;
    private readonly ISquadMoveDispatcher _dispatcher;
    private readonly TimeSpan _confirmTimeout;
    private readonly TimeSpan _pollInterval;

    public SquadMoveOperationService(
        LayoutRegistry layouts,
        PlayerMutationIndex index,
        ISquadMoveDispatcher dispatcher,
        TimeSpan? confirmTimeout = null,
        TimeSpan? pollInterval = null)
    {
        _layouts = layouts ?? throw new ArgumentNullException(nameof(layouts));
        _index = index ?? throw new ArgumentNullException(nameof(index));
        _dispatcher = dispatcher ?? throw new ArgumentNullException(nameof(dispatcher));
        _confirmTimeout = confirmTimeout ?? DefaultConfirmTimeout;
        _pollInterval = pollInterval ?? DefaultPollInterval;
    }

    public SquadMoveExecutionResult Execute(BridgeRequest request, string gameVersion, IMemoryReader reader)
    {
        ArgumentNullException.ThrowIfNull(request);
        ArgumentNullException.ThrowIfNull(reader);

        if (request.Operation != BridgeProtocol.OperationMoveSquadPlayers
            || string.IsNullOrWhiteSpace(request.SourceRequestId)
            || request.ClubUid is not { } clubUid
            || request.SquadMoves is not { Count: > 0 } moves)
        {
            return Failed(SquadMoveFailure.InvalidRequest);
        }

        var source = request.SourceRequestId;
        if (!_layouts.TryResolveFromGameVersion(gameVersion, out var layout))
        {
            return Failed(SquadMoveFailure.UnsupportedGameBuild);
        }

        if (!_index.HasCandidatesForGameVersion(gameVersion))
        {
            return Failed(SquadMoveFailure.NoLiveScan);
        }

        if (!_index.IsFrom(source))
        {
            return Failed(SquadMoveFailure.SourceRequestMismatch);
        }

        if (!_dispatcher.IsAvailable)
        {
            return Failed(SquadMoveFailure.GameActionsUnavailable);
        }

        if (!_index.TryGetClub(source, clubUid, out var club)
            || !reader.TryReadUInt32(club + (ulong)layout.ObjectUidOffset, out var liveClubUid)
            || liveClubUid != clubUid
            || !ClubTeams.TryReadDatabaseIndex(reader, club, layout, out var clubIndex))
        {
            return Failed(SquadMoveFailure.ClubNotFound);
        }

        var teams = ClubTeams.Read(reader, layout, club);
        if (teams.Count == 0)
        {
            return Failed(SquadMoveFailure.ClubNotFound);
        }

        if (!ClubTeams.TryReadHumanManagerIndex(reader, layout, teams, out var managerIndex, out var managerDetail))
        {
            return new SquadMoveExecutionResult(false, SquadMoveFailure.ManagerNotFound, null, managerDetail);
        }

        var outcomes = new SquadMoveOutcome[moves.Count];
        var pending = new List<PendingMove>();
        var actions = new List<SquadMoveAction>();
        for (var slot = 0; slot < moves.Count; slot++)
        {
            var move = moves[slot];
            if (_index.TryGet(source, move.PlayerUid, out var indexed) != PlayerMutationLookup.Found
                || !reader.TryReadUInt32(
                    indexed.Candidate.ObjectAddress + (ulong)layout.ObjectUidOffset,
                    out var liveUid)
                || liveUid != move.PlayerUid
                || !ClubTeams.TryReadDatabaseIndex(reader, indexed.Candidate.ObjectAddress, layout, out var personIndex))
            {
                outcomes[slot] = Outcome(SquadMoveOutcome.NotFound, null);
                continue;
            }

            var player = indexed.Candidate;
            var current = TeamOf(teams, player);
            if (current is null)
            {
                outcomes[slot] = Outcome(SquadMoveOutcome.NotInClubTeams, null);
                continue;
            }

            if (TeamLevelMap.FromTeamType(current.Type) == move.TargetLevel)
            {
                outcomes[slot] = Outcome(SquadMoveOutcome.AlreadyThere, current.Type);
                continue;
            }

            var target = ClubTeams.AtLevel(teams, move.TargetLevel);
            if (target is null)
            {
                outcomes[slot] = Outcome(SquadMoveOutcome.NoTargetTeam, current.Type);
                continue;
            }

            pending.Add(new PendingMove(slot, player, target.Type));
            actions.Add(new SquadMoveAction(personIndex, clubIndex, managerIndex, current.Type, target.Type));
        }

        if (actions.Count == 0)
        {
            return new SquadMoveExecutionResult(true, SquadMoveFailure.None, new SquadMoveResult { Moves = outcomes });
        }

        var dispatched = _dispatcher.TryDispatch(actions, out var error);
        Confirm(reader, layout, club, pending, outcomes);
        var result = new SquadMoveResult { Moves = outcomes };
        return dispatched
            ? new SquadMoveExecutionResult(true, SquadMoveFailure.None, result)
            : new SquadMoveExecutionResult(false, SquadMoveFailure.DispatchFailed, result, error);
    }

    /// <summary>Waits until FM's squad lists show every sent move, or the confirmation time runs out.</summary>
    private void Confirm(
        IMemoryReader reader,
        IFmMemoryLayout layout,
        ulong club,
        IReadOnlyList<PendingMove> pending,
        SquadMoveOutcome[] outcomes)
    {
        var waiting = pending.ToList();
        var clock = Stopwatch.StartNew();
        while (true)
        {
            var teams = ClubTeams.Read(reader, layout, club);
            waiting.RemoveAll(move =>
            {
                // He counts where the dump would put him: his highest-ranked team.
                if (TeamOf(teams, move.Player)?.Type != move.TargetType)
                {
                    return false;
                }

                outcomes[move.Slot] = Outcome(SquadMoveOutcome.Moved, move.TargetType);
                return true;
            });

            if (waiting.Count == 0 || clock.Elapsed >= _confirmTimeout)
            {
                foreach (var move in waiting)
                {
                    outcomes[move.Slot] = Outcome(SquadMoveOutcome.NotConfirmed, TeamOf(teams, move.Player)?.Type);
                }

                return;
            }

            Thread.Sleep(_pollInterval);
        }
    }

    /// <summary>The team a player counts for: the highest-ranked of the club's teams that list him.</summary>
    private static ClubTeam? TeamOf(IReadOnlyList<ClubTeam> teams, PersonCandidate player) =>
        teams
            .Where(team => team.Contains(player))
            .OrderBy(team => TeamLevelMap.Rank(team.Type))
            .ThenBy(team => team.Type)
            .FirstOrDefault();

    private static SquadMoveOutcome Outcome(string outcome, int? teamType) =>
        new()
        {
            Outcome = outcome,
            TeamType = teamType,
            TeamLevel = teamType is { } type ? TeamLevelMap.FromTeamType(type) : null,
        };

    private static SquadMoveExecutionResult Failed(SquadMoveFailure failure) =>
        new(false, failure, null);

    private sealed record PendingMove(int Slot, PersonCandidate Player, int TargetType);
}
