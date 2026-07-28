using BepInEx;
using BepInEx.Logging;
using BepInEx.Unity.IL2CPP;
using FmDataBridge.Memory;
using FmDataBridge.Output;
using FmDataBridge.Protocol;
using FmDataBridge.Scanning;

namespace FmDataBridge;

[BepInPlugin(MyPluginInfo.PLUGIN_GUID, MyPluginInfo.PLUGIN_NAME, MyPluginInfo.PLUGIN_VERSION)]
public class Plugin : BasePlugin
{
    internal static new ManualLogSource Log = null!;

    private CancellationTokenSource? _forceScanCts;

    private static readonly TimeSpan ForceScanPollInterval = TimeSpan.FromSeconds(2);

    public override void Load()
    {
        Log = base.Log;

        var modules = DetectModulesBestEffort();

        try
        {
            var bridgeDirectory = BridgePaths.EnsureBridgeDirectory();
            WriteIdleStatus(bridgeDirectory, modules);
            Log.LogInfo(
                $"FM Data Bridge {MyPluginInfo.PLUGIN_VERSION} loaded; wrote {BridgePaths.GetStatusPath(bridgeDirectory)}");

            // Temporary manual trigger until Commit 3 request polling lands.
            // Poll so a save can be loaded first, then drop an empty force-scan file.
            _forceScanCts = new CancellationTokenSource();
            var token = _forceScanCts.Token;
            var thread = new Thread(() => PollForceScan(bridgeDirectory, modules, token))
            {
                IsBackground = true,
                Name = "FmBridge-ForceScanPoll",
            };
            thread.Start();
        }
        catch (Exception ex)
        {
            Log.LogError($"Failed to write bridge status.json: {ex}");
        }
    }

    public override bool Unload()
    {
        try
        {
            _forceScanCts?.Cancel();
        }
        catch (Exception ex)
        {
            Log.LogWarning($"Force-scan poll cancel failed: {ex.Message}");
        }

        return true;
    }

    private static void PollForceScan(
        string bridgeDirectory,
        ModulePresenceSignals modules,
        CancellationToken token)
    {
        while (!token.IsCancellationRequested)
        {
            try
            {
                if (File.Exists(BridgePaths.GetForceScanPath(bridgeDirectory)))
                {
                    RunForcedScan(bridgeDirectory, modules);
                }
            }
            catch (Exception ex)
            {
                Log.LogError($"Force-scan poll iteration failed: {ex}");
            }

            token.WaitHandle.WaitOne(ForceScanPollInterval);
        }
    }

    private static void RunForcedScan(string bridgeDirectory, ModulePresenceSignals modules)
    {
        try
        {
            WriteStatus(
                bridgeDirectory,
                BridgeProtocol.StateScanning,
                modules.GamePluginModulePresent,
                modules.GameAssemblyModulePresent);

            if (!GameVersionDetector.TryDetectFromCurrentProcess(out var gameVersion)
                || string.IsNullOrWhiteSpace(gameVersion))
            {
                // Fallback when game_plugin.dll version is missing — still exercise 26.3 layout path.
                gameVersion = Fm263FallbackVersion;
                Log.LogWarning(
                    $"Could not read game_plugin.dll version; using fallback '{gameVersion}' for layout resolve");
            }

            var reader = new WindowsMemoryReader();
            var known = reader.LocateKnownModules();
            if (known.GameAssembly is not { } gameAssembly)
            {
                var message = "GameAssembly.dll bounds not found; cannot scan";
                DiagnosticsWriter.Write(bridgeDirectory, message + Environment.NewLine);
                WriteStatus(
                    bridgeDirectory,
                    BridgeProtocol.StateFailed,
                    modules.GamePluginModulePresent,
                    modules.GameAssemblyModulePresent);
                Log.LogError(message);
                TryDeleteForceScanFlag(bridgeDirectory);
                return;
            }

            var result = new CapADumpPipeline().Run(
                reader,
                bridgeDirectory,
                gameVersion,
                MyPluginInfo.PLUGIN_VERSION,
                gameAssembly,
                known.GamePlugin);

            TryDeleteForceScanFlag(bridgeDirectory);

            WriteStatus(
                bridgeDirectory,
                result.Success ? BridgeProtocol.StateReady : BridgeProtocol.StateFailed,
                modules.GamePluginModulePresent,
                modules.GameAssemblyModulePresent);

            if (result.Success)
            {
                Log.LogInfo($"Force scan wrote dump with {result.PlayerCount} players");
            }
            else
            {
                Log.LogWarning($"Force scan failed: {result.Error}");
            }
        }
        catch (Exception ex)
        {
            Log.LogError($"Force scan crashed: {ex}");
            TryDeleteForceScanFlag(bridgeDirectory);
            try
            {
                WriteStatus(
                    bridgeDirectory,
                    BridgeProtocol.StateFailed,
                    modules.GamePluginModulePresent,
                    modules.GameAssemblyModulePresent);
            }
            catch
            {
                // ignored — status best-effort
            }
        }
    }

    private const string Fm263FallbackVersion = "26.3.0";

    private static void TryDeleteForceScanFlag(string bridgeDirectory)
    {
        try
        {
            var path = BridgePaths.GetForceScanPath(bridgeDirectory);
            if (File.Exists(path))
            {
                File.Delete(path);
            }
        }
        catch (Exception ex)
        {
            Log.LogWarning($"Could not remove force-scan flag: {ex.Message}");
        }
    }

    private static void WriteIdleStatus(string bridgeDirectory, ModulePresenceSignals modules) =>
        WriteStatus(
            bridgeDirectory,
            BridgeProtocol.StateIdle,
            modules.GamePluginModulePresent,
            modules.GameAssemblyModulePresent);

    private static void WriteStatus(
        string bridgeDirectory,
        string state,
        bool gamePluginModulePresent,
        bool gameAssemblyModulePresent)
    {
        var status = new BridgeStatus
        {
            ProtocolVersion = BridgeProtocol.ProtocolVersion,
            PluginVersion = MyPluginInfo.PLUGIN_VERSION,
            State = state,
            UpdatedAtUtc = DateTimeOffset.UtcNow,
            GamePluginModulePresent = gamePluginModulePresent,
            GameAssemblyModulePresent = gameAssemblyModulePresent,
        };
        StatusWriter.Write(bridgeDirectory, status);
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
