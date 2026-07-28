using BepInEx;
using BepInEx.Logging;
using BepInEx.Unity.IL2CPP;
using FmDataBridge.Output;
using FmDataBridge.Protocol;

namespace FmDataBridge;

[BepInPlugin(MyPluginInfo.PLUGIN_GUID, MyPluginInfo.PLUGIN_NAME, MyPluginInfo.PLUGIN_VERSION)]
public class Plugin : BasePlugin
{
    internal static new ManualLogSource Log = null!;

    public override void Load()
    {
        Log = base.Log;

        var modules = DetectModulesBestEffort();

        try
        {
            var bridgeDirectory = BridgePaths.EnsureBridgeDirectory();
            var status = new BridgeStatus
            {
                ProtocolVersion = BridgeProtocol.ProtocolVersion,
                PluginVersion = MyPluginInfo.PLUGIN_VERSION,
                State = BridgeProtocol.StateIdle,
                UpdatedAtUtc = DateTimeOffset.UtcNow,
                GamePluginModulePresent = modules.GamePluginModulePresent,
                GameAssemblyModulePresent = modules.GameAssemblyModulePresent,
            };
            StatusWriter.Write(bridgeDirectory, status);
            Log.LogInfo(
                $"FM Data Bridge {MyPluginInfo.PLUGIN_VERSION} loaded; wrote {BridgePaths.GetStatusPath(bridgeDirectory)}");
        }
        catch (Exception ex)
        {
            Log.LogError($"Failed to write bridge status.json: {ex}");
        }
    }

    private static ModulePresenceSignals DetectModulesBestEffort()
    {
        try
        {
            return ModulePresence.Detect();
        }
        catch (Exception ex)
        {
            Log.LogWarning(
                $"Could not enumerate process modules; writing status with modules absent: {ex.Message}");
            return new ModulePresenceSignals(false, false);
        }
    }
}
