using BepInEx;
using BepInEx.Logging;
using BepInEx.Unity.IL2CPP;
using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Mutations;
using FmDataBridge.Output;
using FmDataBridge.Protocol;
using FmDataBridge.Scanning;

namespace FmDataBridge;

[BepInPlugin(MyPluginInfo.PLUGIN_GUID, MyPluginInfo.PLUGIN_NAME, MyPluginInfo.PLUGIN_VERSION)]
public class Plugin : BasePlugin
{
    internal static new ManualLogSource Log = null!;

    private CancellationTokenSource? _pollCts;

    private Thread? _pollThread;

    private static CancellationTokenSource? s_unloadCts;

    private static Thread? s_operationThread;

    private static readonly TimeSpan RequestPollInterval = TimeSpan.FromSeconds(2);

    private static readonly TimeSpan RequestTtl = TimeSpan.FromSeconds(BridgeProtocol.RequestTtlSeconds);

    private static readonly TimeSpan PollThreadJoinTimeout = TimeSpan.FromSeconds(30);

    private static readonly TimeSpan ScanThreadJoinTimeout = TimeSpan.FromSeconds(30);

    private static readonly object WorkStartGate = new();

    private static readonly BridgeWorkGate WorkGate = new();

    private static readonly LayoutRegistry Layouts = LayoutRegistry.CreateDefault();

    private static readonly PlayerMutationIndex PlayerMutationIndex = new();

    private static readonly PlayerBoostOperationService PlayerBoosts = new(Layouts, PlayerMutationIndex);

    private static readonly StaffMutationIndex StaffMutationIndex = new();

    private static readonly StaffBoostOperationService StaffBoosts = new(Layouts, StaffMutationIndex);

    public override void Load()
    {
        Log = base.Log;

        try
        {
            var bridgeDirectory = BridgePaths.EnsureBridgeDirectory();
            PlayerMutationIndex.Clear();
            StaffMutationIndex.Clear();
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

        if (s_operationThread is { IsAlive: true } operationThread
            && !operationThread.Join(ScanThreadJoinTimeout))
        {
            Log.LogWarning(
                $"Bridge work thread did not exit within {ScanThreadJoinTimeout.TotalSeconds:0}s");
        }

        PlayerMutationIndex.Clear();
        StaffMutationIndex.Clear();

        return true;
    }

    private static void PollRequests(string bridgeDirectory, CancellationToken token)
    {
        while (!token.IsCancellationRequested)
        {
            try
            {
                TryStartBridgeWorkFromRequestOrForceFlag(bridgeDirectory);
                TryRefreshStaleModuleFlags(bridgeDirectory);
            }
            catch (Exception ex)
            {
                Log.LogError($"Request poll iteration failed: {ex}");
            }

            token.WaitHandle.WaitOne(RequestPollInterval);
        }
    }

    private static void TryStartBridgeWorkFromRequestOrForceFlag(string bridgeDirectory)
    {
        BridgeRequest? request = null;
        var playerDatabaseScope = PlayerDatabaseScope.Men;

        lock (WorkStartGate)
        {
            if (WorkGate.IsBusy)
            {
                // Keep a waiting request.json fresh so a long dump or boost does not TTL-kill it.
                RequestAcceptance.TryRefreshCreatedAtUtc(
                    BridgePaths.GetRequestPath(bridgeDirectory),
                    DateTimeOffset.UtcNow,
                    RequestTtl);
                return;
            }

            var requestPath = BridgePaths.GetRequestPath(bridgeDirectory);
            if (File.Exists(requestPath))
            {
                if (!RequestAcceptance.TryAccept(
                        requestPath,
                        DateTimeOffset.UtcNow,
                        RequestTtl,
                        out request,
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
            }
            else if (File.Exists(BridgePaths.GetForceScanPath(bridgeDirectory)))
            {
                // Manual fallback until operators prefer only the in-app request path.
                // Unlimited — same as production Load Data (null maxAccepted).
                request = ForceScanRequestFactory.Create(DateTimeOffset.UtcNow);
            }
            else
            {
                return;
            }

            if (request.Operation == BridgeProtocol.OperationFullDump
                && !PlayerDatabaseScopes.TryParse(request.PlayerDatabaseScope, out playerDatabaseScope))
            {
                throw new InvalidOperationException("Accepted request has an invalid player database scope.");
            }

            if (!WorkGate.TryEnter())
            {
                WriteStatus(
                    bridgeDirectory,
                    BridgeProtocol.StateFailed,
                    DetectModulesBestEffort(),
                    requestId: request.RequestId,
                    playersFound: null,
                    error: "bridge work is already in progress; retry the request");
                return;
            }
        }

        var acceptedRequest = request!;
        var acceptedScope = playerDatabaseScope;
        var cancelToken = s_unloadCts?.Token ?? CancellationToken.None;
        s_operationThread = new Thread(() =>
        {
            try
            {
                if (acceptedRequest.Operation == BridgeProtocol.OperationFullDump)
                {
                    RunDumpScan(
                        bridgeDirectory,
                        acceptedRequest.RequestId,
                        acceptedRequest.MaxAccepted,
                        acceptedScope,
                        cancelToken);
                }
                else if (acceptedRequest.Operation == BridgeProtocol.OperationBoostStaffCurrentAbility)
                {
                    RunStaffBoost(bridgeDirectory, acceptedRequest, cancelToken);
                }
                else
                {
                    RunPlayerBoost(bridgeDirectory, acceptedRequest, cancelToken);
                }
            }
            finally
            {
                WorkGate.Exit();
            }
        })
        {
            IsBackground = true,
            Name = "FmBridge-Work",
        };
        s_operationThread.Start();
    }

    private static void RunDumpScan(
        string bridgeDirectory,
        string requestId,
        int? maxAccepted,
        PlayerDatabaseScope playerDatabaseScope,
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

            var result = new CapADumpPipeline(Layouts).Run(
                reader,
                bridgeDirectory,
                gameVersion,
                MyPluginInfo.PLUGIN_VERSION,
                gameAssembly,
                known.GamePlugin,
                maxAccepted,
                playerDatabaseScope,
                cancellationToken: cancellationToken);

            TryDeleteForceScanFlag(bridgeDirectory);

            if (result.Success)
            {
                if (result.LivePlayerCandidates.Count > 0
                    && PlayerBoosts.SupportsExactGameBuild(gameVersion))
                {
                    PlayerMutationIndex.Replace(requestId, gameVersion, result.LivePlayerCandidates);
                }
                else
                {
                    PlayerMutationIndex.Clear();
                }

                if (result.LiveStaffCandidates.Count > 0
                    && StaffBoosts.SupportsExactGameBuild(gameVersion))
                {
                    StaffMutationIndex.Replace(requestId, gameVersion, result.LiveStaffCandidates);
                }
                else
                {
                    StaffMutationIndex.Clear();
                }

                var playerBoostsSupported = PlayerBoosts.HasSupportedLiveIndex(gameVersion);
                var staffBoostsSupported = StaffBoosts.HasSupportedLiveIndex(gameVersion);

                WriteStatus(
                    bridgeDirectory,
                    BridgeProtocol.StateReady,
                    modules,
                    requestId: requestId,
                    playersFound: result.PlayerCount,
                    error: null,
                    scanTruncated: result.ScanTruncated,
                    maxAccepted: result.MaxAccepted,
                    playerBoostsSupported: playerBoostsSupported,
                    staffBoostsSupported: staffBoostsSupported);
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
                    error: result.Error,
                    playerBoostsSupported: PlayerBoosts.HasSupportedLiveIndex(gameVersion),
                    staffBoostsSupported: StaffBoosts.HasSupportedLiveIndex(gameVersion));
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
                    error: "scan failed unexpectedly");
            }
            catch
            {
                // ignored — status best-effort
            }
        }
    }

    private static void RunPlayerBoost(
        string bridgeDirectory,
        BridgeRequest request,
        CancellationToken cancellationToken)
    {
        try
        {
            var reader = new WindowsMemoryReader();
            var known = reader.LocateKnownModules();
            var modules = ModulePresence.FromBounds(known);
            WriteStatus(
                bridgeDirectory,
                BridgeProtocol.StateScanning,
                modules,
                requestId: request.RequestId,
                playersFound: null,
                error: null,
                playerBoostsSupported: false);

            if (cancellationToken.IsCancellationRequested)
            {
                WriteStatus(
                    bridgeDirectory,
                    BridgeProtocol.StateFailed,
                    modules,
                    requestId: request.RequestId,
                    playersFound: null,
                    error: "player boost cancelled before it started",
                    playerBoostsSupported: false);
                return;
            }

            if (!GameVersionDetector.TryDetectFromCurrentProcess(out var gameVersion)
                || string.IsNullOrWhiteSpace(gameVersion))
            {
                const string message =
                    "could not detect FM game_plugin.dll version; refusing player boost (fail closed)";
                WriteStatus(
                    bridgeDirectory,
                    BridgeProtocol.StateFailed,
                    modules,
                    requestId: request.RequestId,
                    playersFound: null,
                    error: message,
                    playerBoostsSupported: false);
                Log.LogWarning(message);
                return;
            }

            var result = PlayerBoosts.Execute(request, gameVersion, reader, reader);
            if (result.Succeeded)
            {
                WriteStatus(
                    bridgeDirectory,
                    BridgeProtocol.StateReady,
                    modules,
                    requestId: request.RequestId,
                    playersFound: null,
                    error: null,
                    playerBoostsSupported: PlayerBoosts.HasSupportedLiveIndex(gameVersion),
                    staffBoostsSupported: StaffBoosts.HasSupportedLiveIndex(gameVersion),
                    playerBoost: result.BoostResult);
                return;
            }

            PlayerMutationIndex.Clear();
            StaffMutationIndex.Clear();

            WriteStatus(
                bridgeDirectory,
                BridgeProtocol.StateFailed,
                modules,
                requestId: request.RequestId,
                playersFound: null,
                error: PlayerBoostFailureMessage(result.Failure),
                playerBoostsSupported: false,
                staffBoostsSupported: false,
                playerBoost: result.BoostResult);
        }
        catch (Exception ex)
        {
            Log.LogError($"Player boost crashed: {ex}");
            PlayerMutationIndex.Clear();
            StaffMutationIndex.Clear();
            try
            {
                WriteStatus(
                    bridgeDirectory,
                    BridgeProtocol.StateFailed,
                    DetectModulesBestEffort(),
                    requestId: request.RequestId,
                    playersFound: null,
                    error: "player boost failed unexpectedly",
                    playerBoostsSupported: false,
                    staffBoostsSupported: false);
            }
            catch
            {
                // ignored — status best-effort
            }
        }
    }

    private static string PlayerBoostFailureMessage(PlayerBoostFailure failure) =>
        failure switch
        {
            PlayerBoostFailure.InvalidRequest => "invalid player boost request; update the app and retry",
            PlayerBoostFailure.UnsupportedGameBuild =>
                "this FM build is not approved for player boosts; update the bridge plugin and Load Data",
            PlayerBoostFailure.NoLiveScan => "Load Data again before using player boosts",
            PlayerBoostFailure.SourceRequestMismatch => "Load Data again before using player boosts",
            PlayerBoostFailure.PlayerNotFound => "player was not found in the latest live scan; Load Data again",
            PlayerBoostFailure.ExpectedValuesMismatch => "player values changed in FM; Load Data again",
            PlayerBoostFailure.LiveIdentityMismatch => "player identity changed in FM; Load Data again",
            PlayerBoostFailure.LiveReadFailed => "could not safely read the player; Load Data again",
            PlayerBoostFailure.InvalidLiveValue => "player values are not valid for this boost; Load Data again",
            PlayerBoostFailure.CurrentAbilityAtLimit => "current ability is already at its potential limit",
            PlayerBoostFailure.MutationFailed => "player boost could not be verified; Load Data again",
            PlayerBoostFailure.PartialRollbackUnverified =>
                "player boost could not verify rollback; Load Data again before making another change",
            _ => "player boost failed; Load Data again",
        };

    private static void RunStaffBoost(
        string bridgeDirectory,
        BridgeRequest request,
        CancellationToken cancellationToken)
    {
        try
        {
            var reader = new WindowsMemoryReader();
            var known = reader.LocateKnownModules();
            var modules = ModulePresence.FromBounds(known);
            WriteStatus(
                bridgeDirectory,
                BridgeProtocol.StateScanning,
                modules,
                requestId: request.RequestId,
                playersFound: null,
                error: null,
                staffBoostsSupported: false);

            if (cancellationToken.IsCancellationRequested)
            {
                WriteStatus(
                    bridgeDirectory,
                    BridgeProtocol.StateFailed,
                    modules,
                    requestId: request.RequestId,
                    playersFound: null,
                    error: "staff boost cancelled before it started",
                    staffBoostsSupported: false);
                return;
            }

            if (!GameVersionDetector.TryDetectFromCurrentProcess(out var gameVersion)
                || string.IsNullOrWhiteSpace(gameVersion))
            {
                const string message =
                    "could not detect FM game_plugin.dll version; refusing staff boost (fail closed)";
                WriteStatus(
                    bridgeDirectory,
                    BridgeProtocol.StateFailed,
                    modules,
                    requestId: request.RequestId,
                    playersFound: null,
                    error: message,
                    staffBoostsSupported: false);
                Log.LogWarning(message);
                return;
            }

            var result = StaffBoosts.Execute(request, gameVersion, reader, reader);
            if (result.Succeeded)
            {
                WriteStatus(
                    bridgeDirectory,
                    BridgeProtocol.StateReady,
                    modules,
                    requestId: request.RequestId,
                    playersFound: null,
                    error: null,
                    playerBoostsSupported: PlayerBoosts.HasSupportedLiveIndex(gameVersion),
                    staffBoostsSupported: StaffBoosts.HasSupportedLiveIndex(gameVersion),
                    staffBoost: result.BoostResult);
                return;
            }

            StaffMutationIndex.Clear();
            PlayerMutationIndex.Clear();
            WriteStatus(
                bridgeDirectory,
                BridgeProtocol.StateFailed,
                modules,
                requestId: request.RequestId,
                playersFound: null,
                error: StaffBoostFailureMessage(result.Failure),
                playerBoostsSupported: false,
                staffBoostsSupported: false,
                staffBoost: result.BoostResult);
        }
        catch (Exception ex)
        {
            Log.LogError($"Staff boost crashed: {ex}");
            StaffMutationIndex.Clear();
            PlayerMutationIndex.Clear();
            try
            {
                WriteStatus(
                    bridgeDirectory,
                    BridgeProtocol.StateFailed,
                    DetectModulesBestEffort(),
                    requestId: request.RequestId,
                    playersFound: null,
                    error: "staff boost failed unexpectedly",
                    playerBoostsSupported: false,
                    staffBoostsSupported: false);
            }
            catch
            {
                // ignored — status best-effort
            }
        }
    }

    private static string StaffBoostFailureMessage(StaffBoostFailure failure) =>
        failure switch
        {
            StaffBoostFailure.InvalidRequest => "invalid staff boost request; update the app and retry",
            StaffBoostFailure.UnsupportedGameBuild =>
                "this FM build is not approved for staff boosts; update the bridge plugin and Load Data",
            StaffBoostFailure.NoLiveScan => "Load Data again before using staff boosts",
            StaffBoostFailure.SourceRequestMismatch => "Load Data again before using staff boosts",
            StaffBoostFailure.StaffNotFound => "staff member was not found in the latest live scan; Load Data again",
            StaffBoostFailure.ExpectedValuesMismatch => "staff values changed in FM; Load Data again",
            StaffBoostFailure.LiveIdentityMismatch => "staff identity changed in FM; Load Data again",
            StaffBoostFailure.LiveReadFailed => "could not safely read the staff member; Load Data again",
            StaffBoostFailure.InvalidLiveValue => "staff values are not valid for this boost; Load Data again",
            StaffBoostFailure.CurrentAbilityAtLimit => "current ability is already at its potential limit",
            StaffBoostFailure.MutationFailed => "staff boost could not be verified; Load Data again",
            StaffBoostFailure.PartialRollbackUnverified =>
                "staff boost could not verify rollback; Load Data again before making another change",
            _ => "staff boost failed; Load Data again",
        };

    /// <summary>
    /// game_plugin.dll may appear after plugin Load; rewrite status module flags without changing state.
    /// </summary>
    private static void TryRefreshStaleModuleFlags(string bridgeDirectory)
    {
        if (WorkGate.IsBusy)
        {
            return;
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

        if (WorkGate.IsBusy)
        {
            return;
        }

        WriteStatus(
            bridgeDirectory,
            current.State,
            modules,
            requestId: current.RequestId,
            playersFound: current.PlayersFound,
            error: current.Error,
            scanTruncated: current.ScanTruncated,
            maxAccepted: current.MaxAccepted,
            playerBoostsSupported: current.PlayerBoostsSupported,
            staffBoostsSupported: current.StaffBoostsSupported,
            playerBoost: current.PlayerBoost,
            staffBoost: current.StaffBoost);
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
            error: null,
            playerBoostsSupported: false,
            staffBoostsSupported: false);

    private static void WriteStatus(
        string bridgeDirectory,
        string state,
        ModulePresenceSignals modules,
        string? requestId,
        int? playersFound,
        string? error,
        bool? scanTruncated = null,
        int? maxAccepted = null,
        bool? playerBoostsSupported = null,
        bool? staffBoostsSupported = null,
        PlayerBoostResult? playerBoost = null,
        StaffBoostResult? staffBoost = null)
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
            PlayerBoostsSupported = playerBoostsSupported,
            StaffBoostsSupported = staffBoostsSupported,
            PlayerBoost = playerBoost,
            StaffBoost = staffBoost,
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
