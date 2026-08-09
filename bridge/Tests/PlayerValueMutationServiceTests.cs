using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Mutations;
using FmDataBridge.Scanning;
using FmDataBridge.Tests.Fakes;
using Xunit;

namespace FmDataBridge.Tests;

public sealed class PlayerValueMutationServiceTests
{
    private const ulong PersonAddress = 0x20000;
    private const ulong PlayerBlockAddress = 0x40000;
    private const uint PlayerUid = 1234;

    private static readonly IFmMemoryLayout Layout = Fm263Layout.Instance;

    [Fact]
    public void Memory_writer_exposes_only_typed_scalar_operations()
    {
        var methodNames = typeof(IMemoryWriter)
            .GetMethods()
            .Select(method => method.Name)
            .OrderBy(name => name)
            .ToArray();

        Assert.Equal(new[] { "TryWriteByte", "TryWriteUInt16" }, methodNames);

        var byteWrite = Assert.Single(
            typeof(IMemoryWriter).GetMethods(),
            method => method.Name == "TryWriteByte");
        Assert.Equal(typeof(bool), byteWrite.ReturnType);
        Assert.Equal(
            new[] { typeof(ulong), typeof(byte), typeof(int).MakeByRefType() },
            byteWrite.GetParameters().Select(parameter => parameter.ParameterType));

        var uint16Write = Assert.Single(
            typeof(IMemoryWriter).GetMethods(),
            method => method.Name == "TryWriteUInt16");
        Assert.Equal(typeof(bool), uint16Write.ReturnType);
        Assert.Equal(
            new[] { typeof(ulong), typeof(ushort), typeof(int).MakeByRefType() },
            uint16Write.GetParameters().Select(parameter => parameter.ParameterType));
    }

    [Fact]
    public void Set_current_ability_writes_only_the_current_ability_bytes()
    {
        var (reader, candidate) = CreatePlayerMemory(ca: 120, pa: 150);
        var writer = new RecordingMemoryWriter(reader);
        var service = new PlayerValueMutationService(reader, writer, Layout);

        var result = service.SetCurrentAbility(candidate, expectedCurrentAbility: 120, targetCurrentAbility: 130);

        Assert.True(result.Succeeded);
        Assert.Equal(PlayerValueMutationFailure.None, result.Failure);
        Assert.Equal(PlayerValueRollback.NotNeeded, result.Rollback);
        Assert.Equal(120, result.PreviousValue);
        Assert.Equal(130, result.VerifiedValue);
        AssertWrite(
            writer.Writes,
            PlayerBlockAddress + (ulong)Layout.CurrentAbilityOffset,
            BitConverter.GetBytes((ushort)130));
        Assert.Equal(130, ReadUInt16(reader, PlayerBlockAddress + (ulong)Layout.CurrentAbilityOffset));
        Assert.Equal(150, ReadUInt16(reader, PlayerBlockAddress + (ulong)Layout.PotentialAbilityOffset));
        Assert.Equal(0xA5, ReadByte(reader, PlayerBlockAddress + (ulong)Layout.CurrentAbilityOffset - 1));
        Assert.Equal(0x5A, ReadByte(reader, PlayerBlockAddress + (ulong)Layout.PotentialAbilityOffset + sizeof(ushort)));
    }

    [Fact]
    public void Set_ambition_writes_the_raw_personality_byte()
    {
        var (reader, candidate) = CreatePlayerMemory(ambition: 8);
        var writer = new RecordingMemoryWriter(reader);
        var service = new PlayerValueMutationService(reader, writer, Layout);

        var result = service.SetAmbition(candidate, expectedAmbition: 8, targetAmbition: 15);

        Assert.True(result.Succeeded);
        Assert.Equal(8, result.PreviousValue);
        Assert.Equal(15, result.VerifiedValue);
        AssertWrite(writer.Writes, PersonAddress + (ulong)PersonalityOffset("Ambition"), new byte[] { 15 });
        Assert.Equal(15, ReadByte(reader, PersonAddress + (ulong)PersonalityOffset("Ambition")));
        Assert.Equal(10, ReadByte(reader, PersonAddress + (ulong)PersonalityOffset("Professionalism")));
    }

    [Fact]
    public void Set_professionalism_writes_the_raw_personality_byte()
    {
        var (reader, candidate) = CreatePlayerMemory(professionalism: 9);
        var writer = new RecordingMemoryWriter(reader);
        var service = new PlayerValueMutationService(reader, writer, Layout);

        var result = service.SetProfessionalism(candidate, expectedProfessionalism: 9, targetProfessionalism: 16);

        Assert.True(result.Succeeded);
        Assert.Equal(9, result.PreviousValue);
        Assert.Equal(16, result.VerifiedValue);
        AssertWrite(writer.Writes, PersonAddress + (ulong)PersonalityOffset("Professionalism"), new byte[] { 16 });
        Assert.Equal(16, ReadByte(reader, PersonAddress + (ulong)PersonalityOffset("Professionalism")));
        Assert.Equal(10, ReadByte(reader, PersonAddress + (ulong)PersonalityOffset("Ambition")));
    }

    [Fact]
    public void Set_determination_encodes_the_target_as_value_times_five()
    {
        var (reader, candidate) = CreatePlayerMemory(determination: 10);
        var writer = new RecordingMemoryWriter(reader);
        var service = new PlayerValueMutationService(reader, writer, Layout);

        var result = service.SetDetermination(candidate, expectedDetermination: 10, targetDetermination: 17);

        Assert.True(result.Succeeded);
        Assert.Equal(10, result.PreviousValue);
        Assert.Equal(17, result.VerifiedValue);
        AssertWrite(
            writer.Writes,
            PlayerBlockAddress + (ulong)Layout.AttrsOffset + (ulong)AttributeOffset("Determination"),
            new byte[] { 85 });
        Assert.Equal(
            85,
            ReadByte(reader, PlayerBlockAddress + (ulong)Layout.AttrsOffset + (ulong)AttributeOffset("Determination")));
    }

    [Fact]
    public void Set_current_ability_rejects_a_target_above_live_potential_before_writing()
    {
        var (reader, candidate) = CreatePlayerMemory(ca: 120, pa: 125);
        var writer = new RecordingMemoryWriter(reader);
        var service = new PlayerValueMutationService(reader, writer, Layout);

        var result = service.SetCurrentAbility(candidate, expectedCurrentAbility: 120, targetCurrentAbility: 126);

        Assert.False(result.Succeeded);
        Assert.Equal(PlayerValueMutationFailure.TargetExceedsPotentialAbility, result.Failure);
        Assert.Equal(PlayerValueRollback.NotNeeded, result.Rollback);
        Assert.Empty(writer.Writes);
        Assert.Equal(120, ReadUInt16(reader, PlayerBlockAddress + (ulong)Layout.CurrentAbilityOffset));
        Assert.Equal(125, ReadUInt16(reader, PlayerBlockAddress + (ulong)Layout.PotentialAbilityOffset));
    }

    [Theory]
    [InlineData(0)]
    [InlineData(201)]
    public void Set_current_ability_rejects_values_outside_the_supported_range(int target)
    {
        var (reader, candidate) = CreatePlayerMemory();
        var writer = new RecordingMemoryWriter(reader);
        var service = new PlayerValueMutationService(reader, writer, Layout);

        var result = service.SetCurrentAbility(candidate, expectedCurrentAbility: 120, targetCurrentAbility: target);

        Assert.False(result.Succeeded);
        Assert.Equal(PlayerValueMutationFailure.InvalidValue, result.Failure);
        Assert.Empty(writer.Writes);
        Assert.Equal(120, ReadUInt16(reader, PlayerBlockAddress + (ulong)Layout.CurrentAbilityOffset));
    }

    [Theory]
    [InlineData(0)]
    [InlineData(21)]
    public void Set_ambition_rejects_values_outside_the_personality_range(int target)
    {
        var (reader, candidate) = CreatePlayerMemory();
        var writer = new RecordingMemoryWriter(reader);
        var service = new PlayerValueMutationService(reader, writer, Layout);

        var result = service.SetAmbition(candidate, expectedAmbition: 10, targetAmbition: target);

        Assert.False(result.Succeeded);
        Assert.Equal(PlayerValueMutationFailure.InvalidValue, result.Failure);
        Assert.Empty(writer.Writes);
        Assert.Equal(10, ReadByte(reader, PersonAddress + (ulong)PersonalityOffset("Ambition")));
    }

    [Fact]
    public void Set_current_ability_rejects_a_stale_expected_value_before_writing()
    {
        var (reader, candidate) = CreatePlayerMemory(ca: 120);
        var writer = new RecordingMemoryWriter(reader);
        var service = new PlayerValueMutationService(reader, writer, Layout);

        var result = service.SetCurrentAbility(candidate, expectedCurrentAbility: 119, targetCurrentAbility: 130);

        Assert.False(result.Succeeded);
        Assert.Equal(PlayerValueMutationFailure.ExpectedValueMismatch, result.Failure);
        Assert.Empty(writer.Writes);
        Assert.Equal(120, ReadUInt16(reader, PlayerBlockAddress + (ulong)Layout.CurrentAbilityOffset));
    }

    [Fact]
    public void Set_current_ability_restores_the_original_value_after_a_readback_mismatch()
    {
        var (reader, candidate) = CreatePlayerMemory(ca: 120, pa: 150);
        var service = new PlayerValueMutationService(reader, new CorruptFirstWriteMemoryWriter(reader), Layout);

        var result = service.SetCurrentAbility(candidate, expectedCurrentAbility: 120, targetCurrentAbility: 130);

        Assert.False(result.Succeeded);
        Assert.Equal(PlayerValueMutationFailure.ReadbackMismatch, result.Failure);
        Assert.Equal(PlayerValueRollback.Restored, result.Rollback);
        Assert.Equal(120, ReadUInt16(reader, PlayerBlockAddress + (ulong)Layout.CurrentAbilityOffset));
        Assert.Equal(150, ReadUInt16(reader, PlayerBlockAddress + (ulong)Layout.PotentialAbilityOffset));
    }

    [Fact]
    public void Set_current_ability_restores_the_original_value_after_a_partial_write()
    {
        var (reader, candidate) = CreatePlayerMemory(ca: 120, pa: 150);
        var service = new PlayerValueMutationService(reader, new PartialFirstWriteMemoryWriter(reader), Layout);

        var result = service.SetCurrentAbility(candidate, expectedCurrentAbility: 120, targetCurrentAbility: 130);

        Assert.False(result.Succeeded);
        Assert.Equal(PlayerValueMutationFailure.WriteFailed, result.Failure);
        Assert.Equal(PlayerValueRollback.Restored, result.Rollback);
        Assert.Equal(120, ReadUInt16(reader, PlayerBlockAddress + (ulong)Layout.CurrentAbilityOffset));
        Assert.Equal(150, ReadUInt16(reader, PlayerBlockAddress + (ulong)Layout.PotentialAbilityOffset));
    }

    [Fact]
    public void Set_current_ability_reports_an_unverified_rollback_when_a_partial_write_cannot_be_restored()
    {
        var (reader, candidate) = CreatePlayerMemory(ca: 120, pa: 150);
        var service = new PlayerValueMutationService(reader, new PartialThenFailMemoryWriter(reader), Layout);

        var result = service.SetCurrentAbility(candidate, expectedCurrentAbility: 120, targetCurrentAbility: 130);

        Assert.False(result.Succeeded);
        Assert.Equal(PlayerValueMutationFailure.WriteFailed, result.Failure);
        Assert.Equal(PlayerValueRollback.Unverified, result.Rollback);
        Assert.Equal(130, ReadUInt16(reader, PlayerBlockAddress + (ulong)Layout.CurrentAbilityOffset));
        Assert.Equal(150, ReadUInt16(reader, PlayerBlockAddress + (ulong)Layout.PotentialAbilityOffset));
    }

    private static (FakeMemoryReader Reader, PersonCandidate Candidate) CreatePlayerMemory(
        int ca = 120,
        int pa = 150,
        int ambition = 10,
        int professionalism = 10,
        int determination = 10)
    {
        var reader = new FakeMemoryReader();
        var personBytes = new byte[PersonalityOffset("Professionalism") + 2];
        personBytes.AsSpan().Fill(0xA5);
        BitConverter.TryWriteBytes(personBytes.AsSpan(Layout.ObjectUidOffset), PlayerUid);
        personBytes[PersonalityOffset("Ambition")] = (byte)ambition;
        personBytes[PersonalityOffset("Professionalism")] = (byte)professionalism;
        reader.AddBytes(PersonAddress, personBytes);

        var length = Math.Max(
            Layout.PotentialAbilityOffset + sizeof(ushort) + 1,
            Layout.AttrsOffset + AttributeOffset("Determination") + 2);
        var playerBytes = new byte[length];
        playerBytes.AsSpan().Fill(0xA5);
        BitConverter.TryWriteBytes(playerBytes.AsSpan(Layout.CurrentAbilityOffset), (ushort)ca);
        BitConverter.TryWriteBytes(playerBytes.AsSpan(Layout.PotentialAbilityOffset), (ushort)pa);
        playerBytes[Layout.PotentialAbilityOffset + sizeof(ushort)] = 0x5A;
        playerBytes[Layout.AttrsOffset + AttributeOffset("Determination")] = (byte)(determination * 5);
        reader.AddBytes(PlayerBlockAddress, playerBytes);

        return (
            reader,
            new PersonCandidate(
                PersonAddress,
                PlayerBlockAddress,
                PlayerUid,
                ca,
                pa,
                ClassOffset: 0x288,
                PersonFacet.Player));
    }

    private static int PersonalityOffset(string key) =>
        Layout.PersonalityEntries.Single(entry => entry.Key == key).Offset;

    private static int AttributeOffset(string key) =>
        Layout.AttributeEntries.Single(entry => entry.Key == key).Offset;

    private static byte ReadByte(FakeMemoryReader reader, ulong address)
    {
        Assert.True(reader.TryReadByte(address, out var value));
        return value;
    }

    private static ushort ReadUInt16(FakeMemoryReader reader, ulong address)
    {
        Assert.True(reader.TryReadUInt16(address, out var value));
        return value;
    }

    private static void AssertWrite(
        IReadOnlyList<RecordedWrite> writes,
        ulong expectedAddress,
        byte[] expectedBytes)
    {
        var write = Assert.Single(writes);
        Assert.Equal(expectedAddress, write.Address);
        Assert.Equal(expectedBytes, write.Bytes);
    }

    private sealed class RecordingMemoryWriter : IMemoryWriter
    {
        private readonly FakeMemoryReader _inner;

        public RecordingMemoryWriter(FakeMemoryReader inner)
        {
            _inner = inner;
        }

        public List<RecordedWrite> Writes { get; } = new();

        public bool TryWriteByte(ulong address, byte value, out int bytesWritten)
        {
            Writes.Add(new RecordedWrite(address, new[] { value }));
            return _inner.TryWriteByte(address, value, out bytesWritten);
        }

        public bool TryWriteUInt16(ulong address, ushort value, out int bytesWritten)
        {
            Writes.Add(new RecordedWrite(address, BitConverter.GetBytes(value)));
            return _inner.TryWriteUInt16(address, value, out bytesWritten);
        }
    }

    private sealed class CorruptFirstWriteMemoryWriter : IMemoryWriter
    {
        private readonly FakeMemoryReader _inner;
        private bool _firstWrite = true;

        public CorruptFirstWriteMemoryWriter(FakeMemoryReader inner)
        {
            _inner = inner;
        }

        public bool TryWriteByte(ulong address, byte value, out int bytesWritten)
        {
            if (_firstWrite)
            {
                _firstWrite = false;
                return _inner.TryWriteByte(address, unchecked((byte)(value + 1)), out bytesWritten);
            }

            return _inner.TryWriteByte(address, value, out bytesWritten);
        }

        public bool TryWriteUInt16(ulong address, ushort value, out int bytesWritten)
        {
            if (_firstWrite)
            {
                _firstWrite = false;
                return _inner.TryWriteUInt16(address, unchecked((ushort)(value + 1)), out bytesWritten);
            }

            return _inner.TryWriteUInt16(address, value, out bytesWritten);
        }
    }

    private sealed class PartialFirstWriteMemoryWriter : IMemoryWriter
    {
        private readonly FakeMemoryReader _inner;
        private bool _firstWrite = true;

        public PartialFirstWriteMemoryWriter(FakeMemoryReader inner)
        {
            _inner = inner;
        }

        public bool TryWriteByte(ulong address, byte value, out int bytesWritten)
        {
            if (_firstWrite)
            {
                _firstWrite = false;
                bytesWritten = 0;
                return false;
            }

            return _inner.TryWriteByte(address, value, out bytesWritten);
        }

        public bool TryWriteUInt16(ulong address, ushort value, out int bytesWritten)
        {
            if (_firstWrite)
            {
                _firstWrite = false;
                _inner.TryWriteByte(address, (byte)value, out bytesWritten);
                return false;
            }

            return _inner.TryWriteUInt16(address, value, out bytesWritten);
        }
    }

    private sealed class PartialThenFailMemoryWriter : IMemoryWriter
    {
        private readonly FakeMemoryReader _inner;
        private bool _firstWrite = true;

        public PartialThenFailMemoryWriter(FakeMemoryReader inner)
        {
            _inner = inner;
        }

        public bool TryWriteByte(ulong address, byte value, out int bytesWritten)
        {
            if (_firstWrite)
            {
                _firstWrite = false;
                bytesWritten = 0;
                return false;
            }

            bytesWritten = 0;
            return false;
        }

        public bool TryWriteUInt16(ulong address, ushort value, out int bytesWritten)
        {
            if (_firstWrite)
            {
                _firstWrite = false;
                _inner.TryWriteByte(address, (byte)value, out bytesWritten);
                return false;
            }

            bytesWritten = 0;
            return false;
        }
    }

    private sealed record RecordedWrite(ulong Address, byte[] Bytes);
}
