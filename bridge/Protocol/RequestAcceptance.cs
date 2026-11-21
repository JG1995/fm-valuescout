using System.Text.Json;

namespace FmDataBridge.Protocol;

/// <summary>
/// Validates and consumes <c>request.json</c> — stale or malformed requests are rejected.
/// </summary>
public static class RequestAcceptance
{
    private static readonly JsonSerializerOptions SerializerOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        PropertyNameCaseInsensitive = true,
        WriteIndented = true,
    };

    public static bool IsFresh(DateTimeOffset createdAtUtc, DateTimeOffset now, TimeSpan ttl) =>
        createdAtUtc <= now && now - createdAtUtc <= ttl;

    /// <summary>
    /// Reads, validates, and deletes <paramref name="requestPath"/> when accepted or rejected.
    /// <paramref name="observedRequestId"/> is set whenever a request id was parseable (including rejects).
    /// </summary>
    public static bool TryAccept(
        string requestPath,
        DateTimeOffset now,
        TimeSpan ttl,
        out BridgeRequest request,
        out string? rejectReason,
        out string? observedRequestId)
    {
        request = null!;
        rejectReason = null;
        observedRequestId = null;

        if (!TryReadRequest(requestPath, out var parsed, out rejectReason))
        {
            return false;
        }

        if (!string.IsNullOrWhiteSpace(parsed!.RequestId))
        {
            observedRequestId = parsed.RequestId;
        }

        if (!TryValidateShape(parsed, out rejectReason))
        {
            TryDelete(requestPath);
            return false;
        }

        if (!IsFresh(parsed.CreatedAtUtc, now, ttl))
        {
            rejectReason = $"stale request (age exceeds {ttl.TotalSeconds:0}s TTL)";
            TryDelete(requestPath);
            return false;
        }

        TryDelete(requestPath);
        request = CopyRequest(parsed, parsed.CreatedAtUtc);
        return true;
    }

    /// <summary>
    /// While bridge work is in progress, bump <c>createdAtUtc</c> so a waiting request does not TTL-expire.
    /// Returns true when the file was rewritten.
    /// </summary>
    public static bool TryRefreshCreatedAtUtc(string requestPath, DateTimeOffset now)
        => TryRefreshCreatedAtUtc(
            requestPath,
            now,
            TimeSpan.FromSeconds(BridgeProtocol.RequestTtlSeconds));

    /// <summary>
    /// Refreshes only a request that is still fresh under <paramref name="ttl"/>. An already expired request stays
    /// unchanged so normal acceptance can reject it instead of extending its authorization window.
    /// </summary>
    public static bool TryRefreshCreatedAtUtc(
        string requestPath,
        DateTimeOffset now,
        TimeSpan ttl)
    {
        if (!TryReadRequest(requestPath, out var parsed, out _)
            || !TryValidateShape(parsed!, out _))
        {
            return false;
        }

        if (!IsFresh(parsed!.CreatedAtUtc, now, ttl))
        {
            return false;
        }

        var refreshed = CopyRequest(parsed!, now);
        try
        {
            var tempPath = requestPath + ".tmp";
            File.WriteAllText(tempPath, JsonSerializer.Serialize(refreshed, SerializerOptions));
            File.Move(tempPath, requestPath, overwrite: true);
            return true;
        }
        catch
        {
            return false;
        }
    }

    private static bool TryReadRequest(
        string requestPath,
        out BridgeRequest? request,
        out string? rejectReason)
    {
        request = null;
        rejectReason = null;

        if (!File.Exists(requestPath))
        {
            rejectReason = "request file missing";
            return false;
        }

        string json;
        try
        {
            json = File.ReadAllText(requestPath);
        }
        catch (Exception ex)
        {
            rejectReason = $"request read failed: {ex.Message}";
            TryDelete(requestPath);
            return false;
        }

        try
        {
            request = JsonSerializer.Deserialize<BridgeRequest>(json, SerializerOptions);
        }
        catch (Exception ex)
        {
            rejectReason = $"request JSON invalid: {ex.Message}";
            TryDelete(requestPath);
            return false;
        }

        if (request is null)
        {
            rejectReason = "request JSON empty";
            TryDelete(requestPath);
            return false;
        }

        return true;
    }

    private static bool TryValidateShape(BridgeRequest request, out string? rejectReason)
    {
        rejectReason = null;

        if (request.ProtocolVersion != BridgeProtocol.ProtocolVersion)
        {
            rejectReason =
                $"unsupported request protocol version {request.ProtocolVersion}; expected {BridgeProtocol.ProtocolVersion}";
            return false;
        }

        if (string.IsNullOrWhiteSpace(request.RequestId))
        {
            rejectReason = "requestId is required";
            return false;
        }

        return request.Operation switch
        {
            BridgeProtocol.OperationFullDump => TryValidateFullDump(request, out rejectReason),
            BridgeProtocol.OperationBoostCurrentAbility => TryValidateBoostCurrentAbility(request, out rejectReason),
            BridgeProtocol.OperationBoostStaffCurrentAbility => TryValidateBoostStaffCurrentAbility(request, out rejectReason),
            BridgeProtocol.OperationWonderkidMentality => TryValidateWonderkidMentality(request, out rejectReason),
            BridgeProtocol.OperationMoveSquadPlayers => TryValidateSquadMoves(request, out rejectReason),
            BridgeProtocol.OperationApplyStaffChanges => TryValidateStaffChanges(request, out rejectReason),
            BridgeProtocol.OperationReadStaffLimits => TryValidateStaffLimits(request, out rejectReason),
            BridgeProtocol.OperationApplyTraining => TryValidateTraining(request, out rejectReason),
            _ => RejectUnsupportedOperation(request.Operation, out rejectReason),
        };
    }

    private static bool HasClubActionFields(BridgeRequest request) =>
        request.ClubUid is not null
        || request.SquadMoves is not null
        || request.StaffChanges is not null
        || request.TrainingChanges is not null;

    /// <summary>FM's position masks for in-possession training: GK, DR, DL, DC, WBR, WBL, DM, MR, ML, MC, AMR, AML, AMC, ST.</summary>
    private static readonly HashSet<int> TrainingPositions =
        new() { 1, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384 };

    private static bool TryValidateSquadMoves(BridgeRequest request, out string? rejectReason)
    {
        rejectReason = null;
        if (!string.Equals(
                request.PlayerDatabaseScope,
                PlayerDatabaseScopes.Men,
                StringComparison.Ordinal))
        {
            rejectReason = "playerDatabaseScope is supported only by full-dump";
            return false;
        }

        if (string.IsNullOrWhiteSpace(request.SourceRequestId))
        {
            rejectReason = "sourceRequestId is required for squad moves";
            return false;
        }

        if (request.ClubUid is not { } clubUid || clubUid == 0)
        {
            rejectReason = "clubUid is required for squad moves";
            return false;
        }

        if (request.PlayerUid is not null
            || request.StaffUid is not null
            || request.ExpectedCurrentAbility is not null
            || request.ExpectedPotentialAbility is not null
            || request.CurrentAbilityIncrement is not null
            || request.ExpectedAmbition is not null
            || request.ExpectedProfessionalism is not null
            || request.ExpectedDetermination is not null)
        {
            rejectReason = "squad moves do not accept boost fields";
            return false;
        }

        if (request.StaffChanges is not null)
        {
            rejectReason = "squad moves do not accept staff changes";
            return false;
        }

        if (request.TrainingChanges is not null)
        {
            rejectReason = "squad moves do not accept training changes";
            return false;
        }

        if (request.SquadMoves is not { Count: > 0 and <= BridgeProtocol.MaxSquadMoves } moves)
        {
            rejectReason = $"squadMoves must list 1 through {BridgeProtocol.MaxSquadMoves} players";
            return false;
        }

        var players = new HashSet<uint>();
        foreach (var move in moves)
        {
            if (move is null || move.PlayerUid == 0 || !players.Add(move.PlayerUid))
            {
                rejectReason = "each squad move needs a distinct playerUid";
                return false;
            }

            if (move.TargetLevel is not ("senior" or "reserve" or "youth"))
            {
                rejectReason = "squad move targetLevel must be senior, reserve, or youth";
                return false;
            }
        }

        return true;
    }

    private static bool TryValidateStaffLimits(BridgeRequest request, out string? rejectReason)
    {
        rejectReason = null;
        if (!string.Equals(
                request.PlayerDatabaseScope,
                PlayerDatabaseScopes.Men,
                StringComparison.Ordinal))
        {
            rejectReason = "playerDatabaseScope is supported only by full-dump";
            return false;
        }

        if (string.IsNullOrWhiteSpace(request.SourceRequestId))
        {
            rejectReason = "sourceRequestId is required for staff limits";
            return false;
        }

        if (request.ClubUid is not { } clubUid || clubUid == 0)
        {
            rejectReason = "clubUid is required for staff limits";
            return false;
        }

        if (request.PlayerUid is not null
            || request.StaffUid is not null
            || request.ExpectedCurrentAbility is not null
            || request.ExpectedPotentialAbility is not null
            || request.CurrentAbilityIncrement is not null
            || request.ExpectedAmbition is not null
            || request.ExpectedProfessionalism is not null
            || request.ExpectedDetermination is not null
            || request.SquadMoves is not null
            || request.StaffChanges is not null
            || request.TrainingChanges is not null)
        {
            rejectReason = "staff limits take only the club";
            return false;
        }

        return true;
    }

    private static bool TryValidateStaffChanges(BridgeRequest request, out string? rejectReason)
    {
        rejectReason = null;
        if (!string.Equals(
                request.PlayerDatabaseScope,
                PlayerDatabaseScopes.Men,
                StringComparison.Ordinal))
        {
            rejectReason = "playerDatabaseScope is supported only by full-dump";
            return false;
        }

        if (string.IsNullOrWhiteSpace(request.SourceRequestId))
        {
            rejectReason = "sourceRequestId is required for staff changes";
            return false;
        }

        if (request.ClubUid is not { } clubUid || clubUid == 0)
        {
            rejectReason = "clubUid is required for staff changes";
            return false;
        }

        if (request.PlayerUid is not null
            || request.StaffUid is not null
            || request.ExpectedCurrentAbility is not null
            || request.ExpectedPotentialAbility is not null
            || request.CurrentAbilityIncrement is not null
            || request.ExpectedAmbition is not null
            || request.ExpectedProfessionalism is not null
            || request.ExpectedDetermination is not null
            || request.SquadMoves is not null)
        {
            rejectReason = "staff changes do not accept boost or squad move fields";
            return false;
        }

        if (request.TrainingChanges is not null)
        {
            rejectReason = "staff changes do not accept training changes";
            return false;
        }

        if (request.StaffChanges is not { Count: > 0 and <= BridgeProtocol.MaxStaffChanges } changes)
        {
            rejectReason = $"staffChanges must list 1 through {BridgeProtocol.MaxStaffChanges} staff";
            return false;
        }

        var staff = new HashSet<uint>();
        foreach (var change in changes)
        {
            if (change is null || change.StaffUid == 0 || !staff.Add(change.StaffUid))
            {
                rejectReason = "each staff change needs a distinct staffUid";
                return false;
            }

            switch (change.Kind)
            {
                case StaffChangeItem.Sack:
                    if (change.JobType is not null || change.TargetLevel is not null)
                    {
                        rejectReason = "a staff sack takes no job or team";
                        return false;
                    }

                    break;
                case StaffChangeItem.Hire:
                case StaffChangeItem.Change:
                    if (change.JobType is not (> 0 and <= 255))
                    {
                        rejectReason = "a staff hire needs a jobType from 1 through 255";
                        return false;
                    }

                    if (change.TargetLevel is not ("senior" or "reserve" or "youth"))
                    {
                        rejectReason = "staff change targetLevel must be senior, reserve, or youth";
                        return false;
                    }

                    break;
                default:
                    rejectReason = "staff change kind must be sack, hire, or change";
                    return false;
            }
        }

        return true;
    }

    private static bool TryValidateTraining(BridgeRequest request, out string? rejectReason)
    {
        rejectReason = null;
        if (!string.Equals(
                request.PlayerDatabaseScope,
                PlayerDatabaseScopes.Men,
                StringComparison.Ordinal))
        {
            rejectReason = "playerDatabaseScope is supported only by full-dump";
            return false;
        }

        if (string.IsNullOrWhiteSpace(request.SourceRequestId))
        {
            rejectReason = "sourceRequestId is required for training changes";
            return false;
        }

        if (request.ClubUid is not { } clubUid || clubUid == 0)
        {
            rejectReason = "clubUid is required for training changes";
            return false;
        }

        if (request.PlayerUid is not null
            || request.StaffUid is not null
            || request.ExpectedCurrentAbility is not null
            || request.ExpectedPotentialAbility is not null
            || request.CurrentAbilityIncrement is not null
            || request.ExpectedAmbition is not null
            || request.ExpectedProfessionalism is not null
            || request.ExpectedDetermination is not null
            || request.SquadMoves is not null
            || request.StaffChanges is not null)
        {
            rejectReason = "training changes take only the club and the players' training";
            return false;
        }

        if (request.TrainingChanges is not { Count: > 0 and <= BridgeProtocol.MaxTrainingChanges } changes)
        {
            rejectReason = $"trainingChanges must list 1 through {BridgeProtocol.MaxTrainingChanges} players";
            return false;
        }

        var players = new HashSet<uint>();
        foreach (var change in changes)
        {
            if (change is null || change.PlayerUid == 0 || !players.Add(change.PlayerUid))
            {
                rejectReason = "each training change needs a distinct playerUid";
                return false;
            }

            if (change.IpPosition is null && change.IpRole is null && change.OopRole is null && change.Focus is null)
            {
                rejectReason = "a training change must set a role or a focus";
                return false;
            }

            if ((change.IpPosition is null) != (change.IpRole is null)
                || (change.IpPosition is { } position && !TrainingPositions.Contains(position))
                || (change.IpRole is { } role && (role == 0 || (role & (role - 1)) != 0)))
            {
                rejectReason = "an in-possession role needs one of FM's positions and a single role bit";
                return false;
            }

            if (change.OopRole is < -31 or > 7)
            {
                rejectReason = "oopRole must be -31 through 7";
                return false;
            }

            if (change.Focus is not null and not (-1 or (>= 5 and <= 32)))
            {
                rejectReason = "focus must be -1 or 5 through 32";
                return false;
            }
        }

        return true;
    }

    private static bool TryValidateFullDump(BridgeRequest request, out string? rejectReason)
    {
        rejectReason = null;
        if (!PlayerDatabaseScopes.TryParse(request.PlayerDatabaseScope, out _))
        {
            rejectReason = "playerDatabaseScope must be one of: men, women, both";
            return false;
        }

        if (request.SourceRequestId is not null
            || request.PlayerUid is not null
            || request.StaffUid is not null
            || request.ExpectedCurrentAbility is not null
            || request.ExpectedPotentialAbility is not null
            || request.CurrentAbilityIncrement is not null
            || request.ExpectedAmbition is not null
            || request.ExpectedProfessionalism is not null
            || request.ExpectedDetermination is not null
            || HasClubActionFields(request))
        {
            rejectReason = "full-dump does not accept player boost fields";
            return false;
        }

        return true;
    }

    private static bool TryValidateBoostStaffCurrentAbility(
        BridgeRequest request,
        out string? rejectReason)
    {
        rejectReason = null;
        if (!string.Equals(
                request.PlayerDatabaseScope,
                PlayerDatabaseScopes.Men,
                StringComparison.Ordinal))
        {
            rejectReason = "playerDatabaseScope is supported only by full-dump";
            return false;
        }

        if (string.IsNullOrWhiteSpace(request.SourceRequestId))
        {
            rejectReason = "sourceRequestId is required for staff boosts";
            return false;
        }

        if (request.StaffUid is not { } staffUid || staffUid == 0)
        {
            rejectReason = "staffUid is required for staff boosts";
            return false;
        }

        if (request.PlayerUid is not null
            || request.CurrentAbilityIncrement is not null
            || request.ExpectedAmbition is not null
            || request.ExpectedProfessionalism is not null
            || request.ExpectedDetermination is not null
            || HasClubActionFields(request))
        {
            rejectReason = "staff CA boost does not accept player boost fields";
            return false;
        }

        if (!IsAbility(request.ExpectedCurrentAbility)
            || !IsAbility(request.ExpectedPotentialAbility)
            || request.ExpectedCurrentAbility > request.ExpectedPotentialAbility)
        {
            rejectReason = "staff expected CA and PA must be 1 through 200 with CA not above PA";
            return false;
        }

        return true;
    }

    private static bool TryValidateBoostCurrentAbility(BridgeRequest request, out string? rejectReason)
    {
        if (!TryValidateBoostPreconditions(request, out rejectReason))
        {
            return false;
        }

        if (request.CurrentAbilityIncrement is not (>= 1 and <= 20))
        {
            rejectReason = "currentAbilityIncrement must be 1 through 20 for boost-current-ability";
            return false;
        }

        if (request.ExpectedAmbition is not null
            || request.ExpectedProfessionalism is not null
            || request.ExpectedDetermination is not null)
        {
            rejectReason = "boost-current-ability does not accept Wonderkid Mentality fields";
            return false;
        }

        return true;
    }

    private static bool TryValidateWonderkidMentality(BridgeRequest request, out string? rejectReason)
    {
        if (!TryValidateBoostPreconditions(request, out rejectReason))
        {
            return false;
        }

        if (request.CurrentAbilityIncrement is not null)
        {
            rejectReason = "wonderkid-mentality does not accept currentAbilityIncrement";
            return false;
        }

        if (!IsMentality(request.ExpectedAmbition)
            || !IsMentality(request.ExpectedProfessionalism)
            || !IsMentality(request.ExpectedDetermination))
        {
            rejectReason = "Wonderkid Mentality values must be null or 1 through 20";
            return false;
        }

        if (!IsEligibleMentality(request.ExpectedAmbition)
            && !IsEligibleMentality(request.ExpectedProfessionalism)
            && !IsEligibleMentality(request.ExpectedDetermination))
        {
            rejectReason = "wonderkid-mentality requires at least one known value from 1 through 10";
            return false;
        }

        return true;
    }

    private static bool TryValidateBoostPreconditions(BridgeRequest request, out string? rejectReason)
    {
        rejectReason = null;
        if (request.StaffUid is not null)
        {
            rejectReason = "player boosts do not accept staffUid";
            return false;
        }

        if (HasClubActionFields(request))
        {
            rejectReason = "player boosts do not accept squad move or staff change fields";
            return false;
        }

        if (!string.Equals(
                request.PlayerDatabaseScope,
                PlayerDatabaseScopes.Men,
                StringComparison.Ordinal))
        {
            rejectReason = "playerDatabaseScope is supported only by full-dump";
            return false;
        }

        if (string.IsNullOrWhiteSpace(request.SourceRequestId))
        {
            rejectReason = "sourceRequestId is required for player boosts";
            return false;
        }

        if (request.PlayerUid is not { } playerUid || playerUid == 0)
        {
            rejectReason = "playerUid is required for player boosts";
            return false;
        }

        if (!IsAbility(request.ExpectedCurrentAbility)
            || !IsAbility(request.ExpectedPotentialAbility))
        {
            rejectReason = "expectedCurrentAbility and expectedPotentialAbility must be 1 through 200";
            return false;
        }

        if (request.ExpectedCurrentAbility > request.ExpectedPotentialAbility)
        {
            rejectReason = "expectedCurrentAbility must not exceed expectedPotentialAbility";
            return false;
        }

        return true;
    }

    private static bool RejectUnsupportedOperation(string operation, out string? rejectReason)
    {
        rejectReason = $"unsupported operation '{operation}'";
        return false;
    }

    private static BridgeRequest CopyRequest(BridgeRequest request, DateTimeOffset createdAtUtc) =>
        new()
        {
            ProtocolVersion = request.ProtocolVersion,
            RequestId = request.RequestId,
            CreatedAtUtc = createdAtUtc,
            Operation = request.Operation,
            PlayerDatabaseScope = request.PlayerDatabaseScope,
            SourceRequestId = request.SourceRequestId,
            PlayerUid = request.PlayerUid,
            StaffUid = request.StaffUid,
            ExpectedCurrentAbility = request.ExpectedCurrentAbility,
            ExpectedPotentialAbility = request.ExpectedPotentialAbility,
            CurrentAbilityIncrement = request.CurrentAbilityIncrement,
            ExpectedAmbition = request.ExpectedAmbition,
            ExpectedProfessionalism = request.ExpectedProfessionalism,
            ExpectedDetermination = request.ExpectedDetermination,
            ClubUid = request.ClubUid,
            SquadMoves = request.SquadMoves?
                .Select(move => new SquadMoveItem { PlayerUid = move.PlayerUid, TargetLevel = move.TargetLevel })
                .ToArray(),
            StaffChanges = request.StaffChanges?
                .Select(change => new StaffChangeItem
                {
                    StaffUid = change.StaffUid,
                    Kind = change.Kind,
                    JobType = change.JobType,
                    TargetLevel = change.TargetLevel,
                })
                .ToArray(),
            TrainingChanges = request.TrainingChanges?
                .Select(change => new TrainingChangeItem
                {
                    PlayerUid = change.PlayerUid,
                    IpPosition = change.IpPosition,
                    IpRole = change.IpRole,
                    OopRole = change.OopRole,
                    Focus = change.Focus,
                })
                .ToArray(),
        };

    private static bool IsAbility(int? value) => value is >= 1 and <= 200;

    private static bool IsMentality(int? value) => value is null || value is >= 1 and <= 20;

    private static bool IsEligibleMentality(int? value) => value is >= 1 and <= 10;

    private static void TryDelete(string path)
    {
        try
        {
            if (File.Exists(path))
            {
                File.Delete(path);
            }
        }
        catch
        {
            // best-effort — next poll may retry delete
        }
    }
}
