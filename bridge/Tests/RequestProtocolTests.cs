using System.Text.Json;
using FmDataBridge.Protocol;
using Xunit;

namespace FmDataBridge.Tests;

public sealed class RequestProtocolTests
{
    private static readonly TimeSpan Ttl = TimeSpan.FromSeconds(BridgeProtocol.RequestTtlSeconds);

    [Fact]
    public void Fresh_request_is_accepted_and_file_deleted()
    {
        var dir = CreateTempDir();
        try
        {
            var path = BridgePaths.GetRequestPath(dir);
            var now = DateTimeOffset.Parse("2026-07-28T18:30:00Z");
            WriteRequest(
                path,
                protocolVersion: 1,
                requestId: "req-1",
                createdAtUtc: now.AddSeconds(-5),
                operation: BridgeProtocol.OperationFullDump);

            Assert.True(
                RequestAcceptance.TryAccept(
                    path,
                    now,
                    Ttl,
                    out var request,
                    out var rejectReason,
                    out var observedRequestId));
            Assert.Null(rejectReason);
            Assert.Equal("req-1", request.RequestId);
            Assert.Equal("req-1", observedRequestId);
            Assert.Equal(BridgeProtocol.OperationFullDump, request.Operation);
            Assert.Equal(PlayerDatabaseScopes.Men, request.PlayerDatabaseScope);
            Assert.False(File.Exists(path));
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    [Fact]
    public void Stale_request_is_rejected_with_observed_request_id()
    {
        var dir = CreateTempDir();
        try
        {
            var path = BridgePaths.GetRequestPath(dir);
            var now = DateTimeOffset.Parse("2026-07-28T18:30:00Z");
            WriteRequest(
                path,
                protocolVersion: 1,
                requestId: "req-stale",
                createdAtUtc: now.AddSeconds(-(BridgeProtocol.RequestTtlSeconds + 1)),
                operation: BridgeProtocol.OperationFullDump);

            Assert.False(
                RequestAcceptance.TryAccept(
                    path,
                    now,
                    Ttl,
                    out _,
                    out var rejectReason,
                    out var observedRequestId));
            Assert.Contains("stale", rejectReason, StringComparison.OrdinalIgnoreCase);
            Assert.Equal("req-stale", observedRequestId);
            Assert.False(File.Exists(path));
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    [Fact]
    public void Unsupported_operation_is_rejected_with_observed_request_id()
    {
        var dir = CreateTempDir();
        try
        {
            var path = BridgePaths.GetRequestPath(dir);
            var now = DateTimeOffset.Parse("2026-07-28T18:30:00Z");
            WriteRequest(
                path,
                protocolVersion: 1,
                requestId: "req-2",
                createdAtUtc: now,
                operation: "partial-dump");

            Assert.False(
                RequestAcceptance.TryAccept(
                    path,
                    now,
                    Ttl,
                    out _,
                    out var rejectReason,
                    out var observedRequestId));
            Assert.Contains("unsupported operation", rejectReason, StringComparison.OrdinalIgnoreCase);
            Assert.Equal("req-2", observedRequestId);
            Assert.False(File.Exists(path));
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    [Fact]
    public void Refresh_created_at_keeps_waiting_request_file()
    {
        var dir = CreateTempDir();
        try
        {
            var path = BridgePaths.GetRequestPath(dir);
            var refreshedAt = DateTimeOffset.Parse("2026-07-28T18:30:00Z");
            var original = refreshedAt.AddSeconds(-5);
            WriteRequest(
                path,
                protocolVersion: 1,
                requestId: "req-wait",
                createdAtUtc: original,
                operation: BridgeProtocol.OperationFullDump);

            Assert.True(RequestAcceptance.TryRefreshCreatedAtUtc(path, refreshedAt));
            Assert.True(File.Exists(path));

            using var doc = JsonDocument.Parse(File.ReadAllText(path));
            Assert.Equal("req-wait", doc.RootElement.GetProperty("requestId").GetString());
            var created = DateTimeOffset.Parse(doc.RootElement.GetProperty("createdAtUtc").GetString()!);
            Assert.Equal(refreshedAt, created);

            Assert.True(
                RequestAcceptance.TryAccept(
                    path,
                    refreshedAt,
                    Ttl,
                    out var request,
                    out _,
                    out _));
            Assert.Equal("req-wait", request.RequestId);
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    [Fact]
    public void IsFresh_rejects_future_skew_beyond_now()
    {
        var now = DateTimeOffset.Parse("2026-07-28T18:30:00Z");
        Assert.False(RequestAcceptance.IsFresh(now.AddSeconds(1), now, Ttl));
        Assert.True(RequestAcceptance.IsFresh(now, now, Ttl));
        Assert.True(RequestAcceptance.IsFresh(now.AddSeconds(-Ttl.TotalSeconds), now, Ttl));
    }

    [Fact]
    public void Accept_rejects_non_positive_max_accepted()
    {
        var dir = CreateTempDir();
        try
        {
            var path = BridgePaths.GetRequestPath(dir);
            var now = DateTimeOffset.Parse("2026-07-28T18:30:00Z");
            WriteRequest(
                path,
                protocolVersion: 1,
                requestId: "req-bad-cap",
                createdAtUtc: now,
                operation: BridgeProtocol.OperationFullDump,
                maxAcceptedJson: "0");

            Assert.False(
                RequestAcceptance.TryAccept(
                    path,
                    now,
                    Ttl,
                    out _,
                    out var rejectReason,
                    out var observedRequestId));
            Assert.Contains("maxAccepted", rejectReason, StringComparison.OrdinalIgnoreCase);
            Assert.Equal("req-bad-cap", observedRequestId);
            Assert.False(File.Exists(path));
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    [Fact]
    public void Accept_preserves_positive_max_accepted()
    {
        var dir = CreateTempDir();
        try
        {
            var path = BridgePaths.GetRequestPath(dir);
            var now = DateTimeOffset.Parse("2026-07-28T18:30:00Z");
            WriteRequest(
                path,
                protocolVersion: 1,
                requestId: "req-cap",
                createdAtUtc: now,
                operation: BridgeProtocol.OperationFullDump,
                maxAcceptedJson: "500");

            Assert.True(
                RequestAcceptance.TryAccept(
                    path,
                    now,
                    Ttl,
                    out var request,
                    out _,
                    out _));
            Assert.Equal(500, request.MaxAccepted);
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    [Fact]
    public void Accept_preserves_null_max_accepted_as_unlimited()
    {
        var dir = CreateTempDir();
        try
        {
            var path = BridgePaths.GetRequestPath(dir);
            var now = DateTimeOffset.Parse("2026-07-28T18:30:00Z");
            WriteRequest(
                path,
                protocolVersion: 1,
                requestId: "req-unlimited",
                createdAtUtc: now,
                operation: BridgeProtocol.OperationFullDump,
                maxAcceptedJson: "null");

            Assert.True(
                RequestAcceptance.TryAccept(
                    path,
                    now,
                    Ttl,
                    out var request,
                    out _,
                    out _));
            Assert.Null(request.MaxAccepted);
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    [Fact]
    public void Accept_omitted_max_accepted_is_unlimited()
    {
        var dir = CreateTempDir();
        try
        {
            var path = BridgePaths.GetRequestPath(dir);
            var now = DateTimeOffset.Parse("2026-07-28T18:30:00Z");
            WriteRequest(
                path,
                protocolVersion: 1,
                requestId: "req-omit",
                createdAtUtc: now,
                operation: BridgeProtocol.OperationFullDump);

            Assert.True(
                RequestAcceptance.TryAccept(
                    path,
                    now,
                    Ttl,
                    out var request,
                    out _,
                    out _));
            Assert.Null(request.MaxAccepted);
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    [Theory]
    [InlineData("men")]
    [InlineData("women")]
    [InlineData("both")]
    public void Accept_preserves_supported_player_database_scopes(string scope)
    {
        var dir = CreateTempDir();
        try
        {
            var path = BridgePaths.GetRequestPath(dir);
            var now = DateTimeOffset.Parse("2026-07-28T18:30:00Z");
            WriteRequest(
                path,
                protocolVersion: 1,
                requestId: $"req-{scope}",
                createdAtUtc: now,
                operation: BridgeProtocol.OperationFullDump,
                playerDatabaseScopeJson: $"\"{scope}\"");

            Assert.True(
                RequestAcceptance.TryAccept(
                    path,
                    now,
                    Ttl,
                    out var request,
                    out _,
                    out _));
            Assert.Equal(scope, request.PlayerDatabaseScope);
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    [Fact]
    public void Accept_rejects_invalid_player_database_scope()
    {
        var dir = CreateTempDir();
        try
        {
            var path = BridgePaths.GetRequestPath(dir);
            var now = DateTimeOffset.Parse("2026-07-28T18:30:00Z");
            WriteRequest(
                path,
                protocolVersion: 1,
                requestId: "req-invalid-scope",
                createdAtUtc: now,
                operation: BridgeProtocol.OperationFullDump,
                playerDatabaseScopeJson: "\"mixed\"");

            Assert.False(
                RequestAcceptance.TryAccept(
                    path,
                    now,
                    Ttl,
                    out _,
                    out var rejectReason,
                    out var observedRequestId));
            Assert.Contains("playerDatabaseScope", rejectReason, StringComparison.Ordinal);
            Assert.Equal("req-invalid-scope", observedRequestId);
            Assert.False(File.Exists(path));
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    [Fact]
    public void Refresh_created_at_preserves_max_accepted()
    {
        var dir = CreateTempDir();
        try
        {
            var path = BridgePaths.GetRequestPath(dir);
            var refreshedAt = DateTimeOffset.Parse("2026-07-28T18:30:00Z");
            var original = refreshedAt.AddSeconds(-5);
            WriteRequest(
                path,
                protocolVersion: 1,
                requestId: "req-wait-cap",
                createdAtUtc: original,
                operation: BridgeProtocol.OperationFullDump,
                maxAcceptedJson: "250",
                playerDatabaseScopeJson: "\"women\"");

            Assert.True(RequestAcceptance.TryRefreshCreatedAtUtc(path, refreshedAt));

            using var doc = JsonDocument.Parse(File.ReadAllText(path));
            Assert.Equal(250, doc.RootElement.GetProperty("maxAccepted").GetInt32());
            Assert.Equal("women", doc.RootElement.GetProperty("playerDatabaseScope").GetString());

            Assert.True(
                RequestAcceptance.TryAccept(
                    path,
                    refreshedAt,
                    Ttl,
                    out var request,
                    out _,
                    out _));
            Assert.Equal(250, request.MaxAccepted);
            Assert.Equal(PlayerDatabaseScopes.Women, request.PlayerDatabaseScope);
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    [Fact]
    public void Accept_preserves_the_closed_current_ability_boost_preconditions()
    {
        var dir = CreateTempDir();
        try
        {
            var path = BridgePaths.GetRequestPath(dir);
            var now = DateTimeOffset.Parse("2026-08-09T12:00:00Z");
            WriteBoostRequest(
                path,
                new BridgeRequest
                {
                    ProtocolVersion = BridgeProtocol.ProtocolVersion,
                    RequestId = "boost-ca-1",
                    CreatedAtUtc = now,
                    Operation = BridgeProtocol.OperationBoostCurrentAbility,
                    SourceRequestId = "scan-1",
                    PlayerUid = 42,
                    ExpectedCurrentAbility = 120,
                    ExpectedPotentialAbility = 150,
                    CurrentAbilityIncrement = 5,
                });

            Assert.True(
                RequestAcceptance.TryAccept(
                    path,
                    now,
                    Ttl,
                    out var request,
                    out _,
                    out _));
            Assert.Equal(BridgeProtocol.OperationBoostCurrentAbility, request.Operation);
            Assert.Equal("scan-1", request.SourceRequestId);
            Assert.Equal(42u, request.PlayerUid);
            Assert.Equal(120, request.ExpectedCurrentAbility);
            Assert.Equal(150, request.ExpectedPotentialAbility);
            Assert.Equal(5, request.CurrentAbilityIncrement);
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    [Fact]
    public void Accept_rejects_an_arbitrary_current_ability_increment()
    {
        var dir = CreateTempDir();
        try
        {
            var path = BridgePaths.GetRequestPath(dir);
            var now = DateTimeOffset.Parse("2026-08-09T12:00:00Z");
            WriteBoostRequest(
                path,
                new BridgeRequest
                {
                    ProtocolVersion = BridgeProtocol.ProtocolVersion,
                    RequestId = "boost-ca-invalid",
                    CreatedAtUtc = now,
                    Operation = BridgeProtocol.OperationBoostCurrentAbility,
                    SourceRequestId = "scan-1",
                    PlayerUid = 42,
                    ExpectedCurrentAbility = 120,
                    ExpectedPotentialAbility = 150,
                    CurrentAbilityIncrement = 6,
                });

            Assert.False(
                RequestAcceptance.TryAccept(
                    path,
                    now,
                    Ttl,
                    out _,
                    out var reason,
                    out var observedRequestId));
            Assert.Contains("currentAbilityIncrement", reason, StringComparison.Ordinal);
            Assert.Equal("boost-ca-invalid", observedRequestId);
            Assert.False(File.Exists(path));
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    [Fact]
    public void Accept_rejects_current_ability_with_wonderkid_mentality_fields()
    {
        var dir = CreateTempDir();
        try
        {
            var path = BridgePaths.GetRequestPath(dir);
            var now = DateTimeOffset.Parse("2026-08-09T12:00:00Z");
            WriteBoostRequest(
                path,
                new BridgeRequest
                {
                    ProtocolVersion = BridgeProtocol.ProtocolVersion,
                    RequestId = "boost-ca-with-mentality",
                    CreatedAtUtc = now,
                    Operation = BridgeProtocol.OperationBoostCurrentAbility,
                    SourceRequestId = "scan-1",
                    PlayerUid = 42,
                    ExpectedCurrentAbility = 120,
                    ExpectedPotentialAbility = 150,
                    CurrentAbilityIncrement = 5,
                    ExpectedAmbition = 10,
                });

            Assert.False(
                RequestAcceptance.TryAccept(
                    path,
                    now,
                    Ttl,
                    out _,
                    out var reason,
                    out _));
            Assert.Contains("does not accept", reason, StringComparison.Ordinal);
            Assert.False(File.Exists(path));
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    [Fact]
    public void Accept_rejects_wonderkid_mentality_without_a_known_eligible_value()
    {
        var dir = CreateTempDir();
        try
        {
            var path = BridgePaths.GetRequestPath(dir);
            var now = DateTimeOffset.Parse("2026-08-09T12:00:00Z");
            WriteBoostRequest(
                path,
                new BridgeRequest
                {
                    ProtocolVersion = BridgeProtocol.ProtocolVersion,
                    RequestId = "boost-mentality-no-eligible-value",
                    CreatedAtUtc = now,
                    Operation = BridgeProtocol.OperationWonderkidMentality,
                    SourceRequestId = "scan-1",
                    PlayerUid = 42,
                    ExpectedCurrentAbility = 120,
                    ExpectedPotentialAbility = 150,
                    ExpectedAmbition = null,
                    ExpectedProfessionalism = 11,
                    ExpectedDetermination = 20,
                });

            Assert.False(
                RequestAcceptance.TryAccept(
                    path,
                    now,
                    Ttl,
                    out _,
                    out var reason,
                    out _));
            Assert.Contains("at least one known value", reason, StringComparison.Ordinal);
            Assert.False(File.Exists(path));
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    [Fact]
    public void Refresh_created_at_preserves_wonderkid_mentality_preconditions()
    {
        var dir = CreateTempDir();
        try
        {
            var path = BridgePaths.GetRequestPath(dir);
            var original = DateTimeOffset.Parse("2026-08-09T12:00:00Z");
            var refreshedAt = original.AddSeconds(5);
            WriteBoostRequest(
                path,
                new BridgeRequest
                {
                    ProtocolVersion = BridgeProtocol.ProtocolVersion,
                    RequestId = "boost-mentality-1",
                    CreatedAtUtc = original,
                    Operation = BridgeProtocol.OperationWonderkidMentality,
                    SourceRequestId = "scan-1",
                    PlayerUid = 42,
                    ExpectedCurrentAbility = 120,
                    ExpectedPotentialAbility = 150,
                    ExpectedAmbition = 10,
                    ExpectedProfessionalism = 11,
                    ExpectedDetermination = null,
                });

            Assert.True(RequestAcceptance.TryRefreshCreatedAtUtc(path, refreshedAt));
            Assert.True(
                RequestAcceptance.TryAccept(
                    path,
                    refreshedAt,
                    Ttl,
                    out var request,
                    out _,
                    out _));
            Assert.Equal(BridgeProtocol.OperationWonderkidMentality, request.Operation);
            Assert.Equal("scan-1", request.SourceRequestId);
            Assert.Equal(42u, request.PlayerUid);
            Assert.Equal(10, request.ExpectedAmbition);
            Assert.Equal(11, request.ExpectedProfessionalism);
            Assert.Null(request.ExpectedDetermination);
            Assert.Null(request.CurrentAbilityIncrement);
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    [Fact]
    public void Refresh_created_at_does_not_resurrect_a_stale_boost_request()
    {
        var dir = CreateTempDir();
        try
        {
            var path = BridgePaths.GetRequestPath(dir);
            var now = DateTimeOffset.Parse("2026-08-09T12:00:00Z");
            var staleAt = now.AddSeconds(-(BridgeProtocol.RequestTtlSeconds + 1));
            WriteBoostRequest(
                path,
                new BridgeRequest
                {
                    ProtocolVersion = BridgeProtocol.ProtocolVersion,
                    RequestId = "boost-stale",
                    CreatedAtUtc = staleAt,
                    Operation = BridgeProtocol.OperationWonderkidMentality,
                    SourceRequestId = "scan-1",
                    PlayerUid = 42,
                    ExpectedCurrentAbility = 120,
                    ExpectedPotentialAbility = 150,
                    ExpectedAmbition = 10,
                    ExpectedProfessionalism = null,
                    ExpectedDetermination = null,
                });

            Assert.False(RequestAcceptance.TryRefreshCreatedAtUtc(path, now));
            using var doc = JsonDocument.Parse(File.ReadAllText(path));
            Assert.Equal(staleAt, DateTimeOffset.Parse(doc.RootElement.GetProperty("createdAtUtc").GetString()!));

            Assert.False(
                RequestAcceptance.TryAccept(
                    path,
                    now,
                    Ttl,
                    out _,
                    out var reason,
                    out _));
            Assert.Contains("stale", reason, StringComparison.OrdinalIgnoreCase);
            Assert.False(File.Exists(path));
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    [Fact]
    public void Force_scan_requests_receive_distinct_provenance_ids()
    {
        var now = DateTimeOffset.Parse("2026-08-09T12:00:00Z");

        var first = ForceScanRequestFactory.Create(now);
        var second = ForceScanRequestFactory.Create(now);

        Assert.StartsWith("force-scan-", first.RequestId, StringComparison.Ordinal);
        Assert.StartsWith("force-scan-", second.RequestId, StringComparison.Ordinal);
        Assert.NotEqual(first.RequestId, second.RequestId);
        Assert.Equal(BridgeProtocol.OperationFullDump, first.Operation);
        Assert.Equal(PlayerDatabaseScopes.Men, first.PlayerDatabaseScope);
    }

    private static void WriteRequest(
        string path,
        int protocolVersion,
        string requestId,
        DateTimeOffset createdAtUtc,
        string operation,
        string? maxAcceptedJson = null,
        string? playerDatabaseScopeJson = null)
    {
        Directory.CreateDirectory(Path.GetDirectoryName(path)!);
        var optionalProperties = new[]
            {
                maxAcceptedJson is null ? null : $"\"maxAccepted\": {maxAcceptedJson}",
                playerDatabaseScopeJson is null
                    ? null
                    : $"\"playerDatabaseScope\": {playerDatabaseScopeJson}",
            }
            .Where(value => value is not null)
            .Cast<string>()
            .ToList();
        var optionalLines = optionalProperties.Count == 0
            ? ""
            : $",\n  {string.Join(",\n  ", optionalProperties)}";
        File.WriteAllText(
            path,
            "{\n"
            + $"  \"protocolVersion\": {protocolVersion},\n"
            + $"  \"requestId\": \"{requestId}\",\n"
            + $"  \"createdAtUtc\": \"{createdAtUtc:O}\",\n"
            + $"  \"operation\": \"{operation}\"{optionalLines}\n"
            + "}\n");
    }

    private static void WriteBoostRequest(string path, BridgeRequest request)
    {
        Directory.CreateDirectory(Path.GetDirectoryName(path)!);
        File.WriteAllText(
            path,
            JsonSerializer.Serialize(
                request,
                new JsonSerializerOptions { PropertyNamingPolicy = JsonNamingPolicy.CamelCase }));
    }

    private static string CreateTempDir()
    {
        var path = Path.Combine(Path.GetTempPath(), "fm-valuescout-tests", Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(path);
        return path;
    }
}
