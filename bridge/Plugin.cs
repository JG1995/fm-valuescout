using BepInEx;
using BepInEx.Logging;
using BepInEx.Unity.IL2CPP;
using FmDataBridge.Memory;
using FmDataBridge.Output;
using FmDataBridge.Protocol;
using FmDataBridge.Research;
using FmDataBridge.Scanning;

namespace FmDataBridge;

[BepInPlugin(MyPluginInfo.PLUGIN_GUID, MyPluginInfo.PLUGIN_NAME, MyPluginInfo.PLUGIN_VERSION)]
public class Plugin : BasePlugin
{
    internal static new ManualLogSource Log = null!;

    private CancellationTokenSource? _pollCts;

    private Thread? _pollThread;

    private static CancellationTokenSource? s_unloadCts;

    private static Thread? s_scanThread;

    private static readonly TimeSpan RequestPollInterval = TimeSpan.FromSeconds(2);

    private static readonly TimeSpan RequestTtl = TimeSpan.FromSeconds(BridgeProtocol.RequestTtlSeconds);

    private static readonly TimeSpan PollThreadJoinTimeout = TimeSpan.FromSeconds(30);

    private static readonly TimeSpan ScanThreadJoinTimeout = TimeSpan.FromSeconds(30);

    private static readonly object ScanGate = new();

    private static bool _scanInProgress;

    public override void Load()
    {
        Log = base.Log;

        try
        {
            var bridgeDirectory = BridgePaths.EnsureBridgeDirectory();
            WriteIdleStatus(bridgeDirectory, DetectModulesBestEffort());
            Log.LogInfo(
                $"FM Data Bridge {MyPluginInfo.PLUGIN_VERSION} loaded; wrote {BridgePaths.GetStatusPath(bridgeDirectory)}");

            s_unloadCts = new CancellationTokenSource();
            _pollCts = new CancellationTokenSource();
            var token = _pollCts.Token;
            _pollThread = new Thread(() => PollRequests(bridgeDirectory, token))
            {
                IsBackground = true,
                Name = "FmBridge-RequestPoll",
            };
            _pollThread.Start();
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
            s_unloadCts?.Cancel();
            _pollCts?.Cancel();
        }
        catch (Exception ex)
        {
            Log.LogWarning($"Bridge shutdown cancel failed: {ex.Message}");
        }

        if (_pollThread is { IsAlive: true } pollThread
            && !pollThread.Join(PollThreadJoinTimeout))
        {
            Log.LogWarning(
                $"Request poll thread did not exit within {PollThreadJoinTimeout.TotalSeconds:0}s");
        }

        if (s_scanThread is { IsAlive: true } scanThread
            && !scanThread.Join(ScanThreadJoinTimeout))
        {
            Log.LogWarning(
                $"Scan thread did not exit within {ScanThreadJoinTimeout.TotalSeconds:0}s");
        }

        return true;
    }

    private static void PollRequests(string bridgeDirectory, CancellationToken token)
    {
        while (!token.IsCancellationRequested)
        {
            try
            {
                TryStartScanFromRequestOrForceFlag(bridgeDirectory);
                TryRefreshStaleModuleFlags(bridgeDirectory);
            }
            catch (Exception ex)
            {
                Log.LogError($"Request poll iteration failed: {ex}");
            }

            token.WaitHandle.WaitOne(RequestPollInterval);
        }
    }

    private static void TryStartScanFromRequestOrForceFlag(string bridgeDirectory)
    {
        var scanKind = ScanRequestKind.None;
        string? requestId = null;
        int? maxAccepted = null;
        ProbeRequest? probeRequest = null;

        lock (ScanGate)
        {
            if (_scanInProgress)
            {
                RefreshQueuedRequests(bridgeDirectory);
                return;
            }

            scanKind = ScanRequestPriority.Select(
                File.Exists(BridgePaths.GetRequestPath(bridgeDirectory)),
                File.Exists(BridgePaths.GetForceScanPath(bridgeDirectory)),
                File.Exists(BridgePaths.GetProbeRequestPath(bridgeDirectory)));
            switch (scanKind)
            {
                case ScanRequestKind.FullDump:
                    if (!RequestAcceptance.TryAccept(
                            BridgePaths.GetRequestPath(bridgeDirectory),
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
                                DetectModulesBestEffort(),
                                requestId: observedRequestId,
                                playersFound: null,
                                error: rejectReason);
                        }

                        return;
                    }

                    requestId = request.RequestId;
                    maxAccepted = request.MaxAccepted;
                    break;

                case ScanRequestKind.ForceDump:
                    // Manual fallback until operators prefer only the in-app request path.
                    // Unlimited — same as production Load Data (null maxAccepted).
                    requestId = "force-scan";
                    maxAccepted = null;
                    break;

                case ScanRequestKind.Probe:
                    if (!ProbeRequestAcceptance.TryAccept(
                            BridgePaths.GetProbeRequestPath(bridgeDirectory),
                            DateTimeOffset.UtcNow,
                            RequestTtl,
                            out probeRequest,
                            out var probeRejectReason,
                            out var observedProbeRequestId))
                    {
                        Log.LogWarning($"Ignored probe request: {probeRejectReason}");
                        WriteProbeStatus(
                            bridgeDirectory,
                            ProbeProtocol.StateFailed,
                            DetectModulesBestEffort(),
                            requestId: observedProbeRequestId,
                            playersCaptured: null,
                            error: probeRejectReason);

                        return;
                    }

                    requestId = probeRequest.RequestId;
                    break;

                default:
                    return;
            }

            _scanInProgress = true;
        }

        var scanRequestId = requestId!;
        var scanMaxAccepted = maxAccepted;
        var acceptedProbeRequest = probeRequest;
        var cancelToken = s_unloadCts?.Token ?? CancellationToken.None;
        s_scanThread = new Thread(() =>
        {
            try
            {
                if (scanKind == ScanRequestKind.Probe)
                {
                    RunProbeScan(bridgeDirectory, acceptedProbeRequest!, cancelToken);
                }
                else
                {
                    RunDumpScan(bridgeDirectory, scanRequestId, scanMaxAccepted, cancelToken);
                }
            }
            finally
            {
                lock (ScanGate)
                {
                    _scanInProgress = false;
                }
            }
        })
        {
            IsBackground = true,
            Name = "FmBridge-Scan",
        };
        s_scanThread.Start();
    }

    private static void RefreshQueuedRequests(string bridgeDirectory)
    {
        var now = DateTimeOffset.UtcNow;
        // A full-dump request still wins when both queues are waiting.
        RequestAcceptance.TryRefreshCreatedAtUtc(BridgePaths.GetRequestPath(bridgeDirectory), now);
        ProbeRequestAcceptance.TryRefreshCreatedAtUtc(BridgePaths.GetProbeRequestPath(bridgeDirectory), now);
    }

    private static void RunDumpScan(
        string bridgeDirectory,
        string requestId,
        int? maxAccepted,
        CancellationToken cancellationToken)
    {
        try
        {
            var reader = new WindowsMemoryReader();
            var known = reader.LocateKnownModules();
            // game_plugin.dll often loads after BepInEx plugin Load — use live bounds, not a Load-time snapshot.
            var modules = ModulePresence.FromBounds(known);

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
                const string message =
                    "could not detect FM game_plugin.dll version; refusing scan (fail closed)";
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
                known.GamePlugin,
                maxAccepted,
                cancellationToken: cancellationToken);

            TryDeleteForceScanFlag(bridgeDirectory);

            if (result.Success)
            {
                WriteStatus(
                    bridgeDirectory,
                    BridgeProtocol.StateReady,
                    modules,
                    requestId: requestId,
                    playersFound: result.PlayerCount,
                    error: null,
                    scanTruncated: result.ScanTruncated,
                    maxAccepted: result.MaxAccepted);
                Log.LogInfo(
                    $"Dump request {requestId} wrote {result.PlayerCount} players"
                    + (result.ScanTruncated ? " (scan truncated)" : ""));
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
                    DetectModulesBestEffort(),
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

    private static void RunProbeScan(
        string bridgeDirectory,
        ProbeRequest request,
        CancellationToken cancellationToken)
    {
        try
        {
            var reader = new WindowsMemoryReader();
            var known = reader.LocateKnownModules();
            var modules = ModulePresence.FromBounds(known);

            WriteProbeStatus(
                bridgeDirectory,
                ProbeProtocol.StateScanning,
                modules,
                requestId: request.RequestId,
                playersCaptured: null,
                error: null);

            if (!GameVersionDetector.TryDetectFromCurrentProcess(out var gameVersion)
                || string.IsNullOrWhiteSpace(gameVersion))
            {
                const string message =
                    "could not detect FM game_plugin.dll version; refusing probe capture (fail closed)";
                WriteProbeStatus(
                    bridgeDirectory,
                    ProbeProtocol.StateFailed,
                    modules,
                    requestId: request.RequestId,
                    playersCaptured: null,
                    error: message);
                Log.LogError(message);
                return;
            }

            if (known.GameAssembly is not { } gameAssembly)
            {
                const string message = "GameAssembly.dll bounds not found; cannot capture probe";
                WriteProbeStatus(
                    bridgeDirectory,
                    ProbeProtocol.StateFailed,
                    modules,
                    requestId: request.RequestId,
                    playersCaptured: null,
                    error: message);
                Log.LogError(message);
                return;
            }

            var result = new ProbeCaptureService().RunAndWrite(
                reader,
                bridgeDirectory,
                request,
                gameVersion,
                MyPluginInfo.PLUGIN_VERSION,
                gameAssembly,
                known.GamePlugin,
                cancellationToken);

            if (result.Success)
            {
                WriteProbeStatus(
                    bridgeDirectory,
                    ProbeProtocol.StateReady,
                    modules,
                    requestId: request.RequestId,
                    playersCaptured: result.Document!.PlayerCount,
                    error: null);
                Log.LogInfo($"Probe request {request.RequestId} wrote {result.Document.PlayerCount} players");
            }
            else
            {
                WriteProbeStatus(
                    bridgeDirectory,
                    ProbeProtocol.StateFailed,
                    modules,
                    requestId: request.RequestId,
                    playersCaptured: null,
                    error: result.Error);
                Log.LogWarning($"Probe request {request.RequestId} failed: {result.Error}");
            }
        }
        catch (Exception ex)
        {
            Log.LogError($"Probe capture crashed: {ex}");
            try
            {
                WriteProbeStatus(
                    bridgeDirectory,
                    ProbeProtocol.StateFailed,
                    DetectModulesBestEffort(),
                    requestId: request.RequestId,
                    playersCaptured: null,
                    error: ex.Message);
            }
            catch
            {
                // ignored — probe status is best-effort during a crash
            }
        }
    }

    /// <summary>
    /// game_plugin.dll may appear after plugin Load; rewrite status module flags without changing state.
    /// </summary>
    private static void TryRefreshStaleModuleFlags(string bridgeDirectory)
    {
        lock (ScanGate)
        {
            if (_scanInProgress)
            {
                return;
            }
        }

        var modules = DetectModulesBestEffort();
        if (!StatusWriter.TryRead(bridgeDirectory, out var current) || current is null)
        {
            return;
        }

        if (current.GamePluginModulePresent == modules.GamePluginModulePresent
            && current.GameAssemblyModulePresent == modules.GameAssemblyModulePresent)
        {
            return;
        }

        lock (ScanGate)
        {
            if (_scanInProgress)
            {
                return;
            }
        }

        WriteStatus(
            bridgeDirectory,
            current.State,
            modules,
            requestId: current.RequestId,
            playersFound: current.PlayersFound,
            error: current.Error,
            scanTruncated: current.ScanTruncated,
            maxAccepted: current.MaxAccepted);
    }

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
        string? error,
        bool? scanTruncated = null,
        int? maxAccepted = null)
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
            ScanTruncated = scanTruncated,
            MaxAccepted = maxAccepted,
        };
        StatusWriter.Write(bridgeDirectory, status);
    }

    private static void WriteProbeStatus(
        string bridgeDirectory,
        string state,
        ModulePresenceSignals modules,
        string? requestId,
        int? playersCaptured,
        string? error)
    {
        var status = new ProbeStatus
        {
            ProtocolVersion = ProbeProtocol.ProtocolVersion,
            PluginVersion = MyPluginInfo.PLUGIN_VERSION,
            State = state,
            UpdatedAtUtc = DateTimeOffset.UtcNow,
            GamePluginModulePresent = modules.GamePluginModulePresent,
            GameAssemblyModulePresent = modules.GameAssemblyModulePresent,
            RequestId = requestId,
            PlayersCaptured = playersCaptured,
            Error = error,
        };
        ProbeStatusWriter.Write(bridgeDirectory, status);
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
