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

    [Theory]
    [InlineData(1)]
    [InlineData(5)]
    [InlineData(20)]
    public void Accept_preserves_the_closed_current_ability_boost_preconditions(int increment)
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
                    CurrentAbilityIncrement = increment,
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
            Assert.Equal(increment, request.CurrentAbilityIncrement);
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    [Fact]
    public void Accept_preserves_only_the_closed_staff_boost_preconditions()
    {
        var dir = CreateTempDir();
        try
        {
            var path = BridgePaths.GetRequestPath(dir);
            var now = DateTimeOffset.Parse("2026-08-16T12:00:00Z");
            WriteBoostRequest(
                path,
                new BridgeRequest
                {
                    ProtocolVersion = BridgeProtocol.ProtocolVersion,
                    RequestId = "staff-boost-1",
                    CreatedAtUtc = now,
                    Operation = BridgeProtocol.OperationBoostStaffCurrentAbility,
                    SourceRequestId = "scan-1",
                    StaffUid = 42,
                    ExpectedCurrentAbility = 120,
                    ExpectedPotentialAbility = 150,
                });

            Assert.True(
                RequestAcceptance.TryAccept(
                    path,
                    now,
                    Ttl,
                    out var request,
                    out _,
                    out _));
            Assert.Equal(BridgeProtocol.OperationBoostStaffCurrentAbility, request.Operation);
            Assert.Equal("scan-1", request.SourceRequestId);
            Assert.Equal(42u, request.StaffUid);
            Assert.Null(request.PlayerUid);
            Assert.Null(request.CurrentAbilityIncrement);
            Assert.Equal(120, request.ExpectedCurrentAbility);
            Assert.Equal(150, request.ExpectedPotentialAbility);
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    [Theory]
    [InlineData(true, false)]
    [InlineData(false, true)]
    public void Accept_rejects_player_fields_or_missing_staff_identity_for_staff_boosts(
        bool includePlayerUid,
        bool omitStaffUid)
    {
        var dir = CreateTempDir();
        try
        {
            var path = BridgePaths.GetRequestPath(dir);
            var now = DateTimeOffset.Parse("2026-08-16T12:00:00Z");
            WriteBoostRequest(
                path,
                new BridgeRequest
                {
                    ProtocolVersion = BridgeProtocol.ProtocolVersion,
                    RequestId = "staff-boost-invalid",
                    CreatedAtUtc = now,
                    Operation = BridgeProtocol.OperationBoostStaffCurrentAbility,
                    SourceRequestId = "scan-1",
                    StaffUid = omitStaffUid ? null : 42,
                    PlayerUid = includePlayerUid ? 42u : null,
                    ExpectedCurrentAbility = 120,
                    ExpectedPotentialAbility = 150,
                });

            Assert.False(
                RequestAcceptance.TryAccept(
                    path,
                    now,
                    Ttl,
                    out _,
                    out var reason,
                    out _));
            Assert.Contains(includePlayerUid ? "player boost fields" : "staffUid", reason, StringComparison.Ordinal);
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    [Theory]
    [InlineData(0)]
    [InlineData(21)]
    public void Accept_rejects_a_current_ability_increment_outside_one_through_twenty(int increment)
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
                    CurrentAbilityIncrement = increment,
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
    public void Accept_rejects_staff_identity_on_a_player_boost_operation()
    {
        var dir = CreateTempDir();
        try
        {
            var path = BridgePaths.GetRequestPath(dir);
            var now = DateTimeOffset.Parse("2026-08-16T12:00:00Z");
            WriteBoostRequest(
                path,
                new BridgeRequest
                {
                    ProtocolVersion = BridgeProtocol.ProtocolVersion,
                    RequestId = "player-boost-with-staff",
                    CreatedAtUtc = now,
                    Operation = BridgeProtocol.OperationBoostCurrentAbility,
                    SourceRequestId = "scan-1",
                    PlayerUid = 42,
                    StaffUid = 84,
                    ExpectedCurrentAbility = 120,
                    ExpectedPotentialAbility = 150,
                    CurrentAbilityIncrement = 5,
                });

            Assert.False(
                RequestAcceptance.TryAccept(
                    path,
                    now,
                    Ttl,
                    out _,
                    out var reason,
                    out _));
            Assert.Contains("staffUid", reason, StringComparison.Ordinal);
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

    [Fact]
    public void Accept_preserves_a_squad_move_request()
    {
        var dir = CreateTempDir();
        try
        {
            var path = BridgePaths.GetRequestPath(dir);
            var now = DateTimeOffset.Parse("2026-11-20T12:00:00Z");
            WriteBoostRequest(path, SquadMoveRequest(now, Moves((77, "reserve"), (78, "youth"))));

            Assert.True(RequestAcceptance.TryAccept(path, now, Ttl, out var request, out _, out _));
            Assert.Equal(BridgeProtocol.OperationMoveSquadPlayers, request.Operation);
            Assert.Equal("scan-1", request.SourceRequestId);
            Assert.Equal(439u, request.ClubUid);
            Assert.Equal(
                new[] { (77u, "reserve"), (78u, "youth") },
                request.SquadMoves!.Select(move => (move.PlayerUid, move.TargetLevel)));
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    [Theory]
    [InlineData("no-club", "clubUid is required")]
    [InlineData("no-moves", "squadMoves must list")]
    [InlineData("duplicate", "distinct playerUid")]
    [InlineData("bad-level", "targetLevel must be")]
    [InlineData("boost-field", "do not accept boost fields")]
    [InlineData("too-many", "squadMoves must list")]
    public void Accept_rejects_an_invalid_squad_move_request(string defect, string expected)
    {
        var dir = CreateTempDir();
        try
        {
            var path = BridgePaths.GetRequestPath(dir);
            var now = DateTimeOffset.Parse("2026-11-20T12:00:00Z");
            var valid = SquadMoveRequest(now, Moves((77, "reserve")));
            var request = defect switch
            {
                "no-club" => Copy(valid, clubUid: null),
                "no-moves" => Copy(valid, moves: Moves()),
                "duplicate" => Copy(valid, moves: Moves((77, "reserve"), (77, "youth"))),
                "bad-level" => Copy(valid, moves: Moves((77, "reserves"))),
                "boost-field" => Copy(valid, playerUid: 77),
                "too-many" => Copy(
                    valid,
                    moves: Enumerable.Range(1, BridgeProtocol.MaxSquadMoves + 1)
                        .Select(uid => new SquadMoveItem { PlayerUid = (uint)uid, TargetLevel = "youth" })
                        .ToArray()),
                _ => throw new ArgumentOutOfRangeException(nameof(defect)),
            };
            WriteBoostRequest(path, request);

            Assert.False(RequestAcceptance.TryAccept(path, now, Ttl, out _, out var reason, out _));
            Assert.Contains(expected, reason, StringComparison.Ordinal);
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    [Theory]
    [InlineData(BridgeProtocol.OperationFullDump)]
    [InlineData(BridgeProtocol.OperationBoostCurrentAbility)]
    [InlineData(BridgeProtocol.OperationBoostStaffCurrentAbility)]
    public void Accept_rejects_squad_move_fields_on_other_operations(string operation)
    {
        var dir = CreateTempDir();
        try
        {
            var path = BridgePaths.GetRequestPath(dir);
            var now = DateTimeOffset.Parse("2026-11-20T12:00:00Z");
            WriteBoostRequest(
                path,
                new BridgeRequest
                {
                    ProtocolVersion = BridgeProtocol.ProtocolVersion,
                    RequestId = "mixed",
                    CreatedAtUtc = now,
                    Operation = operation,
                    SourceRequestId = operation == BridgeProtocol.OperationFullDump ? null : "scan-1",
                    PlayerUid = operation == BridgeProtocol.OperationBoostCurrentAbility ? 77u : null,
                    StaffUid = operation == BridgeProtocol.OperationBoostStaffCurrentAbility ? 42u : null,
                    ExpectedCurrentAbility = operation == BridgeProtocol.OperationFullDump ? null : 120,
                    ExpectedPotentialAbility = operation == BridgeProtocol.OperationFullDump ? null : 150,
                    CurrentAbilityIncrement = operation == BridgeProtocol.OperationBoostCurrentAbility ? 5 : null,
                    SquadMoves = Moves((77, "reserve")),
                });

            Assert.False(RequestAcceptance.TryAccept(path, now, Ttl, out _, out _, out _));
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    [Fact]
    public void Accept_preserves_a_staff_change_request()
    {
        var dir = CreateTempDir();
        try
        {
            var path = BridgePaths.GetRequestPath(dir);
            var now = DateTimeOffset.Parse("2026-11-20T12:00:00Z");
            WriteBoostRequest(
                path,
                StaffChangeRequest(now, Sack(41), Hire(42, 20, "senior"), Change(43, 2, "youth")));

            Assert.True(RequestAcceptance.TryAccept(path, now, Ttl, out var request, out _, out _));
            Assert.Equal(BridgeProtocol.OperationApplyStaffChanges, request.Operation);
            Assert.Equal(439u, request.ClubUid);
            Assert.Equal(
                new[]
                {
                    (41u, "sack", (int?)null, (string?)null),
                    (42u, "hire", 20, "senior"),
                    (43u, "change", 2, "youth"),
                },
                request.StaffChanges!.Select(change => (change.StaffUid, change.Kind, change.JobType, change.TargetLevel)));
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    [Theory]
    [InlineData("no-club", "clubUid is required")]
    [InlineData("no-changes", "staffChanges must list")]
    [InlineData("duplicate", "distinct staffUid")]
    [InlineData("bad-kind", "kind must be sack, hire, or change")]
    [InlineData("sack-with-job", "takes no job or team")]
    [InlineData("hire-without-job", "needs a jobType")]
    [InlineData("bad-level", "targetLevel must be")]
    [InlineData("squad-moves", "do not accept boost or squad move fields")]
    [InlineData("too-many", "staffChanges must list")]
    public void Accept_rejects_an_invalid_staff_change_request(string defect, string expected)
    {
        var dir = CreateTempDir();
        try
        {
            var path = BridgePaths.GetRequestPath(dir);
            var now = DateTimeOffset.Parse("2026-11-20T12:00:00Z");
            var changes = defect switch
            {
                "no-changes" => Array.Empty<StaffChangeItem>(),
                "duplicate" => new[] { Sack(41), Hire(41, 2, "senior") },
                "bad-kind" => new[] { new StaffChangeItem { StaffUid = 41, Kind = "move" } },
                "sack-with-job" => new[] { new StaffChangeItem { StaffUid = 41, Kind = "sack", JobType = 2 } },
                "hire-without-job" => new[] { new StaffChangeItem { StaffUid = 41, Kind = "hire", TargetLevel = "senior" } },
                "bad-level" => new[] { Hire(41, 2, "reserves") },
                "too-many" => Enumerable.Range(1, BridgeProtocol.MaxStaffChanges + 1).Select(uid => Sack((uint)uid)).ToArray(),
                _ => new[] { Sack(41) },
            };
            var valid = StaffChangeRequest(now, changes);
            var request = defect switch
            {
                "no-club" => new BridgeRequest
                {
                    ProtocolVersion = valid.ProtocolVersion,
                    RequestId = valid.RequestId,
                    CreatedAtUtc = now,
                    Operation = valid.Operation,
                    SourceRequestId = valid.SourceRequestId,
                    StaffChanges = valid.StaffChanges,
                },
                "squad-moves" => new BridgeRequest
                {
                    ProtocolVersion = valid.ProtocolVersion,
                    RequestId = valid.RequestId,
                    CreatedAtUtc = now,
                    Operation = valid.Operation,
                    SourceRequestId = valid.SourceRequestId,
                    ClubUid = valid.ClubUid,
                    StaffChanges = valid.StaffChanges,
                    SquadMoves = Moves((77, "reserve")),
                },
                _ => valid,
            };
            WriteBoostRequest(path, request);

            Assert.False(RequestAcceptance.TryAccept(path, now, Ttl, out _, out var reason, out _));
            Assert.Contains(expected, reason, StringComparison.Ordinal);
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    [Fact]
    public void Accept_rejects_staff_changes_on_a_squad_move_request()
    {
        var dir = CreateTempDir();
        try
        {
            var path = BridgePaths.GetRequestPath(dir);
            var now = DateTimeOffset.Parse("2026-11-20T12:00:00Z");
            var squad = SquadMoveRequest(now, Moves((77, "reserve")));
            WriteBoostRequest(
                path,
                new BridgeRequest
                {
                    ProtocolVersion = squad.ProtocolVersion,
                    RequestId = squad.RequestId,
                    CreatedAtUtc = now,
                    Operation = squad.Operation,
                    SourceRequestId = squad.SourceRequestId,
                    ClubUid = squad.ClubUid,
                    SquadMoves = squad.SquadMoves,
                    StaffChanges = new[] { Sack(41) },
                });

            Assert.False(RequestAcceptance.TryAccept(path, now, Ttl, out _, out var reason, out _));
            Assert.Contains("do not accept staff changes", reason, StringComparison.Ordinal);
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    [Fact]
    public void Accept_preserves_a_staff_limit_request()
    {
        var dir = CreateTempDir();
        try
        {
            var path = BridgePaths.GetRequestPath(dir);
            var now = DateTimeOffset.Parse("2026-11-20T12:00:00Z");
            WriteBoostRequest(path, StaffLimitRequest(now));

            Assert.True(RequestAcceptance.TryAccept(path, now, Ttl, out var request, out _, out _));
            Assert.Equal(
                (BridgeProtocol.OperationReadStaffLimits, "scan-1", (uint?)439u),
                (request.Operation, request.SourceRequestId, request.ClubUid));
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    [Theory]
    [InlineData("no-source", "sourceRequestId is required for staff limits")]
    [InlineData("no-club", "clubUid is required for staff limits")]
    [InlineData("staff-changes", "staff limits take only the club")]
    [InlineData("boost", "staff limits take only the club")]
    public void Accept_rejects_an_invalid_staff_limit_request(string defect, string expected)
    {
        var dir = CreateTempDir();
        try
        {
            var path = BridgePaths.GetRequestPath(dir);
            var now = DateTimeOffset.Parse("2026-11-20T12:00:00Z");
            var valid = StaffLimitRequest(now);
            WriteBoostRequest(
                path,
                new BridgeRequest
                {
                    ProtocolVersion = valid.ProtocolVersion,
                    RequestId = valid.RequestId,
                    CreatedAtUtc = now,
                    Operation = valid.Operation,
                    SourceRequestId = defect == "no-source" ? null : valid.SourceRequestId,
                    ClubUid = defect == "no-club" ? null : valid.ClubUid,
                    StaffChanges = defect == "staff-changes" ? new[] { Sack(41) } : null,
                    StaffUid = defect == "boost" ? 41u : null,
                });

            Assert.False(RequestAcceptance.TryAccept(path, now, Ttl, out _, out var reason, out _));
            Assert.Contains(expected, reason, StringComparison.Ordinal);
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    private static BridgeRequest StaffLimitRequest(DateTimeOffset now) =>
        new()
        {
            ProtocolVersion = BridgeProtocol.ProtocolVersion,
            RequestId = "staff-limits-1",
            CreatedAtUtc = now,
            Operation = BridgeProtocol.OperationReadStaffLimits,
            SourceRequestId = "scan-1",
            ClubUid = 439,
        };

    [Fact]
    public void Accept_preserves_a_training_request()
    {
        var dir = CreateTempDir();
        try
        {
            var path = BridgePaths.GetRequestPath(dir);
            var now = DateTimeOffset.Parse("2026-11-20T12:00:00Z");
            WriteBoostRequest(path, TrainingRequest(now, Training(77, ipPosition: 1024, ipRole: 144115188075855872, oopRole: -14, focus: 19), Training(78, focus: -1)));

            Assert.True(RequestAcceptance.TryAccept(path, now, Ttl, out var request, out _, out _));
            Assert.Equal(
                (BridgeProtocol.OperationApplyTraining, "scan-1", (uint?)439u),
                (request.Operation, request.SourceRequestId, request.ClubUid));
            Assert.Equal(
                new (uint, int?, ulong?, int?, int?)[] { (77, 1024, 144115188075855872, -14, 19), (78, null, null, null, -1) },
                request.TrainingChanges!.Select(change => (change.PlayerUid, change.IpPosition, change.IpRole, change.OopRole, change.Focus)));
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    [Theory]
    [InlineData("no-club", "clubUid is required for training changes")]
    [InlineData("duplicate", "each training change needs a distinct playerUid")]
    [InlineData("nothing", "a training change must set a role or a focus")]
    [InlineData("role-without-position", "an in-possession role needs one of FM's positions and a single role bit")]
    [InlineData("unknown-position", "an in-possession role needs one of FM's positions and a single role bit")]
    [InlineData("two-role-bits", "an in-possession role needs one of FM's positions and a single role bit")]
    [InlineData("oop-out-of-range", "oopRole must be -31 through 7")]
    [InlineData("rehab-focus", "focus must be -1 or 5 through 32")]
    [InlineData("squad-moves", "training changes take only the club and the players' training")]
    public void Accept_rejects_an_invalid_training_request(string defect, string expected)
    {
        var dir = CreateTempDir();
        try
        {
            var path = BridgePaths.GetRequestPath(dir);
            var now = DateTimeOffset.Parse("2026-11-20T12:00:00Z");
            var change = defect switch
            {
                "nothing" => Training(77),
                "role-without-position" => Training(77, ipRole: 2),
                "unknown-position" => Training(77, ipPosition: 2, ipRole: 2),
                "two-role-bits" => Training(77, ipPosition: 16, ipRole: 3),
                "oop-out-of-range" => Training(77, oopRole: 8),
                "rehab-focus" => Training(77, focus: 4),
                _ => Training(77, ipPosition: 16, ipRole: 2),
            };
            var valid = TrainingRequest(now, defect == "duplicate" ? new[] { change, change } : new[] { change });
            WriteBoostRequest(
                path,
                new BridgeRequest
                {
                    ProtocolVersion = valid.ProtocolVersion,
                    RequestId = valid.RequestId,
                    CreatedAtUtc = now,
                    Operation = valid.Operation,
                    SourceRequestId = valid.SourceRequestId,
                    ClubUid = defect == "no-club" ? null : valid.ClubUid,
                    SquadMoves = defect == "squad-moves" ? Moves((77, "youth")) : null,
                    TrainingChanges = valid.TrainingChanges,
                });

            Assert.False(RequestAcceptance.TryAccept(path, now, Ttl, out _, out var reason, out _));
            Assert.Contains(expected, reason, StringComparison.Ordinal);
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    [Fact]
    public void Accept_rejects_training_changes_on_a_squad_move()
    {
        var dir = CreateTempDir();
        try
        {
            var path = BridgePaths.GetRequestPath(dir);
            var now = DateTimeOffset.Parse("2026-11-20T12:00:00Z");
            WriteBoostRequest(
                path,
                new BridgeRequest
                {
                    ProtocolVersion = BridgeProtocol.ProtocolVersion,
                    RequestId = "move-1",
                    CreatedAtUtc = now,
                    Operation = BridgeProtocol.OperationMoveSquadPlayers,
                    SourceRequestId = "scan-1",
                    ClubUid = 439,
                    SquadMoves = Moves((77, "youth")),
                    TrainingChanges = new[] { Training(77, focus: 19) },
                });

            Assert.False(RequestAcceptance.TryAccept(path, now, Ttl, out _, out var reason, out _));
            Assert.Contains("squad moves do not accept training changes", reason, StringComparison.Ordinal);
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }

    private static BridgeRequest TrainingRequest(DateTimeOffset now, params TrainingChangeItem[] changes) =>
        new()
        {
            ProtocolVersion = BridgeProtocol.ProtocolVersion,
            RequestId = "training-1",
            CreatedAtUtc = now,
            Operation = BridgeProtocol.OperationApplyTraining,
            SourceRequestId = "scan-1",
            ClubUid = 439,
            TrainingChanges = changes,
        };

    private static TrainingChangeItem Training(
        uint uid,
        int? ipPosition = null,
        ulong? ipRole = null,
        int? oopRole = null,
        int? focus = null) =>
        new() { PlayerUid = uid, IpPosition = ipPosition, IpRole = ipRole, OopRole = oopRole, Focus = focus };

    private static StaffChangeItem Sack(uint uid) => new() { StaffUid = uid, Kind = StaffChangeItem.Sack };

    private static StaffChangeItem Hire(uint uid, int job, string level) =>
        new() { StaffUid = uid, Kind = StaffChangeItem.Hire, JobType = job, TargetLevel = level };

    private static StaffChangeItem Change(uint uid, int job, string level) =>
        new() { StaffUid = uid, Kind = StaffChangeItem.Change, JobType = job, TargetLevel = level };

    private static BridgeRequest StaffChangeRequest(DateTimeOffset now, params StaffChangeItem[] changes) =>
        new()
        {
            ProtocolVersion = BridgeProtocol.ProtocolVersion,
            RequestId = "staff-change-1",
            CreatedAtUtc = now,
            Operation = BridgeProtocol.OperationApplyStaffChanges,
            SourceRequestId = "scan-1",
            ClubUid = 439,
            StaffChanges = changes,
        };

    private static SquadMoveItem[] Moves(params (uint Uid, string Level)[] moves) =>
        moves.Select(move => new SquadMoveItem { PlayerUid = move.Uid, TargetLevel = move.Level }).ToArray();

    private static BridgeRequest SquadMoveRequest(DateTimeOffset now, SquadMoveItem[] moves) =>
        new()
        {
            ProtocolVersion = BridgeProtocol.ProtocolVersion,
            RequestId = "squad-move-1",
            CreatedAtUtc = now,
            Operation = BridgeProtocol.OperationMoveSquadPlayers,
            SourceRequestId = "scan-1",
            ClubUid = 439,
            SquadMoves = moves,
        };

    private static BridgeRequest Copy(
        BridgeRequest request,
        uint? clubUid = 439,
        SquadMoveItem[]? moves = null,
        uint? playerUid = null) =>
        new()
        {
            ProtocolVersion = request.ProtocolVersion,
            RequestId = request.RequestId,
            CreatedAtUtc = request.CreatedAtUtc,
            Operation = request.Operation,
            SourceRequestId = request.SourceRequestId,
            ClubUid = clubUid,
            SquadMoves = moves ?? request.SquadMoves,
            PlayerUid = playerUid,
        };

    private static void WriteRequest(
        string path,
        int protocolVersion,
        string requestId,
        DateTimeOffset createdAtUtc,
        string operation,
        string? playerDatabaseScopeJson = null)
    {
        Directory.CreateDirectory(Path.GetDirectoryName(path)!);
        var optionalProperties = new[]
            {
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
