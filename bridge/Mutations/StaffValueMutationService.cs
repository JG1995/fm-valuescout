using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Scanning;

namespace FmDataBridge.Mutations;

internal enum StaffValueMutationFailure
{
    None,
    InvalidCandidate,
    InvalidValue,
    InvalidLiveValue,
    AddressOverflow,
    ReadFailed,
    ExpectedValueMismatch,
    TargetExceedsPotentialAbility,
    WriteFailed,
    ReadbackFailed,
    ReadbackMismatch,
}

internal enum StaffValueRollback
{
    NotNeeded,
    Restored,
    Unverified,
}

internal readonly record struct StaffValueMutationResult(
    bool Succeeded,
    StaffValueMutationFailure Failure,
    int? PreviousValue,
    int? VerifiedValue,
    StaffValueRollback Rollback);

/// <summary>
/// Applies only the approved staff CA scalar mutation through staff-specific offsets.
/// </summary>
internal sealed class StaffValueMutationService
{
    private readonly IMemoryReader _reader;
    private readonly IMemoryWriter _writer;
    private readonly IFmMemoryLayout _layout;

    public StaffValueMutationService(
        IMemoryReader reader,
        IMemoryWriter writer,
        IFmMemoryLayout layout)
    {
        _reader = reader ?? throw new ArgumentNullException(nameof(reader));
        _writer = writer ?? throw new ArgumentNullException(nameof(writer));
        _layout = layout ?? throw new ArgumentNullException(nameof(layout));
    }

    public StaffValueMutationResult SetCurrentAbility(
        PersonCandidate staff,
        int expectedCurrentAbility,
        int expectedPotentialAbility,
        int targetCurrentAbility)
    {
        if (staff.Facet is not PersonFacet.Staff and not PersonFacet.HumanManager)
        {
            return Failed(StaffValueMutationFailure.InvalidCandidate);
        }

        if (!PersonScanner.IsValidAbility(expectedCurrentAbility)
            || !PersonScanner.IsValidAbility(expectedPotentialAbility)
            || !PersonScanner.IsValidAbility(targetCurrentAbility))
        {
            return Failed(StaffValueMutationFailure.InvalidValue);
        }

        if (!TryAdd(staff.BlockAddress, _layout.StaffCurrentAbilityOffset, out var caAddress)
            || !TryAdd(staff.BlockAddress, _layout.StaffPotentialAbilityOffset, out var paAddress))
        {
            return Failed(StaffValueMutationFailure.AddressOverflow);
        }

        if (!_reader.TryReadUInt16(caAddress, out var liveCa)
            || !_reader.TryReadUInt16(paAddress, out var livePa))
        {
            return Failed(StaffValueMutationFailure.ReadFailed);
        }

        if (!PersonScanner.IsValidAbility(liveCa) || !PersonScanner.IsValidAbility(livePa))
        {
            return Failed(StaffValueMutationFailure.InvalidLiveValue);
        }

        if (liveCa != expectedCurrentAbility || livePa != expectedPotentialAbility)
        {
            return Failed(StaffValueMutationFailure.ExpectedValueMismatch, liveCa);
        }

        if (targetCurrentAbility > livePa)
        {
            return Failed(StaffValueMutationFailure.TargetExceedsPotentialAbility, liveCa);
        }

        var previousBytes = BitConverter.GetBytes(liveCa);
        var targetBytes = BitConverter.GetBytes((ushort)targetCurrentAbility);
        var completeWrite = _writer.TryWriteUInt16(caAddress, (ushort)targetCurrentAbility, out var bytesWritten);
        if (!completeWrite || bytesWritten != sizeof(ushort))
        {
            return Recover(caAddress, previousBytes, liveCa, StaffValueMutationFailure.WriteFailed);
        }

        var readback = new byte[sizeof(ushort)];
        if (!_reader.TryRead(caAddress, readback, out var readbackBytes)
            || readbackBytes != sizeof(ushort))
        {
            return Recover(caAddress, previousBytes, liveCa, StaffValueMutationFailure.ReadbackFailed);
        }

        if (!readback.AsSpan().SequenceEqual(targetBytes))
        {
            return Recover(caAddress, previousBytes, liveCa, StaffValueMutationFailure.ReadbackMismatch);
        }

        return new StaffValueMutationResult(
            true,
            StaffValueMutationFailure.None,
            liveCa,
            targetCurrentAbility,
            StaffValueRollback.NotNeeded);
    }

    private StaffValueMutationResult Recover(
        ulong address,
        byte[] previousBytes,
        int previousValue,
        StaffValueMutationFailure failure)
    {
        if (Matches(address, previousBytes))
        {
            return Failed(failure, previousValue);
        }

        _ = _writer.TryWriteUInt16(address, BitConverter.ToUInt16(previousBytes, 0), out _);
        return Matches(address, previousBytes)
            ? Failed(failure, previousValue, StaffValueRollback.Restored)
            : Failed(failure, previousValue, StaffValueRollback.Unverified);
    }

    private bool Matches(ulong address, byte[] expected)
    {
        var current = new byte[expected.Length];
        return _reader.TryRead(address, current, out var bytesRead)
               && bytesRead == expected.Length
               && current.AsSpan().SequenceEqual(expected);
    }

    private static StaffValueMutationResult Failed(
        StaffValueMutationFailure failure,
        int? previousValue = null,
        StaffValueRollback rollback = StaffValueRollback.NotNeeded) =>
        new(false, failure, previousValue, null, rollback);

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
