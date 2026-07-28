using System.Text.Json;
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
}
