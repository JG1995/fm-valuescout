namespace FmDataBridge.Protocol;

public static class BridgeProtocol
{
    public const int ProtocolVersion = 1;
    public const int DumpSchemaVersion = 10;

    /// <summary>Ignore request.json older than this many seconds (SuperScout-class stale fix).</summary>
    public const int RequestTtlSeconds = 30;

    public const string StateIdle = "idle";
    public const string StateScanning = "scanning";
    public const string StateReady = "ready";
    public const string StateFailed = "failed";
    public const string OperationFullDump = "full-dump";
    public const string OperationBoostCurrentAbility = "boost-current-ability";
    public const string OperationBoostStaffCurrentAbility = "boost-staff-current-ability";
    public const string OperationWonderkidMentality = "wonderkid-mentality";
    public const string OperationMoveSquadPlayers = "move-squad-players";
    public const string OperationApplyStaffChanges = "apply-staff-changes";
    public const string OperationReadStaffLimits = "read-staff-limits";
    public const string OperationApplyTraining = "apply-training";

    /// <summary>Most players one squad-move request may move.</summary>
    public const int MaxSquadMoves = 100;

    /// <summary>Most staff one staff-change request may sack, hire, or re-hire.</summary>
    public const int MaxStaffChanges = 100;

    /// <summary>Most players one training request may set training for: a whole club, all teams.</summary>
    public const int MaxTrainingChanges = 250;
    public const string AppFolderName = "fm-valuescout";
    public const string BridgeFolderName = "fm-bridge";
    public const string StatusFileName = "status.json";
    public const string RequestFileName = "request.json";
    public const string DumpFileName = "dump.json";
    public const string DiagnosticsFileName = "diagnostics.txt";
    public const string ForceScanFileName = "force-scan";
}
