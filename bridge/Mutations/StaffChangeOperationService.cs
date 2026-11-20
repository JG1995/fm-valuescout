using System.Diagnostics;
using FmDataBridge.Extraction;
using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Protocol;

namespace FmDataBridge.Mutations;

internal enum StaffChangeFailure
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
/// The outcome of one staff-change request. A dispatch failure still carries the outcomes it
/// read, because FM may have made some of the changes.
/// </summary>
internal readonly record struct StaffChangeExecutionResult(
    bool Succeeded,
    StaffChangeFailure Failure,
    StaffChangeResult? Result,
    string? Detail = null);

internal enum StaffActionKind
{
    Sack,
    Hire,
}

/// <summary>
/// One FM staff action, named by database indexes. A sack names the team the person's contract
/// names; a hire names the team they join, the club's first team, and FM's job type.
/// </summary>
internal readonly record struct StaffAction(
    StaffActionKind Kind,
    int PersonIndex,
    int HumanManagerIndex,
    int TeamIndex,
    int FirstTeamIndex = 0,
    int JobType = 0);

/// <summary>Runs FM's own staff actions. The plugin's implementation calls into the game.</summary>
internal interface IStaffChangeDispatcher
{
    bool StaffActionsAvailable { get; }

    /// <summary>Runs every action in order, or returns false with a reason when it could not.</summary>
    bool TryDispatch(IReadOnlyList<StaffAction> actions, out string? error);
}

/// <summary>
/// Sacks, hires, and re-hires the managed club's staff through FM's own actions: the contract
/// termination its menu sends and the in-game editor's move, which gives the person a contract FM
/// sets for the job. Every sack runs before any hire, so a single-holder job is free when its new
/// holder arrives. Each change is confirmed from the person's contract. It writes no game memory.
/// </summary>
internal sealed class StaffChangeOperationService
{
    private static readonly TimeSpan DefaultConfirmTimeout = TimeSpan.FromSeconds(5);
    private static readonly TimeSpan DefaultPollInterval = TimeSpan.FromMilliseconds(100);

    private readonly LayoutRegistry _layouts;
    private readonly PlayerMutationIndex _players;
    private readonly StaffMutationIndex _staff;
    private readonly IStaffChangeDispatcher _dispatcher;
    private readonly TimeSpan _confirmTimeout;
    private readonly TimeSpan _pollInterval;

    public StaffChangeOperationService(
        LayoutRegistry layouts,
        PlayerMutationIndex players,
        StaffMutationIndex staff,
        IStaffChangeDispatcher dispatcher,
        TimeSpan? confirmTimeout = null,
        TimeSpan? pollInterval = null)
    {
        _layouts = layouts ?? throw new ArgumentNullException(nameof(layouts));
        _players = players ?? throw new ArgumentNullException(nameof(players));
        _staff = staff ?? throw new ArgumentNullException(nameof(staff));
        _dispatcher = dispatcher ?? throw new ArgumentNullException(nameof(dispatcher));
        _confirmTimeout = confirmTimeout ?? DefaultConfirmTimeout;
        _pollInterval = pollInterval ?? DefaultPollInterval;
    }

    public StaffChangeExecutionResult Execute(BridgeRequest request, string gameVersion, IMemoryReader reader)
    {
        ArgumentNullException.ThrowIfNull(request);
        ArgumentNullException.ThrowIfNull(reader);

        if (request.Operation != BridgeProtocol.OperationApplyStaffChanges
            || string.IsNullOrWhiteSpace(request.SourceRequestId)
            || request.ClubUid is not { } clubUid
            || request.StaffChanges is not { Count: > 0 } changes)
        {
            return Failed(StaffChangeFailure.InvalidRequest);
        }

        var source = request.SourceRequestId;
        if (!_layouts.TryResolveFromGameVersion(gameVersion, out var layout))
        {
            return Failed(StaffChangeFailure.UnsupportedGameBuild);
        }

        if (!_players.HasCandidatesForGameVersion(gameVersion) || !_staff.HasCandidatesForGameVersion(gameVersion))
        {
            return Failed(StaffChangeFailure.NoLiveScan);
        }

        if (!_players.IsFrom(source))
        {
            return Failed(StaffChangeFailure.SourceRequestMismatch);
        }

        if (!_dispatcher.StaffActionsAvailable)
        {
            return Failed(StaffChangeFailure.GameActionsUnavailable);
        }

        if (!_players.TryGetClub(source, clubUid, out var club)
            || !reader.TryReadUInt32(club + (ulong)layout.ObjectUidOffset, out var liveClubUid)
            || liveClubUid != clubUid)
        {
            return Failed(StaffChangeFailure.ClubNotFound);
        }

        var teams = ClubTeams.Read(reader, layout, club);
        var firstTeam = teams.FirstOrDefault(team => team.Type == 0);
        if (firstTeam is null || !ClubTeams.TryReadDatabaseIndex(reader, firstTeam.Address, layout, out var firstTeamIndex))
        {
            return Failed(StaffChangeFailure.ClubNotFound);
        }

        if (!ClubTeams.TryReadHumanManagerIndex(reader, layout, teams, out var managerIndex, out var managerDetail))
        {
            return new StaffChangeExecutionResult(false, StaffChangeFailure.ManagerNotFound, null, managerDetail);
        }

        var outcomes = new StaffChangeOutcome[changes.Count];
        var pending = new List<PendingChange>();
        var sacks = new List<StaffAction>();
        var hires = new List<StaffAction>();
        for (var slot = 0; slot < changes.Count; slot++)
        {
            var change = changes[slot];
            if (_staff.TryGet(source, change.StaffUid, out var indexed) != StaffMutationLookup.Found
                || !reader.TryReadUInt32(indexed.Candidate.ObjectAddress + (ulong)layout.ObjectUidOffset, out var liveUid)
                || liveUid != change.StaffUid
                || !ClubTeams.TryReadDatabaseIndex(reader, indexed.Candidate.ObjectAddress, layout, out var personIndex))
            {
                outcomes[slot] = new StaffChangeOutcome { Outcome = StaffChangeOutcome.NotFound };
                continue;
            }

            var person = indexed.Candidate.ObjectAddress;
            var leaves = change.Kind is StaffChangeItem.Sack or StaffChangeItem.Change;
            var joins = change.Kind is StaffChangeItem.Hire or StaffChangeItem.Change;

            ClubTeam? target = null;
            var targetIndex = 0;
            if (joins
                && ((target = ClubTeams.AtLevel(teams, change.TargetLevel ?? "")) is null
                    || !ClubTeams.TryReadDatabaseIndex(reader, target.Address, layout, out targetIndex)))
            {
                outcomes[slot] = Contract(StaffChangeOutcome.NoTargetTeam, reader, layout, person);
                continue;
            }

            var contractTeamIndex = 0;
            if (leaves)
            {
                var link = ContractClubReader.TryRead(reader, person, layout);
                if (link?.ClubUid != clubUid
                    || link.TeamAddress == 0
                    || !ClubTeams.TryReadDatabaseIndex(reader, link.TeamAddress, layout, out contractTeamIndex))
                {
                    outcomes[slot] = Contract(StaffChangeOutcome.NotAtClub, reader, layout, person);
                    continue;
                }

                sacks.Add(new StaffAction(StaffActionKind.Sack, personIndex, managerIndex, contractTeamIndex));
            }

            if (joins)
            {
                hires.Add(new StaffAction(
                    StaffActionKind.Hire,
                    personIndex,
                    managerIndex,
                    targetIndex,
                    firstTeamIndex,
                    change.JobType ?? 0));
            }

            pending.Add(new PendingChange(slot, change.Kind, person, change.JobType, target?.Type));
        }

        var actions = sacks.Concat(hires).ToList();
        if (actions.Count == 0)
        {
            return new StaffChangeExecutionResult(true, StaffChangeFailure.None, new StaffChangeResult { Changes = outcomes });
        }

        var dispatched = _dispatcher.TryDispatch(actions, out var error);
        Confirm(reader, layout, clubUid, pending, outcomes);
        var result = new StaffChangeResult { Changes = outcomes };
        return dispatched
            ? new StaffChangeExecutionResult(true, StaffChangeFailure.None, result)
            : new StaffChangeExecutionResult(false, StaffChangeFailure.DispatchFailed, result, error);
    }

    /// <summary>Waits until every person's contract shows their change, or the confirmation time runs out.</summary>
    private void Confirm(
        IMemoryReader reader,
        IFmMemoryLayout layout,
        uint clubUid,
        IReadOnlyList<PendingChange> pending,
        StaffChangeOutcome[] outcomes)
    {
        var waiting = pending.ToList();
        var clock = Stopwatch.StartNew();
        while (true)
        {
            waiting.RemoveAll(change =>
            {
                var link = ContractClubReader.TryRead(reader, change.Person, layout);
                var atClub = link?.ClubUid == clubUid;
                bool done;
                string outcome;
                if (change.Kind == StaffChangeItem.Sack)
                {
                    done = !atClub;
                    outcome = StaffChangeOutcome.Sacked;
                }
                else
                {
                    done = atClub
                        && link!.TeamType == change.TargetTeamType
                        && StaffContractReader.Read(reader, change.Person, layout).JobId == change.JobType;
                    outcome = change.Kind == StaffChangeItem.Change ? StaffChangeOutcome.Changed : StaffChangeOutcome.Hired;
                }

                if (done)
                {
                    outcomes[change.Slot] = Contract(outcome, reader, layout, change.Person);
                }

                return done;
            });

            if (waiting.Count == 0 || clock.Elapsed >= _confirmTimeout)
            {
                foreach (var change in waiting)
                {
                    var atClub = ContractClubReader.TryRead(reader, change.Person, layout)?.ClubUid == clubUid;
                    var outcome = change.Kind == StaffChangeItem.Change && !atClub
                        ? StaffChangeOutcome.LeftUnemployed
                        : StaffChangeOutcome.NotConfirmed;
                    outcomes[change.Slot] = Contract(outcome, reader, layout, change.Person);
                }

                return;
            }

            Thread.Sleep(_pollInterval);
        }
    }

    /// <summary>The outcome with the person's contract as FM holds it now.</summary>
    private static StaffChangeOutcome Contract(string outcome, IMemoryReader reader, IFmMemoryLayout layout, ulong person)
    {
        var link = ContractClubReader.TryRead(reader, person, layout);
        if (link is null)
        {
            return new StaffChangeOutcome { Outcome = outcome };
        }

        var contract = StaffContractReader.Read(reader, person, layout);
        return new StaffChangeOutcome
        {
            Outcome = outcome,
            ClubUid = link.ClubUid,
            Club = link.ClubName,
            Division = link.Division,
            JobId = contract.JobId,
            WeeklyWageGbp = contract.WeeklyWageGbp,
            ContractExpiryYear = contract.ContractExpiryYear,
            ContractExpiryDayOfYear = contract.ContractExpiryDayOfYear,
            TeamType = link.TeamType,
            TeamLevel = link.TeamType is { } type ? TeamLevelMap.FromTeamType(type) : null,
        };
    }

    private static StaffChangeExecutionResult Failed(StaffChangeFailure failure) =>
        new(false, failure, null);

    private sealed record PendingChange(int Slot, string Kind, ulong Person, int? JobType, int? TargetTeamType);
}
