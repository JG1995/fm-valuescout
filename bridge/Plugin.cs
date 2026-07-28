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

    private CancellationTokenSource? _pollCts;

    private static readonly TimeSpan RequestPollInterval = TimeSpan.FromSeconds(2);

    private static readonly TimeSpan RequestTtl = TimeSpan.FromSeconds(BridgeProtocol.RequestTtlSeconds);

    private static readonly object ScanGate = new();

    private static bool _scanInProgress;

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

            _pollCts = new CancellationTokenSource();
            var token = _pollCts.Token;
            var thread = new Thread(() => PollRequests(bridgeDirectory, modules, token))
            {
                IsBackground = true,
                Name = "FmBridge-RequestPoll",
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
            _pollCts?.Cancel();
        }
        catch (Exception ex)
        {
            Log.LogWarning($"Request poll cancel failed: {ex.Message}");
        }

        return true;
    }

    private static void PollRequests(
        string bridgeDirectory,
        ModulePresenceSignals modules,
        CancellationToken token)
    {
        while (!token.IsCancellationRequested)
        {
            try
            {
                TryStartScanFromRequestOrForceFlag(bridgeDirectory, modules);
            }
            catch (Exception ex)
            {
                Log.LogError($"Request poll iteration failed: {ex}");
            }

            token.WaitHandle.WaitOne(RequestPollInterval);
        }
    }

    private static void TryStartScanFromRequestOrForceFlag(
        string bridgeDirectory,
        ModulePresenceSignals modules)
    {
        string? requestId = null;

        lock (ScanGate)
        {
            if (_scanInProgress)
            {
                // Keep a waiting request.json fresh so a long scan does not TTL-kill it.
                RequestAcceptance.TryRefreshCreatedAtUtc(
                    BridgePaths.GetRequestPath(bridgeDirectory),
                    DateTimeOffset.UtcNow);
                return;
            }

            var requestPath = BridgePaths.GetRequestPath(bridgeDirectory);
            if (File.Exists(requestPath))
            {
                if (!RequestAcceptance.TryAccept(
                        requestPath,
                        DateTimeOffset.UtcNow,
                        RequestTtl,
                        out var request,
                        out var rejectReason,
                        out var observedRequestId))
                {
                    Log.LogWarning($"Ignored bridge request: {rejectReason}");
                    if (!string.IsNullOrEmpty(observedRequestId))
                    {
                        WriteStatus(
                            bridgeDirectory,
                            BridgeProtocol.StateFailed,
                            modules,
                            requestId: observedRequestId,
                            playersFound: null,
                            error: rejectReason);
                    }

                    return;
                }

                requestId = request.RequestId;
            }
            else if (File.Exists(BridgePaths.GetForceScanPath(bridgeDirectory)))
            {
                // Manual fallback until operators prefer only the in-app request path.
                requestId = "force-scan";
            }
            else
            {
                return;
            }

            _scanInProgress = true;
        }

        try
        {
            RunDumpScan(bridgeDirectory, modules, requestId!);
        }
        finally
        {
            lock (ScanGate)
            {
                _scanInProgress = false;
            }
        }
    }

    private static void RunDumpScan(
        string bridgeDirectory,
        ModulePresenceSignals modules,
        string requestId)
    {
        try
        {
            WriteStatus(
                bridgeDirectory,
                BridgeProtocol.StateScanning,
                modules,
                requestId: requestId,
                playersFound: null,
                error: null);

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
                    modules,
                    requestId: requestId,
                    playersFound: null,
                    error: message);
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

            if (result.Success)
            {
                WriteStatus(
                    bridgeDirectory,
                    BridgeProtocol.StateReady,
                    modules,
                    requestId: requestId,
                    playersFound: result.PlayerCount,
                    error: null);
                Log.LogInfo($"Dump request {requestId} wrote {result.PlayerCount} players");
            }
            else
            {
                WriteStatus(
                    bridgeDirectory,
                    BridgeProtocol.StateFailed,
                    modules,
                    requestId: requestId,
                    playersFound: null,
                    error: result.Error);
                Log.LogWarning($"Dump request {requestId} failed: {result.Error}");
            }
        }
        catch (Exception ex)
        {
            Log.LogError($"Dump scan crashed: {ex}");
            TryDeleteForceScanFlag(bridgeDirectory);
            try
            {
                WriteStatus(
                    bridgeDirectory,
                    BridgeProtocol.StateFailed,
                    modules,
                    requestId: requestId,
                    playersFound: null,
                    error: ex.Message);
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
            modules,
            requestId: null,
            playersFound: null,
            error: null);

    private static void WriteStatus(
        string bridgeDirectory,
        string state,
        ModulePresenceSignals modules,
        string? requestId,
        int? playersFound,
        string? error)
    {
        var status = new BridgeStatus
        {
            ProtocolVersion = BridgeProtocol.ProtocolVersion,
            PluginVersion = MyPluginInfo.PLUGIN_VERSION,
            State = state,
            UpdatedAtUtc = DateTimeOffset.UtcNow,
            GamePluginModulePresent = modules.GamePluginModulePresent,
            GameAssemblyModulePresent = modules.GameAssemblyModulePresent,
            RequestId = requestId,
            PlayersFound = playersFound,
            Error = error,
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
