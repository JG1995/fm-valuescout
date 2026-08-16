using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Protocol;
using FmDataBridge.Scanning;

namespace FmDataBridge.Mutations;

internal enum StaffBoostFailure
{
    None,
    InvalidRequest,
    UnsupportedGameBuild,
    NoLiveScan,
    SourceRequestMismatch,
    StaffNotFound,
    ExpectedValuesMismatch,
    LiveIdentityMismatch,
    LiveReadFailed,
    InvalidLiveValue,
    CurrentAbilityAtLimit,
    MutationFailed,
    PartialRollbackUnverified,
}

internal readonly record struct StaffBoostExecutionResult(
    bool Succeeded,
    StaffBoostFailure Failure,
    StaffBoostResult? BoostResult);

/// <summary>Executes only the fixed, capped staff CA boost against one live staff candidate.</summary>
internal sealed class StaffBoostOperationService
{
    private const int Increment = 10;
    private const int MaxAbility = 200;

    private readonly LayoutRegistry _layouts;
    private readonly StaffMutationIndex _index;
    private readonly Func<string, bool>? _supportsExactGameBuild;

    public StaffBoostOperationService(
        LayoutRegistry layouts,
        StaffMutationIndex index,
        Func<string, bool>? supportsExactGameBuild = null)
    {
        _layouts = layouts ?? throw new ArgumentNullException(nameof(layouts));
        _index = index ?? throw new ArgumentNullException(nameof(index));
        _supportsExactGameBuild = supportsExactGameBuild;
    }

    public bool SupportsExactGameBuild(string gameVersion) =>
        _layouts.TryResolveFromGameVersion(gameVersion, out var layout)
        && (_supportsExactGameBuild?.Invoke(gameVersion)
            ?? layout.SupportsStaffBoosts(gameVersion));

    public bool HasSupportedLiveIndex(string gameVersion) =>
        SupportsExactGameBuild(gameVersion)
        && _index.HasCandidatesForGameVersion(gameVersion);

    public StaffBoostExecutionResult Execute(
        BridgeRequest request,
        string gameVersion,
        IMemoryReader reader,
        IMemoryWriter writer)
    {
        ArgumentNullException.ThrowIfNull(request);
        ArgumentNullException.ThrowIfNull(reader);
        ArgumentNullException.ThrowIfNull(writer);

        if (!TryGetRequest(request, out var sourceRequestId, out var staffUid, out var expectedCa, out var expectedPa))
        {
            return Failed(StaffBoostFailure.InvalidRequest);
        }

        if (!_layouts.TryResolveFromGameVersion(gameVersion, out var layout)
            || !SupportsExactGameBuild(gameVersion))
        {
            return Failed(StaffBoostFailure.UnsupportedGameBuild);
        }

        var lookup = _index.TryGet(sourceRequestId, staffUid, out var indexed);
        if (lookup != StaffMutationLookup.Found)
        {
            return Failed(MapLookupFailure(lookup));
        }

        if (!string.Equals(indexed.GameVersion, gameVersion, StringComparison.Ordinal))
        {
            return Failed(StaffBoostFailure.SourceRequestMismatch);
        }

        var staff = indexed.Candidate;
        if (staff.Ca != expectedCa || staff.Pa != expectedPa)
        {
            return Failed(StaffBoostFailure.ExpectedValuesMismatch);
        }

        var liveFailure = TryReadLiveState(reader, staff, layout, expectedCa, expectedPa, out var liveCa, out var livePa);
        if (liveFailure != StaffBoostFailure.None)
        {
            return Failed(liveFailure);
        }

        var target = Math.Min(liveCa + Increment, Math.Min(livePa, MaxAbility));
        if (target <= liveCa)
        {
            return Failed(StaffBoostFailure.CurrentAbilityAtLimit);
        }

        var mutation = new StaffValueMutationService(reader, writer, layout)
            .SetCurrentAbility(staff, liveCa, livePa, target);
        if (!mutation.Succeeded)
        {
            return FailedFromMutation(mutation, livePa);
        }

        if (mutation.VerifiedValue is not { } verified)
        {
            return Failed(StaffBoostFailure.MutationFailed);
        }

        if (!_index.TryUpdateCurrentAbility(
                sourceRequestId,
                staff.Uid,
                liveCa,
                livePa,
                verified))
        {
            return new StaffBoostExecutionResult(
                false,
                StaffBoostFailure.PartialRollbackUnverified,
                new StaffBoostResult
                {
                    Operation = BridgeProtocol.OperationBoostStaffCurrentAbility,
                    Outcome = "partial-unverified",
                    Rollback = "unverified",
                    PreviousCurrentAbility = liveCa,
                    CurrentAbility = verified,
                    PotentialAbility = livePa,
                });
        }

        return new StaffBoostExecutionResult(
            true,
            StaffBoostFailure.None,
            new StaffBoostResult
            {
                Operation = BridgeProtocol.OperationBoostStaffCurrentAbility,
                Outcome = "verified",
                Rollback = "not-needed",
                PreviousCurrentAbility = liveCa,
                CurrentAbility = verified,
                PotentialAbility = livePa,
            });
    }

    private static StaffBoostFailure TryReadLiveState(
        IMemoryReader reader,
        PersonCandidate staff,
        IFmMemoryLayout layout,
        int expectedCa,
        int expectedPa,
        out int liveCa,
        out int livePa)
    {
        liveCa = 0;
        livePa = 0;
        if (staff.Facet is not PersonFacet.Staff and not PersonFacet.HumanManager
            || !TryAdd(staff.ObjectAddress, layout.ObjectUidOffset, out var uidAddress)
            || !TryAdd(staff.BlockAddress, layout.StaffCurrentAbilityOffset, out var caAddress)
            || !TryAdd(staff.BlockAddress, layout.StaffPotentialAbilityOffset, out var paAddress))
        {
            return StaffBoostFailure.LiveReadFailed;
        }

        if (!reader.TryReadUInt32(uidAddress, out var uid)
            || !reader.TryReadUInt16(caAddress, out var ca)
            || !reader.TryReadUInt16(paAddress, out var pa))
        {
            return StaffBoostFailure.LiveReadFailed;
        }

        if (uid != staff.Uid)
        {
            return StaffBoostFailure.LiveIdentityMismatch;
        }

        if (!PersonScanner.IsValidAbility(ca) || !PersonScanner.IsValidAbility(pa))
        {
            return StaffBoostFailure.InvalidLiveValue;
        }

        liveCa = ca;
        livePa = pa;
        return ca == expectedCa && pa == expectedPa
            ? StaffBoostFailure.None
            : StaffBoostFailure.ExpectedValuesMismatch;
    }

    private static bool TryGetRequest(
        BridgeRequest request,
        out string sourceRequestId,
        out uint staffUid,
        out int expectedCa,
        out int expectedPa)
    {
        sourceRequestId = request.SourceRequestId ?? "";
        staffUid = request.StaffUid ?? 0;
        expectedCa = request.ExpectedCurrentAbility ?? 0;
        expectedPa = request.ExpectedPotentialAbility ?? 0;
        return request.Operation == BridgeProtocol.OperationBoostStaffCurrentAbility
            && !string.IsNullOrWhiteSpace(sourceRequestId)
            && staffUid != 0
            && request.PlayerUid is null
            && request.CurrentAbilityIncrement is null
            && request.ExpectedAmbition is null
            && request.ExpectedProfessionalism is null
            && request.ExpectedDetermination is null
            && PersonScanner.IsValidAbility(expectedCa)
            && PersonScanner.IsValidAbility(expectedPa)
            && expectedCa <= expectedPa;
    }

    private static StaffBoostExecutionResult FailedFromMutation(
        StaffValueMutationResult mutation,
        int potentialAbility)
    {
        var failure = mutation.Rollback == StaffValueRollback.Unverified
            ? StaffBoostFailure.PartialRollbackUnverified
            : mutation.Failure == StaffValueMutationFailure.ExpectedValueMismatch
                ? StaffBoostFailure.ExpectedValuesMismatch
                : StaffBoostFailure.MutationFailed;
        return new StaffBoostExecutionResult(
            false,
            failure,
            new StaffBoostResult
            {
                Operation = BridgeProtocol.OperationBoostStaffCurrentAbility,
                Outcome = mutation.Rollback == StaffValueRollback.Unverified ? "partial-unverified" : "failed",
                Rollback = mutation.Rollback switch
                {
                    StaffValueRollback.NotNeeded => "not-needed",
                    StaffValueRollback.Restored => "restored",
                    StaffValueRollback.Unverified => "unverified",
                    _ => throw new ArgumentOutOfRangeException(nameof(mutation)),
                },
                PreviousCurrentAbility = mutation.PreviousValue,
                PotentialAbility = potentialAbility,
            });
    }

    private static StaffBoostFailure MapLookupFailure(StaffMutationLookup lookup) =>
        lookup switch
        {
            StaffMutationLookup.MissingIndex => StaffBoostFailure.NoLiveScan,
            StaffMutationLookup.SourceRequestMismatch => StaffBoostFailure.SourceRequestMismatch,
            StaffMutationLookup.StaffNotFound => StaffBoostFailure.StaffNotFound,
            _ => throw new ArgumentOutOfRangeException(nameof(lookup)),
        };

    private static StaffBoostExecutionResult Failed(StaffBoostFailure failure) =>
        new(false, failure, null);

    private static bool TryAdd(ulong address, int offset, out ulong result)
    {
        result = 0;
        if (offset < 0 || (ulong)offset > ulong.MaxValue - address)
        {
            return false;
        }

        result = address + (ulong)offset;
        return true;
    }
}
