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

        BridgeRequest? parsed;
        try
        {
            parsed = JsonSerializer.Deserialize<BridgeRequest>(json, SerializerOptions);
        }
        catch (Exception ex)
        {
            rejectReason = $"request JSON invalid: {ex.Message}";
            TryDelete(requestPath);
            return false;
        }

        if (parsed is null)
        {
            rejectReason = "request JSON empty";
            TryDelete(requestPath);
            return false;
        }

        if (!string.IsNullOrWhiteSpace(parsed.RequestId))
        {
            observedRequestId = parsed.RequestId;
        }

        if (parsed.ProtocolVersion != BridgeProtocol.ProtocolVersion)
        {
            rejectReason =
                $"unsupported request protocol version {parsed.ProtocolVersion}; expected {BridgeProtocol.ProtocolVersion}";
            TryDelete(requestPath);
            return false;
        }

        if (string.IsNullOrWhiteSpace(parsed.RequestId))
        {
            rejectReason = "requestId is required";
            TryDelete(requestPath);
            return false;
        }

        if (!string.Equals(
                parsed.Operation,
                BridgeProtocol.OperationFullDump,
                StringComparison.Ordinal))
        {
            rejectReason = $"unsupported operation '{parsed.Operation}'";
            TryDelete(requestPath);
            return false;
        }

        if (parsed.MaxAccepted is <= 0)
        {
            rejectReason = "maxAccepted must be null or a positive integer";
            TryDelete(requestPath);
            return false;
        }

        if (!PlayerDatabaseScopes.TryParse(parsed.PlayerDatabaseScope, out var playerDatabaseScope))
        {
            rejectReason = "playerDatabaseScope must be one of: men, women, both";
            TryDelete(requestPath);
            return false;
        }

        if (!IsFresh(parsed.CreatedAtUtc, now, ttl))
        {
            rejectReason =
                $"stale request (age exceeds {ttl.TotalSeconds:0}s TTL)";
            TryDelete(requestPath);
            return false;
        }

        TryDelete(requestPath);
        request = new BridgeRequest
        {
            ProtocolVersion = parsed.ProtocolVersion,
            RequestId = parsed.RequestId,
            CreatedAtUtc = parsed.CreatedAtUtc,
            Operation = parsed.Operation,
            MaxAccepted = parsed.MaxAccepted,
            PlayerDatabaseScope = PlayerDatabaseScopes.ToWireValue(playerDatabaseScope),
        };
        return true;
    }

    /// <summary>
    /// While a scan is in progress, bump <c>createdAtUtc</c> so a waiting request does not TTL-expire.
    /// Returns true when the file was rewritten.
    /// </summary>
    public static bool TryRefreshCreatedAtUtc(string requestPath, DateTimeOffset now)
    {
        if (!File.Exists(requestPath))
        {
            return false;
        }

        string json;
        try
        {
            json = File.ReadAllText(requestPath);
        }
        catch
        {
            return false;
        }

        BridgeRequest? parsed;
        try
        {
            parsed = JsonSerializer.Deserialize<BridgeRequest>(json, SerializerOptions);
        }
        catch
        {
            return false;
        }

        if (parsed is null
            || string.IsNullOrWhiteSpace(parsed.RequestId)
            || parsed.ProtocolVersion != BridgeProtocol.ProtocolVersion
            || !string.Equals(
                parsed.Operation,
                BridgeProtocol.OperationFullDump,
                StringComparison.Ordinal)
            || !PlayerDatabaseScopes.TryParse(parsed.PlayerDatabaseScope, out var playerDatabaseScope))
        {
            return false;
        }

        var refreshed = new BridgeRequest
        {
            ProtocolVersion = parsed.ProtocolVersion,
            RequestId = parsed.RequestId,
            CreatedAtUtc = now,
            Operation = parsed.Operation,
            MaxAccepted = parsed.MaxAccepted,
            PlayerDatabaseScope = PlayerDatabaseScopes.ToWireValue(playerDatabaseScope),
        };

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
