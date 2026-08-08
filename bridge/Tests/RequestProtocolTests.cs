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
            var original = DateTimeOffset.Parse("2026-07-28T18:00:00Z");
            var refreshedAt = DateTimeOffset.Parse("2026-07-28T18:30:00Z");
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
            var original = DateTimeOffset.Parse("2026-07-28T18:00:00Z");
            var refreshedAt = DateTimeOffset.Parse("2026-07-28T18:30:00Z");
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

    private static string CreateTempDir()
    {
        var path = Path.Combine(Path.GetTempPath(), "fm-valuescout-tests", Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(path);
        return path;
    }
}
