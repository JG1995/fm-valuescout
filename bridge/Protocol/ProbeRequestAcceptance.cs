using System.Text.Json;

namespace FmDataBridge.Protocol;

/// <summary>
/// Validates and consumes <c>probe-request.json</c> without touching production requests.
/// </summary>
public static class ProbeRequestAcceptance
{
    public const int MaxRequestedUids = 16;

    private static readonly JsonSerializerOptions SerializerOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        PropertyNameCaseInsensitive = true,
        WriteIndented = true,
    };

    public static bool TryValidateForCapture(ProbeRequest request, out string? rejectReason)
    {
        ArgumentNullException.ThrowIfNull(request);
        return TryValidate(
            request,
            DateTimeOffset.MinValue,
            TimeSpan.Zero,
            requireFresh: false,
            out rejectReason);
    }

    public static bool TryAccept(
        string requestPath,
        DateTimeOffset now,
        TimeSpan ttl,
        out ProbeRequest request,
        out string? rejectReason,
        out string? observedRequestId)
    {
        request = null!;
        rejectReason = null;
        observedRequestId = null;

        if (!File.Exists(requestPath))
        {
            rejectReason = "probe request file missing";
            return false;
        }

        string json;
        try
        {
            json = File.ReadAllText(requestPath);
        }
        catch (Exception ex)
        {
            rejectReason = $"probe request read failed: {ex.Message}";
            TryDelete(requestPath);
            return false;
        }

        ProbeRequest? parsed;
        try
        {
            parsed = JsonSerializer.Deserialize<ProbeRequest>(json, SerializerOptions);
        }
        catch (Exception ex)
        {
            rejectReason = $"probe request JSON invalid: {ex.Message}";
            TryDelete(requestPath);
            return false;
        }

        if (parsed is null)
        {
            rejectReason = "probe request JSON empty";
            TryDelete(requestPath);
            return false;
        }

        if (!string.IsNullOrWhiteSpace(parsed.RequestId))
        {
            observedRequestId = parsed.RequestId;
        }

        if (!TryValidate(parsed, now, ttl, requireFresh: true, out rejectReason))
        {
            TryDelete(requestPath);
            return false;
        }

        TryDelete(requestPath);
        request = parsed;
        return true;
    }

    /// <summary>
    /// Keeps a waiting valid probe fresh while the single scan gate serves another request.
    /// </summary>
    public static bool TryRefreshCreatedAtUtc(string requestPath, DateTimeOffset now)
    {
        if (!File.Exists(requestPath))
        {
            return false;
        }

        ProbeRequest? parsed;
        try
        {
            parsed = JsonSerializer.Deserialize<ProbeRequest>(
                File.ReadAllText(requestPath),
                SerializerOptions);
        }
        catch
        {
            return false;
        }

        if (parsed is null || !TryValidate(parsed, now, TimeSpan.Zero, requireFresh: false, out _))
        {
            return false;
        }

        var refreshed = new ProbeRequest
        {
            ProtocolVersion = parsed.ProtocolVersion,
            RequestId = parsed.RequestId,
            CreatedAtUtc = now,
            Uids = parsed.Uids,
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

    private static bool TryValidate(
        ProbeRequest request,
        DateTimeOffset now,
        TimeSpan ttl,
        bool requireFresh,
        out string? reason)
    {
        if (request.ProtocolVersion != ProbeProtocol.ProtocolVersion)
        {
            reason = $"unsupported probe protocol version {request.ProtocolVersion}; expected {ProbeProtocol.ProtocolVersion}";
            return false;
        }

        if (!IsSafeRequestId(request.RequestId))
        {
            reason = "probe requestId must use 1-128 letters, digits, dots, underscores, or hyphens";
            return false;
        }

        if (request.Uids is null || request.Uids.Length is 0 or > MaxRequestedUids)
        {
            reason = $"probe uids must contain 1-{MaxRequestedUids} values";
            return false;
        }

        var seen = new HashSet<uint>();
        foreach (var uid in request.Uids)
        {
            if (uid == 0 || uid == uint.MaxValue)
            {
                reason = "probe uids must be valid FM player UIDs";
                return false;
            }

            if (!seen.Add(uid))
            {
                reason = "probe uids must not contain duplicates";
                return false;
            }
        }

        if (requireFresh && !RequestAcceptance.IsFresh(request.CreatedAtUtc, now, ttl))
        {
            reason = $"stale probe request (age exceeds {ttl.TotalSeconds:0}s TTL)";
            return false;
        }

        reason = null;
        return true;
    }

    private static bool IsSafeRequestId(string requestId)
    {
        if (string.IsNullOrEmpty(requestId) || requestId.Length > 128)
        {
            return false;
        }

        foreach (var character in requestId)
        {
            if (!((character >= 'a' && character <= 'z')
                    || (character >= 'A' && character <= 'Z')
                    || (character >= '0' && character <= '9'))
                && character is not '.' and not '_' and not '-')
            {
                return false;
            }
        }

        return true;
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
            // Best effort: the next poll can retry the same request.
        }
    }
}
