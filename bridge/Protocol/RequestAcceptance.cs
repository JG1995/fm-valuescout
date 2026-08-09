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
            BridgeProtocol.OperationWonderkidMentality => TryValidateWonderkidMentality(request, out rejectReason),
            _ => RejectUnsupportedOperation(request.Operation, out rejectReason),
        };
    }

    private static bool TryValidateFullDump(BridgeRequest request, out string? rejectReason)
    {
        rejectReason = null;
        if (request.MaxAccepted is <= 0)
        {
            rejectReason = "maxAccepted must be null or a positive integer";
            return false;
        }

        if (!PlayerDatabaseScopes.TryParse(request.PlayerDatabaseScope, out _))
        {
            rejectReason = "playerDatabaseScope must be one of: men, women, both";
            return false;
        }

        if (request.SourceRequestId is not null
            || request.PlayerUid is not null
            || request.ExpectedCurrentAbility is not null
            || request.ExpectedPotentialAbility is not null
            || request.CurrentAbilityIncrement is not null
            || request.ExpectedAmbition is not null
            || request.ExpectedProfessionalism is not null
            || request.ExpectedDetermination is not null)
        {
            rejectReason = "full-dump does not accept player boost fields";
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

        if (request.CurrentAbilityIncrement is not 5 and not 10)
        {
            rejectReason = "currentAbilityIncrement must be 5 or 10 for boost-current-ability";
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
        if (request.MaxAccepted is not null)
        {
            rejectReason = "player boost requests do not accept maxAccepted";
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
            MaxAccepted = request.MaxAccepted,
            PlayerDatabaseScope = request.PlayerDatabaseScope,
            SourceRequestId = request.SourceRequestId,
            PlayerUid = request.PlayerUid,
            ExpectedCurrentAbility = request.ExpectedCurrentAbility,
            ExpectedPotentialAbility = request.ExpectedPotentialAbility,
            CurrentAbilityIncrement = request.CurrentAbilityIncrement,
            ExpectedAmbition = request.ExpectedAmbition,
            ExpectedProfessionalism = request.ExpectedProfessionalism,
            ExpectedDetermination = request.ExpectedDetermination,
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
