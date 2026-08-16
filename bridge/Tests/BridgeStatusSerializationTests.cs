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
    public void Serialize_player_boost_status_exposes_verified_values_without_identity_or_memory_details()
    {
        var json = StatusWriter.Serialize(
            new BridgeStatus
            {
                ProtocolVersion = BridgeProtocol.ProtocolVersion,
                PluginVersion = "0.1.0",
                State = BridgeProtocol.StateReady,
                UpdatedAtUtc = new DateTimeOffset(2026, 8, 9, 12, 0, 0, TimeSpan.Zero),
                GamePluginModulePresent = true,
                GameAssemblyModulePresent = true,
                RequestId = "boost-ca-1",
                PlayerBoostsSupported = true,
                PlayerBoost = new PlayerBoostResult
                {
                    Operation = BridgeProtocol.OperationBoostCurrentAbility,
                    Outcome = "verified",
                    Rollback = "not-needed",
                    PreviousCurrentAbility = 120,
                    CurrentAbility = 125,
                    PotentialAbility = 150,
                },
            });

        using var doc = JsonDocument.Parse(json);
        var root = doc.RootElement;
        Assert.True(root.GetProperty("playerBoostsSupported").GetBoolean());
        var result = root.GetProperty("playerBoost");
        Assert.Equal(BridgeProtocol.OperationBoostCurrentAbility, result.GetProperty("operation").GetString());
        Assert.Equal("verified", result.GetProperty("outcome").GetString());
        Assert.Equal(125, result.GetProperty("currentAbility").GetInt32());
        Assert.DoesNotContain("address", json, StringComparison.OrdinalIgnoreCase);
        Assert.DoesNotContain("uid", json, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void Serialize_staff_boost_status_exposes_only_sanitized_verified_values()
    {
        var json = StatusWriter.Serialize(
            new BridgeStatus
            {
                ProtocolVersion = BridgeProtocol.ProtocolVersion,
                PluginVersion = "0.1.0",
                State = BridgeProtocol.StateReady,
                UpdatedAtUtc = new DateTimeOffset(2026, 8, 16, 12, 0, 0, TimeSpan.Zero),
                GamePluginModulePresent = true,
                GameAssemblyModulePresent = true,
                RequestId = "staff-boost-1",
                StaffBoostsSupported = true,
                StaffBoost = new StaffBoostResult
                {
                    Operation = BridgeProtocol.OperationBoostStaffCurrentAbility,
                    Outcome = "verified",
                    Rollback = "not-needed",
                    PreviousCurrentAbility = 120,
                    CurrentAbility = 130,
                    PotentialAbility = 150,
                },
            });

        using var doc = JsonDocument.Parse(json);
        var root = doc.RootElement;
        Assert.True(root.GetProperty("staffBoostsSupported").GetBoolean());
        var result = root.GetProperty("staffBoost");
        Assert.Equal(BridgeProtocol.OperationBoostStaffCurrentAbility, result.GetProperty("operation").GetString());
        Assert.Equal(130, result.GetProperty("currentAbility").GetInt32());
        Assert.DoesNotContain("address", json, StringComparison.OrdinalIgnoreCase);
        Assert.DoesNotContain("uid", json, StringComparison.OrdinalIgnoreCase);
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
    public void Write_replaces_machine_local_paths_in_failed_status_errors()
    {
        var bridgeDir = Path.Combine(Path.GetTempPath(), "fm-valuescout-tests", Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(bridgeDir);

        try
        {
            StatusWriter.Write(
                bridgeDir,
                new BridgeStatus
                {
                    ProtocolVersion = BridgeProtocol.ProtocolVersion,
                    PluginVersion = "0.1.0",
                    State = BridgeProtocol.StateFailed,
                    UpdatedAtUtc = DateTimeOffset.UtcNow,
                    GamePluginModulePresent = true,
                    GameAssemblyModulePresent = true,
                    StaffBoostsSupported = true,
                    StaffBoost = new StaffBoostResult
                    {
                        Operation = BridgeProtocol.OperationBoostStaffCurrentAbility,
                        Outcome = "failed",
                        Rollback = "restored",
                        PreviousCurrentAbility = 120,
                        PotentialAbility = 150,
                    },
                    Error = "Could not create C:\\Users\\player\\AppData\\Local\\fm-valuescout\\fm-bridge\\dump.json",
                });

            var serialized = File.ReadAllText(BridgePaths.GetStatusPath(bridgeDir));
            Assert.DoesNotContain("C:\\Users\\player", serialized, StringComparison.Ordinal);
            Assert.True(StatusWriter.TryRead(bridgeDir, out var status));
            Assert.Equal("scan failed unexpectedly", status!.Error);
            Assert.True(status.StaffBoostsSupported);
            Assert.Equal("restored", status.StaffBoost!.Rollback);
        }
        finally
        {
            Directory.Delete(bridgeDir, recursive: true);
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
                    ScanTruncated = true,
                    MaxAccepted = 10_000,
                });

            Assert.True(StatusWriter.TryRead(dir, out var status));
            Assert.NotNull(status);
            Assert.True(status!.GamePluginModulePresent);
            Assert.True(status.GameAssemblyModulePresent);
            Assert.Equal("ready", status.State);
            Assert.Equal("req-1", status.RequestId);
            Assert.True(status.ScanTruncated);
            Assert.Equal(10_000, status.MaxAccepted);
        }
        finally
        {
            Directory.Delete(dir, recursive: true);
        }
    }
}
