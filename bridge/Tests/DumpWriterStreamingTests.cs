using System.Text.Json;
using FmDataBridge.Models;
using FmDataBridge.Output;
using FmDataBridge.Protocol;
using Xunit;

namespace FmDataBridge.Tests;

public sealed class DumpWriterStreamingTests
{
    [Fact]
    public void Dump_writer_emits_compact_schema_v6_json()
    {
        var document = MinimalDocument(playerCount: 2);
        var json = DumpWriter.Serialize(document);

        Assert.DoesNotContain("\n  ", json, StringComparison.Ordinal);
        Assert.DoesNotContain("\r\n", json, StringComparison.Ordinal);

        using var parsed = JsonDocument.Parse(json);
        var root = parsed.RootElement;
        Assert.Equal(6, root.GetProperty("schemaVersion").GetInt32());
        Assert.Equal("2026-07-30T00:00:00+00:00", root.GetProperty("generatedAtUtc").GetString());
        Assert.Equal("26.3.2", root.GetProperty("gameVersion").GetString());
        Assert.Equal("26.3", root.GetProperty("supportedGameVersion").GetString());
        Assert.Equal("0.1.0", root.GetProperty("bridgeVersion").GetString());
        Assert.Equal(BridgeProtocol.ProtocolVersion, root.GetProperty("protocolVersion").GetInt32());
        Assert.Equal(JsonValueKind.Null, root.GetProperty("gameDate").ValueKind);
        Assert.Equal("unknown", root.GetProperty("gameDateSource").GetString());
        Assert.Equal("unknown", root.GetProperty("gameDateBasis").GetString());
        Assert.Equal("men", root.GetProperty("playerDatabaseScope").GetString());
        Assert.False(root.GetProperty("scanTruncated").GetBoolean());
        Assert.Equal(JsonValueKind.Null, root.GetProperty("maxAccepted").ValueKind);
        Assert.Equal(2, root.GetProperty("playerCount").GetInt32());
        Assert.Equal(2, root.GetProperty("players").GetArrayLength());
        Assert.Equal(0, root.GetProperty("staffCount").GetInt32());
        Assert.Empty(root.GetProperty("staff").EnumerateArray());
        Assert.Equal(JsonValueKind.Null, root.GetProperty("manager").ValueKind);
        Assert.Equal(1u, root.GetProperty("players")[0].GetProperty("uid").GetUInt32());
        Assert.Equal(2u, root.GetProperty("players")[1].GetProperty("uid").GetUInt32());
    }

    [Fact]
    public void Dump_writer_replace_writes_compact_json_to_disk()
    {
        var bridgeDir = Path.Combine(Path.GetTempPath(), "fm-dump-stream-" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(bridgeDir);
        try
        {
            var document = MinimalDocument(playerCount: 3);
            Assert.True(DumpWriter.TryWriteReplaceOnSuccess(bridgeDir, document));

            var json = File.ReadAllText(BridgePaths.GetDumpPath(bridgeDir));
            Assert.DoesNotContain("\n  ", json, StringComparison.Ordinal);

            using var parsed = JsonDocument.Parse(json);
            Assert.Equal(3, parsed.RootElement.GetProperty("playerCount").GetInt32());
            Assert.Equal(3, parsed.RootElement.GetProperty("players").GetArrayLength());
        }
        finally
        {
            Directory.Delete(bridgeDir, recursive: true);
        }
    }

    [Fact]
    public void Dump_writer_cancellation_during_streaming_preserves_prior_dump()
    {
        var bridgeDir = Path.Combine(Path.GetTempPath(), "fm-dump-stream-" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(bridgeDir);
        try
        {
            Assert.True(DumpWriter.TryWriteReplaceOnSuccess(bridgeDir, MinimalDocument(playerCount: 1)));
            using var cancellation = new CancellationTokenSource();
            var pending = MinimalDocument(playerCount: 2);
            var document = new DumpDocument
            {
                SchemaVersion = pending.SchemaVersion,
                GeneratedAtUtc = pending.GeneratedAtUtc,
                GameVersion = pending.GameVersion,
                SupportedGameVersion = pending.SupportedGameVersion,
                BridgeVersion = pending.BridgeVersion,
                ProtocolVersion = pending.ProtocolVersion,
                GameDateSource = pending.GameDateSource,
                GameDateBasis = pending.GameDateBasis,
                PlayerDatabaseScope = pending.PlayerDatabaseScope,
                PlayerCount = pending.PlayerCount,
                Players = new CancelOnEnumerationPlayers(pending.Players, cancellation),
            };

            Assert.Throws<OperationCanceledException>(() =>
                DumpWriter.TryWriteReplaceOnSuccess(bridgeDir, document, cancellation.Token));

            var path = BridgePaths.GetDumpPath(bridgeDir);
            Assert.False(File.Exists(path + ".tmp"));
            using var parsed = JsonDocument.Parse(File.ReadAllText(path));
            Assert.Equal(1, parsed.RootElement.GetProperty("playerCount").GetInt32());
        }
        finally
        {
            Directory.Delete(bridgeDir, recursive: true);
        }
    }

    [Fact]
    public void Dump_writer_streams_players_with_bounded_write_chunks()
    {
        var document = MinimalDocument(playerCount: 5_000);
        using var tracking = new MaxWriteTrackingStream();
        DumpWriter.WriteCompact(tracking, document);
        tracking.Flush();

        // One-shot Serialize-to-string then a single Write is one huge chunk and one call.
        // Per-player Utf8JsonWriter.Flush produces many small writes.
        Assert.True(
            tracking.WriteCallCount >= 5_000,
            $"expected at least one flush write per player, got WriteCallCount={tracking.WriteCallCount}");
        Assert.True(
            tracking.MaxWriteBytes < 256 * 1024,
            $"expected streamed writes under 256 KiB, got max write {tracking.MaxWriteBytes} bytes");
        Assert.Equal(5_000, CountPlayers(tracking.ToArray()));
    }

    [Theory]
    [InlineData(184_000)]
    [InlineData(500_000)]
    public void Dump_writer_replace_completes_for_large_player_counts(int playerCount)
    {
        var bridgeDir = Path.Combine(Path.GetTempPath(), "fm-dump-replace-" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(bridgeDir);
        try
        {
            var document = MinimalDocument(playerCount);
            Assert.True(DumpWriter.TryWriteReplaceOnSuccess(bridgeDir, document));

            var path = BridgePaths.GetDumpPath(bridgeDir);
            Assert.True(File.Exists(path));
            Assert.Equal(playerCount, CountPlayersFromFile(path));
        }
        finally
        {
            Directory.Delete(bridgeDir, recursive: true);
        }
    }

    private static DumpDocument MinimalDocument(int playerCount)
    {
        var players = new DumpPlayer[playerCount];
        for (var i = 0; i < playerCount; i++)
        {
            players[i] = new DumpPlayer
            {
                Uid = (uint)(i + 1),
                Ca = 10,
                Pa = 20,
                Name = "P",
                BirthYear = 2000,
                BirthDayOfYear = 1,
                PreferredFoot = "right",
            };
        }

        return new DumpDocument
        {
            SchemaVersion = BridgeProtocol.DumpSchemaVersion,
            GeneratedAtUtc = "2026-07-30T00:00:00+00:00",
            GameVersion = "26.3.2",
            SupportedGameVersion = "26.3",
            BridgeVersion = "0.1.0",
            ProtocolVersion = BridgeProtocol.ProtocolVersion,
            GameDateSource = "unknown",
            PlayerCount = playerCount,
            Players = players,
        };
    }

    private static int CountPlayersFromFile(string path)
    {
        using var stream = File.OpenRead(path);
        using var ms = new MemoryStream();
        stream.CopyTo(ms);
        return CountPlayers(ms.ToArray());
    }

    private static int CountPlayers(byte[] utf8)
    {
        var count = 0;
        var reader = new Utf8JsonReader(utf8);
        var inPlayers = false;
        while (reader.Read())
        {
            if (reader.TokenType == JsonTokenType.PropertyName
                && reader.ValueTextEquals("players"))
            {
                inPlayers = true;
                continue;
            }

            if (!inPlayers)
            {
                continue;
            }

            if (reader.TokenType == JsonTokenType.StartObject)
            {
                count++;
                reader.Skip();
                continue;
            }

            if (reader.TokenType == JsonTokenType.EndArray)
            {
                break;
            }
        }

        return count;
    }

    private sealed class MaxWriteTrackingStream : MemoryStream
    {
        public int MaxWriteBytes { get; private set; }

        public int WriteCallCount { get; private set; }

        public override void Write(byte[] buffer, int offset, int count)
        {
            WriteCallCount++;
            if (count > MaxWriteBytes)
            {
                MaxWriteBytes = count;
            }

            base.Write(buffer, offset, count);
        }

        public override void Write(ReadOnlySpan<byte> buffer)
        {
            WriteCallCount++;
            if (buffer.Length > MaxWriteBytes)
            {
                MaxWriteBytes = buffer.Length;
            }

            base.Write(buffer);
        }
    }

    private sealed class CancelOnEnumerationPlayers : IReadOnlyList<DumpPlayer>
    {
        private readonly IReadOnlyList<DumpPlayer> _players;
        private readonly CancellationTokenSource _cancellation;

        public CancelOnEnumerationPlayers(
            IReadOnlyList<DumpPlayer> players,
            CancellationTokenSource cancellation)
        {
            _players = players;
            _cancellation = cancellation;
        }

        public int Count => _players.Count;

        public DumpPlayer this[int index] => _players[index];

        public IEnumerator<DumpPlayer> GetEnumerator()
        {
            foreach (var player in _players)
            {
                _cancellation.Cancel();
                yield return player;
            }
        }

        System.Collections.IEnumerator System.Collections.IEnumerable.GetEnumerator() => GetEnumerator();
    }
}
