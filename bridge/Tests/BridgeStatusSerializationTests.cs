using System.Text.Json;
using FmDataBridge.Memory;
using FmDataBridge.Output;
using FmDataBridge.Protocol;
using Xunit;

namespace FmDataBridge.Tests;

public sealed class BridgeStatusSerializationTests
{
    [Fact]
    public void Serialize_idle_status_includes_versioned_contract_fields()
    {
        var status = new BridgeStatus
        {
            ProtocolVersion = BridgeProtocol.ProtocolVersion,
            PluginVersion = "0.1.0",
            State = BridgeProtocol.StateIdle,
            UpdatedAtUtc = new DateTimeOffset(2026, 7, 28, 15, 0, 0, TimeSpan.Zero),
            GamePluginModulePresent = true,
            GameAssemblyModulePresent = false,
        };

        var json = StatusWriter.Serialize(status);

        using var doc = JsonDocument.Parse(json);
        var root = doc.RootElement;
        Assert.Equal(1, root.GetProperty("protocolVersion").GetInt32());
        Assert.Equal("0.1.0", root.GetProperty("pluginVersion").GetString());
        Assert.Equal("idle", root.GetProperty("state").GetString());
        Assert.Equal(
            "2026-07-28T15:00:00+00:00",
            root.GetProperty("updatedAtUtc").GetString());
        Assert.True(root.GetProperty("gamePluginModulePresent").GetBoolean());
        Assert.False(root.GetProperty("gameAssemblyModulePresent").GetBoolean());
    }

    [Fact]
    public void Write_creates_status_json_under_bridge_directory()
    {
        var bridgeDir = Path.Combine(Path.GetTempPath(), "fm-valuescout-tests", Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(bridgeDir);

        try
        {
            var status = new BridgeStatus
            {
                ProtocolVersion = BridgeProtocol.ProtocolVersion,
                PluginVersion = "0.1.0",
                State = BridgeProtocol.StateIdle,
                UpdatedAtUtc = DateTimeOffset.UtcNow,
                GamePluginModulePresent = false,
                GameAssemblyModulePresent = true,
            };

            StatusWriter.Write(bridgeDir, status);

            var path = BridgePaths.GetStatusPath(bridgeDir);
            Assert.True(File.Exists(path));
            using var doc = JsonDocument.Parse(File.ReadAllText(path));
            Assert.Equal("idle", doc.RootElement.GetProperty("state").GetString());
            Assert.True(doc.RootElement.GetProperty("gameAssemblyModulePresent").GetBoolean());
        }
        finally
        {
            if (Directory.Exists(bridgeDir))
            {
                Directory.Delete(bridgeDir, recursive: true);
            }
        }
    }

    [Fact]
    public void Bridge_directory_is_localappdata_fm_valuescout_fm_bridge()
    {
        var expected = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
            "fm-valuescout",
            "fm-bridge");

        Assert.Equal(expected, BridgePaths.GetBridgeDirectory());
    }

    [Fact]
    public void Module_presence_detects_game_plugin_and_game_assembly_by_name()
    {
        var signals = ModulePresence.DetectFromModuleNames(
            new[] { "fm.exe", "game_plugin.dll", "kernel32.dll" });

        Assert.True(signals.GamePluginModulePresent);
        Assert.False(signals.GameAssemblyModulePresent);
    }

    [Fact]
    public void Module_presence_from_bounds_matches_locator_results()
    {
        var both = new ModulePresenceBounds(
            new ModuleBounds("game_plugin.dll", 0x1000, 0x2000),
            new ModuleBounds("GameAssembly.dll", 0x3000, 0x4000));
        var signals = ModulePresence.FromBounds(both);
        Assert.True(signals.GamePluginModulePresent);
        Assert.True(signals.GameAssemblyModulePresent);

        var assemblyOnly = ModulePresence.FromBounds(
            new ModulePresenceBounds(null, new ModuleBounds("GameAssembly.dll", 0x3000, 0x4000)));
        Assert.False(assemblyOnly.GamePluginModulePresent);
        Assert.True(assemblyOnly.GameAssemblyModulePresent);
    }

    [Fact]
    public void Status_writer_round_trips_module_flags()
    {
        var dir = Path.Combine(Path.GetTempPath(), "fm-valuescout-tests", Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(dir);
        try
        {
            StatusWriter.Write(
                dir,
                new BridgeStatus
                {
                    ProtocolVersion = 1,
                    PluginVersion = "0.1.0",
                    State = "ready",
                    UpdatedAtUtc = DateTimeOffset.Parse("2026-07-28T19:00:00Z"),
                    GamePluginModulePresent = true,
                    GameAssemblyModulePresent = true,
                    RequestId = "req-1",
                    PlayersFound = 10,
                });

            Assert.True(StatusWriter.TryRead(dir, out var status));
            Assert.NotNull(status);
            Assert.True(status!.GamePluginModulePresent);
            Assert.True(status.GameAssemblyModulePresent);
            Assert.Equal("ready", status.State);
            Assert.Equal("req-1", status.RequestId);
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }
}
