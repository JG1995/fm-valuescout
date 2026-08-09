using FmDataBridge.Extraction;
using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Protocol;
using FmDataBridge.Scanning;

namespace FmDataBridge.Mutations;

internal enum PlayerBoostFailure
{
    None,
    InvalidRequest,
    UnsupportedGameBuild,
    NoLiveScan,
    SourceRequestMismatch,
    PlayerNotFound,
    ExpectedValuesMismatch,
    LiveIdentityMismatch,
    LiveReadFailed,
    InvalidLiveValue,
    CurrentAbilityAtLimit,
    MutationFailed,
    PartialRollbackUnverified,
}

internal readonly record struct PlayerBoostExecutionResult(
    bool Succeeded,
    PlayerBoostFailure Failure,
    PlayerBoostResult? BoostResult);

/// <summary>
/// Executes only the two accepted player boosts against candidates from one successful live dump.
/// </summary>
internal sealed class PlayerBoostOperationService
{
    private const int MinAbility = 1;
    private const int MaxAbility = 200;
    private const int MentalityEligibilityMaximum = 10;
    private const int MentalityTargetMinimum = 11;
    private const int MentalityTargetMaximumExclusive = 21;

    private readonly LayoutRegistry _layouts;
    private readonly PlayerMutationIndex _index;
    private readonly Func<int, int, int> _nextRandom;

    public PlayerBoostOperationService(
        LayoutRegistry layouts,
        PlayerMutationIndex index,
        Func<int, int, int>? nextRandom = null)
    {
        _layouts = layouts ?? throw new ArgumentNullException(nameof(layouts));
        _index = index ?? throw new ArgumentNullException(nameof(index));
        _nextRandom = nextRandom ?? Random.Shared.Next;
    }

    public bool SupportsExactGameBuild(string gameVersion) =>
        _layouts.TryResolveFromGameVersion(gameVersion, out var layout)
        && layout.SupportsPlayerBoosts(gameVersion);

    public bool HasSupportedLiveIndex(string gameVersion) =>
        SupportsExactGameBuild(gameVersion)
        && _index.HasCandidatesForGameVersion(gameVersion);

    public PlayerBoostExecutionResult Execute(
        BridgeRequest request,
        string gameVersion,
        IMemoryReader reader,
        IMemoryWriter writer)
    {
        ArgumentNullException.ThrowIfNull(request);
        if (string.IsNullOrWhiteSpace(gameVersion))
        {
            throw new ArgumentException("A game version is required.", nameof(gameVersion));
        }
        ArgumentNullException.ThrowIfNull(reader);
        ArgumentNullException.ThrowIfNull(writer);

        if (!TryGetBoostRequest(request, out var sourceRequestId, out var playerUid, out var expectedCa, out var expectedPa))
        {
            return Failed(PlayerBoostFailure.InvalidRequest);
        }

        if (!_layouts.TryResolveFromGameVersion(gameVersion, out var layout)
            || !layout.SupportsPlayerBoosts(gameVersion))
        {
            return Failed(PlayerBoostFailure.UnsupportedGameBuild);
        }

        var lookup = _index.TryGet(sourceRequestId, playerUid, out var indexed);
        if (lookup != PlayerMutationLookup.Found)
        {
            return Failed(MapLookupFailure(lookup));
        }

        if (!string.Equals(indexed.GameVersion, gameVersion, StringComparison.Ordinal))
        {
            return Failed(PlayerBoostFailure.SourceRequestMismatch);
        }

        var player = indexed.Candidate;
        if (player.Ca != expectedCa || player.Pa != expectedPa)
        {
            return Failed(PlayerBoostFailure.ExpectedValuesMismatch);
        }

        var liveFailure = TryReadLiveAbilityState(
            reader,
            player,
            layout,
            expectedCa,
            expectedPa,
            out var live);
        if (liveFailure != PlayerBoostFailure.None)
        {
            return Failed(liveFailure);
        }

        var mutations = new PlayerValueMutationService(reader, writer, layout);
        return request.Operation switch
        {
            BridgeProtocol.OperationBoostCurrentAbility => ExecuteCurrentAbility(
                request,
                sourceRequestId,
                player,
                live,
                mutations),
            BridgeProtocol.OperationWonderkidMentality => ExecuteWonderkidMentality(
                request,
                player,
                live,
                layout,
                reader,
                mutations),
            _ => Failed(PlayerBoostFailure.InvalidRequest),
        };
    }

    private PlayerBoostExecutionResult ExecuteCurrentAbility(
        BridgeRequest request,
        string sourceRequestId,
        PersonCandidate player,
        LiveAbilityState live,
        PlayerValueMutationService mutations)
    {
        if (request.CurrentAbilityIncrement is not 5 and not 10
            || request.ExpectedAmbition is not null
            || request.ExpectedProfessionalism is not null
            || request.ExpectedDetermination is not null)
        {
            return Failed(PlayerBoostFailure.InvalidRequest);
        }

        var target = Math.Min(
            live.CurrentAbility + request.CurrentAbilityIncrement.Value,
            Math.Min(live.PotentialAbility, MaxAbility));
        if (target <= live.CurrentAbility)
        {
            return Failed(PlayerBoostFailure.CurrentAbilityAtLimit);
        }

        var mutation = mutations.SetCurrentAbility(
            player,
            live.CurrentAbility,
            live.PotentialAbility,
            target);
        if (!mutation.Succeeded)
        {
            return FailedFromMutation(
                BridgeProtocol.OperationBoostCurrentAbility,
                mutation,
                live);
        }

        if (mutation.VerifiedValue is not { } verifiedCurrentAbility
            || !_index.TryUpdateCurrentAbility(
                sourceRequestId,
                player.Uid,
                live.CurrentAbility,
                live.PotentialAbility,
                verifiedCurrentAbility))
        {
            _index.Clear();
        }

        return Succeeded(
            new PlayerBoostResult
            {
                Operation = BridgeProtocol.OperationBoostCurrentAbility,
                Outcome = "verified",
                Rollback = ToWireRollback(mutation.Rollback),
                PreviousCurrentAbility = mutation.PreviousValue,
                CurrentAbility = mutation.VerifiedValue,
                PotentialAbility = live.PotentialAbility,
            });
    }

    private PlayerBoostExecutionResult ExecuteWonderkidMentality(
        BridgeRequest request,
        PersonCandidate player,
        LiveAbilityState live,
        IFmMemoryLayout layout,
        IMemoryReader reader,
        PlayerValueMutationService mutations)
    {
        if (!TryGetMentalityExpectations(request, out var expectations))
        {
            return Failed(PlayerBoostFailure.InvalidRequest);
        }

        var readFailure = TryReadMentality(reader, player, layout, expectations, out var mentality);
        if (readFailure != PlayerBoostFailure.None)
        {
            return Failed(readFailure);
        }

        if (!MatchesMentalityExpectations(mentality, expectations))
        {
            return Failed(PlayerBoostFailure.ExpectedValuesMismatch);
        }

        var targets = new MentalityState(
            ChooseMentalityTarget(mentality.Ambition),
            ChooseMentalityTarget(mentality.Professionalism),
            ChooseMentalityTarget(mentality.Determination));
        var applied = new List<MentalityMutation>();

        var ambition = ApplyMentalityChange(
            MentalityField.Ambition,
            player,
            mentality.Ambition,
            targets.Ambition,
            mutations,
            applied);
        if (!ambition.Succeeded)
        {
            return FailedMentalityMutation(
                ambition,
                player,
                mentality,
                layout,
                reader,
                mutations,
                applied);
        }

        var professionalism = ApplyMentalityChange(
            MentalityField.Professionalism,
            player,
            mentality.Professionalism,
            targets.Professionalism,
            mutations,
            applied);
        if (!professionalism.Succeeded)
        {
            return FailedMentalityMutation(
                professionalism,
                player,
                mentality,
                layout,
                reader,
                mutations,
                applied);
        }

        var determination = ApplyMentalityChange(
            MentalityField.Determination,
            player,
            mentality.Determination,
            targets.Determination,
            mutations,
            applied);
        if (!determination.Succeeded)
        {
            return FailedMentalityMutation(
                determination,
                player,
                mentality,
                layout,
                reader,
                mutations,
                applied);
        }

        return Succeeded(
            new PlayerBoostResult
            {
                Operation = BridgeProtocol.OperationWonderkidMentality,
                Outcome = "verified",
                Rollback = "not-needed",
                PreviousCurrentAbility = live.CurrentAbility,
                CurrentAbility = live.CurrentAbility,
                PotentialAbility = live.PotentialAbility,
                PreviousAmbition = mentality.Ambition,
                Ambition = targets.Ambition,
                PreviousProfessionalism = mentality.Professionalism,
                Professionalism = targets.Professionalism,
                PreviousDetermination = mentality.Determination,
                Determination = targets.Determination,
            });
    }

    private PlayerBoostExecutionResult FailedMentalityMutation(
        PlayerValueMutationResult failedMutation,
        PersonCandidate player,
        MentalityState previous,
        IFmMemoryLayout layout,
        IMemoryReader reader,
        PlayerValueMutationService mutations,
        IReadOnlyList<MentalityMutation> applied)
    {
        var rollback = CombineRollback(
            failedMutation.Rollback,
            RollbackAppliedMentality(mutations, player, applied));
        MentalityState? observed = TryReadMentality(reader, player, layout, previous, out var current)
            == PlayerBoostFailure.None
            ? current
            : null;

        var failure = rollback == PlayerValueRollback.Unverified
            ? PlayerBoostFailure.PartialRollbackUnverified
            : MapMutationFailure(failedMutation.Failure);
        return new PlayerBoostExecutionResult(
            Succeeded: false,
            failure,
            new PlayerBoostResult
            {
                Operation = BridgeProtocol.OperationWonderkidMentality,
                Outcome = rollback == PlayerValueRollback.Unverified ? "partial-unverified" : "failed",
                Rollback = ToWireRollback(rollback),
                PreviousAmbition = previous.Ambition,
                Ambition = observed?.Ambition,
                PreviousProfessionalism = previous.Professionalism,
                Professionalism = observed?.Professionalism,
                PreviousDetermination = previous.Determination,
                Determination = observed?.Determination,
            });
    }

    private PlayerValueMutationResult ApplyMentalityChange(
        MentalityField field,
        PersonCandidate player,
        int? previous,
        int? target,
        PlayerValueMutationService mutations,
        ICollection<MentalityMutation> applied)
    {
        if (previous is null || target is null || target == previous)
        {
            return new PlayerValueMutationResult(
                Succeeded: true,
                PlayerValueMutationFailure.None,
                previous,
                target,
                PlayerValueRollback.NotNeeded);
        }

        var result = WriteMentality(field, player, previous.Value, target.Value, mutations);
        if (result.Succeeded)
        {
            applied.Add(new MentalityMutation(field, previous.Value, target.Value));
        }

        return result;
    }

    private static PlayerValueRollback RollbackAppliedMentality(
        PlayerValueMutationService mutations,
        PersonCandidate player,
        IReadOnlyList<MentalityMutation> applied)
    {
        for (var index = applied.Count - 1; index >= 0; index--)
        {
            var mutation = applied[index];
            var restore = WriteMentality(
                mutation.Field,
                player,
                mutation.Target,
                mutation.Previous,
                mutations);
            if (!restore.Succeeded)
            {
                return PlayerValueRollback.Unverified;
            }
        }

        return applied.Count == 0
            ? PlayerValueRollback.NotNeeded
            : PlayerValueRollback.Restored;
    }

    private static PlayerValueRollback CombineRollback(
        PlayerValueRollback first,
        PlayerValueRollback second)
    {
        if (first == PlayerValueRollback.Unverified || second == PlayerValueRollback.Unverified)
        {
            return PlayerValueRollback.Unverified;
        }

        return first == PlayerValueRollback.Restored || second == PlayerValueRollback.Restored
            ? PlayerValueRollback.Restored
            : PlayerValueRollback.NotNeeded;
    }

    private static PlayerValueMutationResult WriteMentality(
        MentalityField field,
        PersonCandidate player,
        int expected,
        int target,
        PlayerValueMutationService mutations) =>
        field switch
        {
            MentalityField.Ambition => mutations.SetAmbition(player, expected, target),
            MentalityField.Professionalism => mutations.SetProfessionalism(player, expected, target),
            MentalityField.Determination => mutations.SetDetermination(player, expected, target),
            _ => throw new ArgumentOutOfRangeException(nameof(field)),
        };

    private PlayerBoostExecutionResult FailedFromMutation(
        string operation,
        PlayerValueMutationResult mutation,
        LiveAbilityState live)
    {
        var failure = mutation.Rollback == PlayerValueRollback.Unverified
            ? PlayerBoostFailure.PartialRollbackUnverified
            : MapMutationFailure(mutation.Failure);
        return new PlayerBoostExecutionResult(
            Succeeded: false,
            failure,
            new PlayerBoostResult
            {
                Operation = operation,
                Outcome = mutation.Rollback == PlayerValueRollback.Unverified ? "partial-unverified" : "failed",
                Rollback = ToWireRollback(mutation.Rollback),
                PreviousCurrentAbility = mutation.PreviousValue,
                PotentialAbility = live.PotentialAbility,
            });
    }

    private int? ChooseMentalityTarget(int? current) =>
        current is <= MentalityEligibilityMaximum
            ? _nextRandom(MentalityTargetMinimum, MentalityTargetMaximumExclusive)
            : current;

    private static PlayerBoostFailure TryReadLiveAbilityState(
        IMemoryReader reader,
        PersonCandidate player,
        IFmMemoryLayout layout,
        int expectedCurrentAbility,
        int expectedPotentialAbility,
        out LiveAbilityState state)
    {
        state = default;
        if (player.Facet != PersonFacet.Player
            || !TryAdd(player.ObjectAddress, layout.ObjectUidOffset, out var uidAddress)
            || !TryAdd(player.BlockAddress, layout.CurrentAbilityOffset, out var currentAbilityAddress)
            || !TryAdd(player.BlockAddress, layout.PotentialAbilityOffset, out var potentialAbilityAddress))
        {
            return PlayerBoostFailure.LiveReadFailed;
        }

        if (!reader.TryReadUInt32(uidAddress, out var liveUid)
            || !reader.TryReadUInt16(currentAbilityAddress, out var currentAbility)
            || !reader.TryReadUInt16(potentialAbilityAddress, out var potentialAbility))
        {
            return PlayerBoostFailure.LiveReadFailed;
        }

        if (liveUid != player.Uid)
        {
            return PlayerBoostFailure.LiveIdentityMismatch;
        }

        if (!IsAbility(currentAbility) || !IsAbility(potentialAbility))
        {
            return PlayerBoostFailure.InvalidLiveValue;
        }

        if (currentAbility != expectedCurrentAbility || potentialAbility != expectedPotentialAbility)
        {
            return PlayerBoostFailure.ExpectedValuesMismatch;
        }

        state = new LiveAbilityState(currentAbility, potentialAbility);
        return PlayerBoostFailure.None;
    }

    private static PlayerBoostFailure TryReadMentality(
        IMemoryReader reader,
        PersonCandidate player,
        IFmMemoryLayout layout,
        MentalityState requested,
        out MentalityState state)
    {
        state = default;
        var ambitionFailure = TryReadPersonality(
            reader,
            player,
            layout,
            "Ambition",
            requested.Ambition is not null,
            out var ambition);
        if (ambitionFailure != PlayerBoostFailure.None)
        {
            return ambitionFailure;
        }

        var professionalismFailure = TryReadPersonality(
            reader,
            player,
            layout,
            "Professionalism",
            requested.Professionalism is not null,
            out var professionalism);
        if (professionalismFailure != PlayerBoostFailure.None)
        {
            return professionalismFailure;
        }

        var determinationFailure = TryReadDetermination(
            reader,
            player,
            layout,
            requested.Determination is not null,
            out var determination);
        if (determinationFailure != PlayerBoostFailure.None)
        {
            return determinationFailure;
        }

        state = new MentalityState(ambition, professionalism, determination);
        return PlayerBoostFailure.None;
    }

    private static PlayerBoostFailure TryReadPersonality(
        IMemoryReader reader,
        PersonCandidate player,
        IFmMemoryLayout layout,
        string key,
        bool required,
        out int? value)
    {
        value = null;
        if (!required)
        {
            return PlayerBoostFailure.None;
        }

        if (!TryFindOffset(layout.PersonalityEntries, key, out var offset)
            || !TryAdd(player.ObjectAddress, offset, out var address))
        {
            return PlayerBoostFailure.LiveReadFailed;
        }

        if (!reader.TryReadByte(address, out var rawValue))
        {
            return PlayerBoostFailure.LiveReadFailed;
        }

        if (!IsMentality(rawValue))
        {
            return PlayerBoostFailure.InvalidLiveValue;
        }

        value = rawValue;
        return PlayerBoostFailure.None;
    }

    private static PlayerBoostFailure TryReadDetermination(
        IMemoryReader reader,
        PersonCandidate player,
        IFmMemoryLayout layout,
        bool required,
        out int? value)
    {
        value = null;
        if (!required)
        {
            return PlayerBoostFailure.None;
        }

        if (!TryFindOffset(layout.AttributeEntries, "Determination", out var offset)
            || !TryAdd(player.BlockAddress, layout.AttrsOffset, out var attributesAddress)
            || !TryAdd(attributesAddress, offset, out var address))
        {
            return PlayerBoostFailure.LiveReadFailed;
        }

        if (!reader.TryReadByte(address, out var rawValue))
        {
            return PlayerBoostFailure.LiveReadFailed;
        }

        value = AttributeScale.TryDecodeScaledStrict(rawValue);
        return value is null
            ? PlayerBoostFailure.InvalidLiveValue
            : PlayerBoostFailure.None;
    }

    private static bool MatchesMentalityExpectations(
        MentalityState actual,
        MentalityState expected) =>
        (expected.Ambition is null || actual.Ambition == expected.Ambition)
        && (expected.Professionalism is null || actual.Professionalism == expected.Professionalism)
        && (expected.Determination is null || actual.Determination == expected.Determination);

    private static bool TryGetMentalityExpectations(
        BridgeRequest request,
        out MentalityState expectations)
    {
        expectations = new MentalityState(
            request.ExpectedAmbition,
            request.ExpectedProfessionalism,
            request.ExpectedDetermination);
        return request.Operation == BridgeProtocol.OperationWonderkidMentality
            && request.CurrentAbilityIncrement is null
            && IsOptionalMentality(expectations.Ambition)
            && IsOptionalMentality(expectations.Professionalism)
            && IsOptionalMentality(expectations.Determination)
            && (IsEligibleMentality(expectations.Ambition)
                || IsEligibleMentality(expectations.Professionalism)
                || IsEligibleMentality(expectations.Determination));
    }

    private static bool TryGetBoostRequest(
        BridgeRequest request,
        out string sourceRequestId,
        out uint playerUid,
        out int expectedCurrentAbility,
        out int expectedPotentialAbility)
    {
        sourceRequestId = request.SourceRequestId ?? "";
        playerUid = request.PlayerUid ?? 0;
        expectedCurrentAbility = request.ExpectedCurrentAbility ?? 0;
        expectedPotentialAbility = request.ExpectedPotentialAbility ?? 0;
        return (request.Operation == BridgeProtocol.OperationBoostCurrentAbility
                || request.Operation == BridgeProtocol.OperationWonderkidMentality)
            && !string.IsNullOrWhiteSpace(sourceRequestId)
            && playerUid != 0
            && IsAbility(expectedCurrentAbility)
            && IsAbility(expectedPotentialAbility)
            && expectedCurrentAbility <= expectedPotentialAbility;
    }

    private static PlayerBoostFailure MapLookupFailure(PlayerMutationLookup lookup) =>
        lookup switch
        {
            PlayerMutationLookup.MissingIndex => PlayerBoostFailure.NoLiveScan,
            PlayerMutationLookup.SourceRequestMismatch => PlayerBoostFailure.SourceRequestMismatch,
            PlayerMutationLookup.PlayerNotFound => PlayerBoostFailure.PlayerNotFound,
            _ => throw new ArgumentOutOfRangeException(nameof(lookup)),
        };

    private static PlayerBoostFailure MapMutationFailure(PlayerValueMutationFailure failure) =>
        failure switch
        {
            PlayerValueMutationFailure.ExpectedValueMismatch => PlayerBoostFailure.ExpectedValuesMismatch,
            PlayerValueMutationFailure.TargetExceedsPotentialAbility => PlayerBoostFailure.CurrentAbilityAtLimit,
            _ => PlayerBoostFailure.MutationFailed,
        };

    private static PlayerBoostExecutionResult Succeeded(PlayerBoostResult result) =>
        new(true, PlayerBoostFailure.None, result);

    private static PlayerBoostExecutionResult Failed(PlayerBoostFailure failure) =>
        new(false, failure, null);

    private static string ToWireRollback(PlayerValueRollback rollback) =>
        rollback switch
        {
            PlayerValueRollback.NotNeeded => "not-needed",
            PlayerValueRollback.Restored => "restored",
            PlayerValueRollback.Unverified => "unverified",
            _ => throw new ArgumentOutOfRangeException(nameof(rollback)),
        };

    private static bool IsAbility(int value) => value is >= MinAbility and <= MaxAbility;

    private static bool IsMentality(int value) => value is >= MinAbility and <= MentalityTargetMaximumExclusive - 1;

    private static bool IsOptionalMentality(int? value) => value is null || IsMentality(value.Value);

    private static bool IsEligibleMentality(int? value) => value is >= MinAbility and <= MentalityEligibilityMaximum;

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

    private static bool TryFindOffset(
        IReadOnlyList<AttributeLayoutEntry> entries,
        string key,
        out int offset)
    {
        foreach (var entry in entries)
        {
            if (entry.Key == key && entry.Offset >= 0)
            {
                offset = entry.Offset;
                return true;
            }
        }

        offset = 0;
        return false;
    }

    private readonly record struct LiveAbilityState(int CurrentAbility, int PotentialAbility);

    private readonly record struct MentalityState(int? Ambition, int? Professionalism, int? Determination);

    private readonly record struct MentalityMutation(MentalityField Field, int Previous, int Target);

    private enum MentalityField
    {
        Ambition,
        Professionalism,
        Determination,
    }
}
