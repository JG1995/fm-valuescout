using FmDataBridge.Extraction;
using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Scanning;

namespace FmDataBridge.Mutations;

internal enum PlayerValueMutationFailure
{
    None,
    InvalidCandidate,
    InvalidValue,
    InvalidLiveValue,
    LayoutInvalid,
    AddressOverflow,
    ReadFailed,
    ExpectedValueMismatch,
    TargetExceedsPotentialAbility,
    WriteFailed,
    ReadbackFailed,
    ReadbackMismatch,
}

internal enum PlayerValueRollback
{
    NotNeeded,
    Restored,
    Unverified,
}

internal readonly record struct PlayerValueMutationResult(
    bool Succeeded,
    PlayerValueMutationFailure Failure,
    int? PreviousValue,
    int? VerifiedValue,
    PlayerValueRollback Rollback);

/// <summary>
/// Applies only the four approved player scalar mutations after re-reading their live values.
/// This service is intentionally internal and has no protocol or arbitrary-address entry point.
/// </summary>
internal sealed class PlayerValueMutationService
{
    private const string AmbitionKey = "Ambition";
    private const string ProfessionalismKey = "Professionalism";
    private const string DeterminationKey = "Determination";
    private const int MinPersonalityValue = 1;
    private const int MaxPersonalityValue = 20;

    private readonly IMemoryReader _reader;
    private readonly IMemoryWriter _writer;
    private readonly IFmMemoryLayout _layout;

    public PlayerValueMutationService(
        IMemoryReader reader,
        IMemoryWriter writer,
        IFmMemoryLayout layout)
    {
        _reader = reader ?? throw new ArgumentNullException(nameof(reader));
        _writer = writer ?? throw new ArgumentNullException(nameof(writer));
        _layout = layout ?? throw new ArgumentNullException(nameof(layout));
    }

    public PlayerValueMutationResult SetCurrentAbility(
        PersonCandidate player,
        int expectedCurrentAbility,
        int targetCurrentAbility)
        => SetCurrentAbility(player, expectedCurrentAbility, expectedPotentialAbility: null, targetCurrentAbility);

    public PlayerValueMutationResult SetCurrentAbility(
        PersonCandidate player,
        int expectedCurrentAbility,
        int? expectedPotentialAbility,
        int targetCurrentAbility)
    {
        if (!IsPlayer(player))
        {
            return Failed(PlayerValueMutationFailure.InvalidCandidate);
        }

        if (!PersonScanner.IsValidAbility(expectedCurrentAbility)
            || (expectedPotentialAbility is not null
                && !PersonScanner.IsValidAbility(expectedPotentialAbility.Value))
            || !PersonScanner.IsValidAbility(targetCurrentAbility))
        {
            return Failed(PlayerValueMutationFailure.InvalidValue);
        }

        if (!TryAdd(player.BlockAddress, _layout.CurrentAbilityOffset, out var currentAbilityAddress)
            || !TryAdd(player.BlockAddress, _layout.PotentialAbilityOffset, out var potentialAbilityAddress))
        {
            return Failed(PlayerValueMutationFailure.AddressOverflow);
        }

        if (!_reader.TryReadUInt16(currentAbilityAddress, out var liveCurrentAbility)
            || !_reader.TryReadUInt16(potentialAbilityAddress, out var livePotentialAbility))
        {
            return Failed(PlayerValueMutationFailure.ReadFailed);
        }

        if (!PersonScanner.IsValidAbility(liveCurrentAbility)
            || !PersonScanner.IsValidAbility(livePotentialAbility))
        {
            return Failed(PlayerValueMutationFailure.InvalidLiveValue);
        }

        if (liveCurrentAbility != expectedCurrentAbility)
        {
            return Failed(PlayerValueMutationFailure.ExpectedValueMismatch, liveCurrentAbility);
        }

        if (expectedPotentialAbility is not null
            && livePotentialAbility != expectedPotentialAbility.Value)
        {
            return Failed(PlayerValueMutationFailure.ExpectedValueMismatch, liveCurrentAbility);
        }

        if (targetCurrentAbility > livePotentialAbility)
        {
            return Failed(PlayerValueMutationFailure.TargetExceedsPotentialAbility, liveCurrentAbility);
        }

        return WriteAndVerify(
            currentAbilityAddress,
            BitConverter.GetBytes(liveCurrentAbility),
            BitConverter.GetBytes((ushort)targetCurrentAbility),
            liveCurrentAbility,
            targetCurrentAbility);
    }

    public PlayerValueMutationResult SetAmbition(
        PersonCandidate player,
        int expectedAmbition,
        int targetAmbition) =>
        SetPersonality(player, AmbitionKey, expectedAmbition, targetAmbition);

    public PlayerValueMutationResult SetProfessionalism(
        PersonCandidate player,
        int expectedProfessionalism,
        int targetProfessionalism) =>
        SetPersonality(player, ProfessionalismKey, expectedProfessionalism, targetProfessionalism);

    public PlayerValueMutationResult SetDetermination(
        PersonCandidate player,
        int expectedDetermination,
        int targetDetermination)
    {
        if (!IsPlayer(player))
        {
            return Failed(PlayerValueMutationFailure.InvalidCandidate);
        }

        if (!IsPersonalityValue(expectedDetermination) || !IsPersonalityValue(targetDetermination))
        {
            return Failed(PlayerValueMutationFailure.InvalidValue);
        }

        if (!TryFindOffset(_layout.AttributeEntries, DeterminationKey, out var determinationOffset))
        {
            return Failed(PlayerValueMutationFailure.LayoutInvalid);
        }

        if (!TryAdd(player.BlockAddress, _layout.AttrsOffset, out var attributesAddress)
            || !TryAdd(attributesAddress, determinationOffset, out var determinationAddress))
        {
            return Failed(PlayerValueMutationFailure.AddressOverflow);
        }

        if (!_reader.TryReadByte(determinationAddress, out var liveRawDetermination))
        {
            return Failed(PlayerValueMutationFailure.ReadFailed);
        }

        var liveDetermination = AttributeScale.TryDecodeScaledStrict(liveRawDetermination);
        if (liveDetermination is null)
        {
            return Failed(PlayerValueMutationFailure.InvalidLiveValue);
        }

        if (liveDetermination != expectedDetermination)
        {
            return Failed(PlayerValueMutationFailure.ExpectedValueMismatch, liveDetermination);
        }

        return WriteAndVerify(
            determinationAddress,
            new[] { liveRawDetermination },
            new[] { checked((byte)(targetDetermination * 5)) },
            liveDetermination.Value,
            targetDetermination);
    }

    private PlayerValueMutationResult SetPersonality(
        PersonCandidate player,
        string personalityKey,
        int expectedValue,
        int targetValue)
    {
        if (!IsPlayer(player))
        {
            return Failed(PlayerValueMutationFailure.InvalidCandidate);
        }

        if (!IsPersonalityValue(expectedValue) || !IsPersonalityValue(targetValue))
        {
            return Failed(PlayerValueMutationFailure.InvalidValue);
        }

        if (!TryFindOffset(_layout.PersonalityEntries, personalityKey, out var personalityOffset))
        {
            return Failed(PlayerValueMutationFailure.LayoutInvalid);
        }

        if (!TryAdd(player.ObjectAddress, personalityOffset, out var personalityAddress))
        {
            return Failed(PlayerValueMutationFailure.AddressOverflow);
        }

        if (!_reader.TryReadByte(personalityAddress, out var liveValue))
        {
            return Failed(PlayerValueMutationFailure.ReadFailed);
        }

        if (!IsPersonalityValue(liveValue))
        {
            return Failed(PlayerValueMutationFailure.InvalidLiveValue);
        }

        if (liveValue != expectedValue)
        {
            return Failed(PlayerValueMutationFailure.ExpectedValueMismatch, liveValue);
        }

        return WriteAndVerify(
            personalityAddress,
            new[] { liveValue },
            new[] { (byte)targetValue },
            liveValue,
            targetValue);
    }

    private PlayerValueMutationResult WriteAndVerify(
        ulong address,
        byte[] previousBytes,
        byte[] targetBytes,
        int previousValue,
        int targetValue)
    {
        var completeWrite = TryWriteScalar(address, targetBytes, out var bytesWritten);
        if (!completeWrite || bytesWritten != targetBytes.Length)
        {
            return Recover(address, previousBytes, previousValue, PlayerValueMutationFailure.WriteFailed);
        }

        var readback = new byte[targetBytes.Length];
        if (!_reader.TryRead(address, readback, out var readbackBytes)
            || readbackBytes != targetBytes.Length)
        {
            return Recover(address, previousBytes, previousValue, PlayerValueMutationFailure.ReadbackFailed);
        }

        if (!readback.AsSpan().SequenceEqual(targetBytes))
        {
            return Recover(address, previousBytes, previousValue, PlayerValueMutationFailure.ReadbackMismatch);
        }

        return new PlayerValueMutationResult(
            Succeeded: true,
            PlayerValueMutationFailure.None,
            previousValue,
            targetValue,
            PlayerValueRollback.NotNeeded);
    }

    private PlayerValueMutationResult Recover(
        ulong address,
        byte[] previousBytes,
        int previousValue,
        PlayerValueMutationFailure failure)
    {
        if (Matches(address, previousBytes))
        {
            return Failed(failure, previousValue);
        }

        _ = TryWriteScalar(address, previousBytes, out _);
        return Matches(address, previousBytes)
            ? Failed(failure, previousValue, PlayerValueRollback.Restored)
            : Failed(failure, previousValue, PlayerValueRollback.Unverified);
    }

    private bool TryWriteScalar(ulong address, byte[] bytes, out int bytesWritten)
    {
        switch (bytes.Length)
        {
            case sizeof(byte):
                return _writer.TryWriteByte(address, bytes[0], out bytesWritten);
            case sizeof(ushort):
                return _writer.TryWriteUInt16(address, BitConverter.ToUInt16(bytes, 0), out bytesWritten);
            default:
                throw new InvalidOperationException("Player mutation writes must be one or two bytes.");
        }
    }

    private bool Matches(ulong address, byte[] expectedBytes)
    {
        var currentBytes = new byte[expectedBytes.Length];
        return _reader.TryRead(address, currentBytes, out var bytesRead)
               && bytesRead == expectedBytes.Length
               && currentBytes.AsSpan().SequenceEqual(expectedBytes);
    }

    private static PlayerValueMutationResult Failed(
        PlayerValueMutationFailure failure,
        int? previousValue = null,
        PlayerValueRollback rollback = PlayerValueRollback.NotNeeded) =>
        new(
            Succeeded: false,
            failure,
            previousValue,
            VerifiedValue: null,
            rollback);

    private static bool IsPlayer(PersonCandidate player) => player.Facet == PersonFacet.Player;

    private static bool IsPersonalityValue(int value) => value is >= MinPersonalityValue and <= MaxPersonalityValue;

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
}
