using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Mutations;
using FmDataBridge.Protocol;
using FmDataBridge.Scanning;
using FmDataBridge.Tests.Fakes;
using Xunit;

namespace FmDataBridge.Tests;

public sealed class PlayerBoostOperationServiceTests
{
    private const ulong PersonAddress = 0x20000;
    private const ulong PlayerBlockAddress = 0x40000;
    private const uint PlayerUid = 1234;
    private const string SourceRequestId = "R1";
    private const string SupportedGameVersion = "26.3.2";

    private static readonly IFmMemoryLayout Layout = Fm263Layout.Instance;

    [Fact]
    public void Boost_current_ability_caps_to_live_potential_and_reports_verified_values()
    {
        var (reader, candidate) = CreatePlayerMemory(ca: 120, pa: 123);
        var service = CreateService(candidate);

        var result = service.Execute(
            CurrentAbilityRequest(expectedCa: 120, expectedPa: 123, increment: 5),
            SupportedGameVersion,
            reader,
            reader);

        Assert.True(result.Succeeded);
        Assert.Equal(PlayerBoostFailure.None, result.Failure);
        Assert.NotNull(result.BoostResult);
        Assert.Equal(120, result.BoostResult!.PreviousCurrentAbility);
        Assert.Equal(123, result.BoostResult.CurrentAbility);
        Assert.Equal(123, result.BoostResult.PotentialAbility);
        Assert.Equal(123, ReadUInt16(reader, PlayerBlockAddress + (ulong)Layout.CurrentAbilityOffset));
        Assert.Equal(123, ReadUInt16(reader, PlayerBlockAddress + (ulong)Layout.PotentialAbilityOffset));
    }

    [Fact]
    public void Boost_current_ability_rejects_a_stale_source_request_without_writing()
    {
        var (reader, candidate) = CreatePlayerMemory(ca: 120, pa: 150);
        var service = CreateService(candidate);
        var request = CurrentAbilityRequest(
            expectedCa: 120,
            expectedPa: 150,
            increment: 5,
            sourceRequestId: "R2");

        var result = service.Execute(request, SupportedGameVersion, reader, reader);

        Assert.False(result.Succeeded);
        Assert.Equal(PlayerBoostFailure.SourceRequestMismatch, result.Failure);
        Assert.Equal(120, ReadUInt16(reader, PlayerBlockAddress + (ulong)Layout.CurrentAbilityOffset));
        Assert.Equal(150, ReadUInt16(reader, PlayerBlockAddress + (ulong)Layout.PotentialAbilityOffset));
    }

    [Fact]
    public void Boost_current_ability_rejects_stale_expected_values_without_writing()
    {
        var (reader, candidate) = CreatePlayerMemory(ca: 121, pa: 150);
        var service = CreateService(candidate with { Ca = 120 });

        var result = service.Execute(
            CurrentAbilityRequest(expectedCa: 120, expectedPa: 150, increment: 5),
            SupportedGameVersion,
            reader,
            reader);

        Assert.False(result.Succeeded);
        Assert.Equal(PlayerBoostFailure.ExpectedValuesMismatch, result.Failure);
        Assert.Equal(121, ReadUInt16(reader, PlayerBlockAddress + (ulong)Layout.CurrentAbilityOffset));
        Assert.Equal(150, ReadUInt16(reader, PlayerBlockAddress + (ulong)Layout.PotentialAbilityOffset));
    }

    [Fact]
    public void Boost_current_ability_rejects_a_changed_live_potential_without_writing()
    {
        var (reader, candidate) = CreatePlayerMemory(ca: 120, pa: 151);
        var service = CreateService(candidate with { Pa = 150 });

        var result = service.Execute(
            CurrentAbilityRequest(expectedCa: 120, expectedPa: 150, increment: 5),
            SupportedGameVersion,
            reader,
            reader);

        Assert.False(result.Succeeded);
        Assert.Equal(PlayerBoostFailure.ExpectedValuesMismatch, result.Failure);
        Assert.Equal(120, ReadUInt16(reader, PlayerBlockAddress + (ulong)Layout.CurrentAbilityOffset));
        Assert.Equal(151, ReadUInt16(reader, PlayerBlockAddress + (ulong)Layout.PotentialAbilityOffset));
    }

    [Fact]
    public void Boost_current_ability_rejects_an_unsupported_exact_game_build_without_writing()
    {
        var (reader, candidate) = CreatePlayerMemory(ca: 120, pa: 150);
        var service = CreateService(candidate);

        var result = service.Execute(
            CurrentAbilityRequest(expectedCa: 120, expectedPa: 150, increment: 5),
            gameVersion: "26.3.1",
            reader,
            reader);

        Assert.False(result.Succeeded);
        Assert.Equal(PlayerBoostFailure.UnsupportedGameBuild, result.Failure);
        Assert.Equal(120, ReadUInt16(reader, PlayerBlockAddress + (ulong)Layout.CurrentAbilityOffset));
    }

    [Fact]
    public void Wonderkid_mentality_rerolls_each_eligible_value_with_deterministic_inclusive_targets()
    {
        var (reader, candidate) = CreatePlayerMemory(ambition: 10, professionalism: 11, determination: 9);
        var targets = new Queue<int>(new[] { 20, 11 });
        var randomCalls = new List<(int Minimum, int MaximumExclusive)>();
        var service = CreateService(candidate, (minimum, maximumExclusive) =>
        {
            randomCalls.Add((minimum, maximumExclusive));
            return targets.Dequeue();
        });

        var result = service.Execute(
            MentalityRequest(expectedAmbition: 10, expectedProfessionalism: 11, expectedDetermination: 9),
            SupportedGameVersion,
            reader,
            reader);

        Assert.True(result.Succeeded);
        Assert.Equal(PlayerBoostFailure.None, result.Failure);
        Assert.NotNull(result.BoostResult);
        Assert.Equal(20, result.BoostResult!.Ambition);
        Assert.Equal(11, result.BoostResult.Professionalism);
        Assert.Equal(11, result.BoostResult.Determination);
        Assert.Empty(targets);
        Assert.All(
            randomCalls,
            call => Assert.Equal((11, 21), (call.Minimum, call.MaximumExclusive)));
        Assert.Equal(20, ReadByte(reader, PersonAddress + (ulong)PersonalityOffset("Ambition")));
        Assert.Equal(11, ReadByte(reader, PersonAddress + (ulong)PersonalityOffset("Professionalism")));
        Assert.Equal(
            55,
            ReadByte(reader, PlayerBlockAddress + (ulong)Layout.AttrsOffset + (ulong)AttributeOffset("Determination")));
    }

    [Fact]
    public void Wonderkid_mentality_rejects_a_changed_live_uid_without_writing()
    {
        var (reader, candidate) = CreatePlayerMemory(
            ambition: 10,
            professionalism: 10,
            determination: 10,
            liveUid: PlayerUid + 1);
        var service = CreateService(candidate, (_, _) => 20);

        var result = service.Execute(MentalityRequest(), SupportedGameVersion, reader, reader);

        Assert.False(result.Succeeded);
        Assert.Equal(PlayerBoostFailure.LiveIdentityMismatch, result.Failure);
        Assert.Equal(10, ReadByte(reader, PersonAddress + (ulong)PersonalityOffset("Ambition")));
        Assert.Equal(10, ReadByte(reader, PersonAddress + (ulong)PersonalityOffset("Professionalism")));
        Assert.Equal(
            50,
            ReadByte(reader, PlayerBlockAddress + (ulong)Layout.AttrsOffset + (ulong)AttributeOffset("Determination")));
    }

    [Fact]
    public void Wonderkid_mentality_leaves_a_null_snapshot_value_untouched()
    {
        var (reader, candidate) = CreatePlayerMemory(ambition: 5, professionalism: 10, determination: 11);
        var service = CreateService(candidate, (_, _) => 20);

        var result = service.Execute(
            MentalityRequest(
                expectedAmbition: null,
                expectedProfessionalism: 10,
                expectedDetermination: 11),
            SupportedGameVersion,
            reader,
            reader);

        Assert.True(result.Succeeded);
        Assert.NotNull(result.BoostResult);
        Assert.Null(result.BoostResult!.PreviousAmbition);
        Assert.Null(result.BoostResult.Ambition);
        Assert.Equal(10, result.BoostResult.PreviousProfessionalism);
        Assert.Equal(20, result.BoostResult.Professionalism);
        Assert.Equal(5, ReadByte(reader, PersonAddress + (ulong)PersonalityOffset("Ambition")));
        Assert.Equal(20, ReadByte(reader, PersonAddress + (ulong)PersonalityOffset("Professionalism")));
    }

    [Fact]
    public void Wonderkid_mentality_rejects_a_changed_known_value_before_writing()
    {
        var (reader, candidate) = CreatePlayerMemory(ambition: 10, professionalism: 8, determination: 11);
        var service = CreateService(candidate, (_, _) => 20);

        var result = service.Execute(
            MentalityRequest(
                expectedAmbition: 10,
                expectedProfessionalism: 9,
                expectedDetermination: 11),
            SupportedGameVersion,
            reader,
            reader);

        Assert.False(result.Succeeded);
        Assert.Equal(PlayerBoostFailure.ExpectedValuesMismatch, result.Failure);
        Assert.Equal(10, ReadByte(reader, PersonAddress + (ulong)PersonalityOffset("Ambition")));
        Assert.Equal(8, ReadByte(reader, PersonAddress + (ulong)PersonalityOffset("Professionalism")));
    }

    [Fact]
    public void Replaying_the_same_current_ability_request_cannot_apply_the_increment_twice()
    {
        var (reader, candidate) = CreatePlayerMemory(ca: 120, pa: 150);
        var service = CreateService(candidate);
        var request = CurrentAbilityRequest(expectedCa: 120, expectedPa: 150, increment: 5);

        var first = service.Execute(request, SupportedGameVersion, reader, reader);
        var second = service.Execute(request, SupportedGameVersion, reader, reader);

        Assert.True(first.Succeeded);
        Assert.False(second.Succeeded);
        Assert.Equal(PlayerBoostFailure.ExpectedValuesMismatch, second.Failure);
        Assert.Equal(125, ReadUInt16(reader, PlayerBlockAddress + (ulong)Layout.CurrentAbilityOffset));
    }

    [Fact]
    public void A_second_confirmed_current_ability_boost_uses_the_verified_cached_value()
    {
        var (reader, candidate) = CreatePlayerMemory(ca: 120, pa: 150);
        var service = CreateService(candidate);

        var first = service.Execute(
            CurrentAbilityRequest(expectedCa: 120, expectedPa: 150, increment: 5),
            SupportedGameVersion,
            reader,
            reader);
        var second = service.Execute(
            CurrentAbilityRequest(expectedCa: 125, expectedPa: 150, increment: 5),
            SupportedGameVersion,
            reader,
            reader);

        Assert.True(first.Succeeded);
        Assert.True(second.Succeeded);
        Assert.Equal(130, second.BoostResult!.CurrentAbility);
        Assert.Equal(130, ReadUInt16(reader, PlayerBlockAddress + (ulong)Layout.CurrentAbilityOffset));
    }

    [Fact]
    public void Wonderkid_mentality_restores_an_earlier_change_when_a_later_field_fails()
    {
        var (reader, candidate) = CreatePlayerMemory(ambition: 10, professionalism: 10, determination: 10);
        var targets = new Queue<int>(new[] { 11, 12, 13 });
        var service = CreateService(candidate, (_, _) => targets.Dequeue());

        var result = service.Execute(
            MentalityRequest(),
            SupportedGameVersion,
            reader,
            new FailSecondByteWriteMemoryWriter(reader));

        Assert.False(result.Succeeded);
        Assert.Equal(PlayerBoostFailure.MutationFailed, result.Failure);
        Assert.NotNull(result.BoostResult);
        Assert.Equal("restored", result.BoostResult!.Rollback);
        Assert.Equal(10, result.BoostResult.Ambition);
        Assert.Equal(10, result.BoostResult.Professionalism);
        Assert.Equal(10, result.BoostResult.Determination);
        Assert.Equal(10, ReadByte(reader, PersonAddress + (ulong)PersonalityOffset("Ambition")));
        Assert.Equal(10, ReadByte(reader, PersonAddress + (ulong)PersonalityOffset("Professionalism")));
        Assert.Equal(
            50,
            ReadByte(reader, PlayerBlockAddress + (ulong)Layout.AttrsOffset + (ulong)AttributeOffset("Determination")));
    }

    [Fact]
    public void Wonderkid_mentality_reports_restored_when_the_first_failed_write_is_recovered()
    {
        var (reader, candidate) = CreatePlayerMemory(ambition: 10, professionalism: 11, determination: 11);
        var service = CreateService(candidate, (_, _) => 20);

        var result = service.Execute(
            MentalityRequest(expectedAmbition: 10, expectedProfessionalism: 11, expectedDetermination: 11),
            SupportedGameVersion,
            reader,
            new WriteThenReportFailureMemoryWriter(reader));

        Assert.False(result.Succeeded);
        Assert.Equal(PlayerBoostFailure.MutationFailed, result.Failure);
        Assert.NotNull(result.BoostResult);
        Assert.Equal("restored", result.BoostResult!.Rollback);
        Assert.Equal(10, result.BoostResult.Ambition);
        Assert.Equal(10, ReadByte(reader, PersonAddress + (ulong)PersonalityOffset("Ambition")));
    }

    [Fact]
    public void Work_gate_excludes_a_second_operation_until_the_first_completes()
    {
        var gate = new BridgeWorkGate();

        Assert.True(gate.TryEnter());
        Assert.True(gate.IsBusy);
        Assert.False(gate.TryEnter());

        gate.Exit();

        Assert.False(gate.IsBusy);
        Assert.True(gate.TryEnter());
    }

    [Fact]
    public void Candidate_index_replaces_a_prior_live_scan_only_when_a_new_success_is_recorded()
    {
        var (_, firstCandidate) = CreatePlayerMemory();
        var (_, secondCandidate) = CreatePlayerMemory();
        var index = new PlayerMutationIndex();
        index.Replace("scan-1", SupportedGameVersion, new[] { firstCandidate });

        Assert.True(index.HasCandidatesForGameVersion(SupportedGameVersion));
        Assert.Equal(
            PlayerMutationLookup.Found,
            index.TryGet("scan-1", PlayerUid, out _));

        // A failed dump does not call Replace, so the last successful live scan remains usable.
        Assert.Equal(
            PlayerMutationLookup.Found,
            index.TryGet("scan-1", PlayerUid, out _));

        index.Replace("scan-2", SupportedGameVersion, new[] { secondCandidate });

        Assert.Equal(
            PlayerMutationLookup.SourceRequestMismatch,
            index.TryGet("scan-1", PlayerUid, out _));
        Assert.Equal(
            PlayerMutationLookup.Found,
            index.TryGet("scan-2", PlayerUid, out _));

        index.Clear();

        Assert.False(index.HasCandidatesForGameVersion(SupportedGameVersion));
        Assert.Equal(
            PlayerMutationLookup.MissingIndex,
            index.TryGet("scan-2", PlayerUid, out _));
    }

    [Theory]
    [InlineData((int)PlayerBoostFailure.ExpectedValuesMismatch, true)]
    [InlineData((int)PlayerBoostFailure.CurrentAbilityAtLimit, true)]
    [InlineData((int)PlayerBoostFailure.MutationFailed, false)]
    [InlineData((int)PlayerBoostFailure.PartialRollbackUnverified, false)]
    public void Plugin_preserves_live_indexes_only_for_proven_player_no_write_failures(
        int failure,
        bool expected)
    {
        Assert.Equal(
            expected,
            PlayerBoostOperationService.PreservesLiveIndexOnFailure(
                (PlayerBoostFailure)failure));
    }

    private static PlayerBoostOperationService CreateService(
        PersonCandidate candidate,
        Func<int, int, int>? nextRandom = null)
    {
        var index = new PlayerMutationIndex();
        index.Replace(SourceRequestId, SupportedGameVersion, new[] { candidate });
        return new PlayerBoostOperationService(
            new LayoutRegistry(new[] { Layout }),
            index,
            nextRandom);
    }

    private static BridgeRequest CurrentAbilityRequest(
        int expectedCa,
        int expectedPa,
        int increment,
        string sourceRequestId = SourceRequestId) =>
        new()
        {
            ProtocolVersion = BridgeProtocol.ProtocolVersion,
            RequestId = "boost-ca-1",
            CreatedAtUtc = DateTimeOffset.UtcNow,
            Operation = BridgeProtocol.OperationBoostCurrentAbility,
            SourceRequestId = sourceRequestId,
            PlayerUid = PlayerUid,
            ExpectedCurrentAbility = expectedCa,
            ExpectedPotentialAbility = expectedPa,
            CurrentAbilityIncrement = increment,
        };

    private static BridgeRequest MentalityRequest(
        int? expectedAmbition = 10,
        int? expectedProfessionalism = 10,
        int? expectedDetermination = 10) =>
        new()
        {
            ProtocolVersion = BridgeProtocol.ProtocolVersion,
            RequestId = "boost-mentality-1",
            CreatedAtUtc = DateTimeOffset.UtcNow,
            Operation = BridgeProtocol.OperationWonderkidMentality,
            SourceRequestId = SourceRequestId,
            PlayerUid = PlayerUid,
            ExpectedCurrentAbility = 120,
            ExpectedPotentialAbility = 150,
            ExpectedAmbition = expectedAmbition,
            ExpectedProfessionalism = expectedProfessionalism,
            ExpectedDetermination = expectedDetermination,
        };

    private static (FakeMemoryReader Reader, PersonCandidate Candidate) CreatePlayerMemory(
        int ca = 120,
        int pa = 150,
        int ambition = 10,
        int professionalism = 10,
        int determination = 10,
        uint liveUid = PlayerUid)
    {
        var reader = new FakeMemoryReader();
        var personBytes = new byte[PersonalityOffset("Professionalism") + 2];
        personBytes.AsSpan().Fill(0xA5);
        BitConverter.TryWriteBytes(personBytes.AsSpan(Layout.ObjectUidOffset), liveUid);
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

    private sealed class FailSecondByteWriteMemoryWriter : IMemoryWriter
    {
        private readonly FakeMemoryReader _inner;
        private int _byteWrites;

        public FailSecondByteWriteMemoryWriter(FakeMemoryReader inner)
        {
            _inner = inner;
        }

        public bool TryWriteByte(ulong address, byte value, out int bytesWritten)
        {
            _byteWrites++;
            if (_byteWrites == 2)
            {
                bytesWritten = 0;
                return false;
            }

            return _inner.TryWriteByte(address, value, out bytesWritten);
        }

        public bool TryWriteUInt16(ulong address, ushort value, out int bytesWritten) =>
            _inner.TryWriteUInt16(address, value, out bytesWritten);
    }

    private sealed class WriteThenReportFailureMemoryWriter : IMemoryWriter
    {
        private readonly FakeMemoryReader _inner;
        private bool _hasFailed;

        public WriteThenReportFailureMemoryWriter(FakeMemoryReader inner)
        {
            _inner = inner;
        }

        public bool TryWriteByte(ulong address, byte value, out int bytesWritten)
        {
            if (!_hasFailed)
            {
                _hasFailed = true;
                Assert.True(_inner.TryWriteByte(address, value, out _));
                bytesWritten = 0;
                return false;
            }

            return _inner.TryWriteByte(address, value, out bytesWritten);
        }

        public bool TryWriteUInt16(ulong address, ushort value, out int bytesWritten) =>
            _inner.TryWriteUInt16(address, value, out bytesWritten);
    }
}
