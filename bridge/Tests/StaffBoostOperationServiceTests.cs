using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Mutations;
using FmDataBridge.Protocol;
using FmDataBridge.Scanning;
using FmDataBridge.Tests.Fakes;
using Xunit;

namespace FmDataBridge.Tests;

public sealed class StaffBoostOperationServiceTests
{
    private const ulong PersonAddress = 0x20000;
    private const ulong StaffBlockAddress = 0x40000;
    private const uint StaffUid = 4321;
    private const string SourceRequestId = "R1";
    private const string GameVersion = "26.3.2";

    private static readonly IFmMemoryLayout Layout = Fm263Layout.Instance;

    [Fact]
    public void Staff_boost_adds_ten_and_reports_verified_values()
    {
        var (reader, candidate) = CreateStaffMemory(ca: 120, pa: 150);
        var service = CreateService(candidate);

        var result = service.Execute(Request(120, 150), GameVersion, reader, reader);

        Assert.True(result.Succeeded);
        Assert.Equal(StaffBoostFailure.None, result.Failure);
        Assert.Equal(120, result.BoostResult!.PreviousCurrentAbility);
        Assert.Equal(130, result.BoostResult.CurrentAbility);
        Assert.Equal(150, result.BoostResult.PotentialAbility);
        Assert.Equal(130, ReadUInt16(reader, StaffBlockAddress + (ulong)Layout.StaffCurrentAbilityOffset));
    }

    [Fact]
    public void Staff_boost_caps_to_potential_ability()
    {
        var (reader, candidate) = CreateStaffMemory(ca: 120, pa: 124);

        var result = CreateService(candidate).Execute(Request(120, 124), GameVersion, reader, reader);

        Assert.True(result.Succeeded);
        Assert.Equal(124, result.BoostResult!.CurrentAbility);
        Assert.Equal(124, ReadUInt16(reader, StaffBlockAddress + (ulong)Layout.StaffCurrentAbilityOffset));
    }

    [Fact]
    public void Staff_boost_rejects_current_ability_already_at_potential_without_writing()
    {
        var (reader, candidate) = CreateStaffMemory(ca: 150, pa: 150);

        var result = CreateService(candidate).Execute(Request(150, 150), GameVersion, reader, reader);

        Assert.False(result.Succeeded);
        Assert.Equal(StaffBoostFailure.CurrentAbilityAtLimit, result.Failure);
        Assert.Equal(150, ReadUInt16(reader, StaffBlockAddress + (ulong)Layout.StaffCurrentAbilityOffset));
    }

    [Fact]
    public void Staff_boost_accepts_a_human_manager_staff_candidate()
    {
        var (reader, candidate) = CreateStaffMemory(ca: 120, pa: 150);

        var result = CreateService(candidate with { Facet = PersonFacet.HumanManager })
            .Execute(Request(120, 150), GameVersion, reader, reader);

        Assert.True(result.Succeeded);
        Assert.Equal(130, result.BoostResult!.CurrentAbility);
    }

    [Fact]
    public void Staff_boost_rejects_a_stale_source_without_writing()
    {
        var (reader, candidate) = CreateStaffMemory(ca: 120, pa: 150);

        var result = CreateService(candidate).Execute(Request(120, 150, "R2"), GameVersion, reader, reader);

        Assert.False(result.Succeeded);
        Assert.Equal(StaffBoostFailure.SourceRequestMismatch, result.Failure);
        Assert.Equal(120, ReadUInt16(reader, StaffBlockAddress + (ulong)Layout.StaffCurrentAbilityOffset));
    }

    [Fact]
    public void Staff_boost_rejects_snapshot_or_live_value_mismatches_without_writing()
    {
        var (snapshotReader, candidate) = CreateStaffMemory(ca: 120, pa: 150);
        var staleCandidate = candidate with { Ca = 119 };
        var snapshotMismatch = CreateService(staleCandidate)
            .Execute(Request(119, 150), GameVersion, snapshotReader, snapshotReader);

        Assert.False(snapshotMismatch.Succeeded);
        Assert.Equal(StaffBoostFailure.ExpectedValuesMismatch, snapshotMismatch.Failure);

        var (liveReader, liveCandidate) = CreateStaffMemory(ca: 121, pa: 150);
        var liveMismatch = CreateService(liveCandidate)
            .Execute(Request(120, 150), GameVersion, liveReader, liveReader);

        Assert.False(liveMismatch.Succeeded);
        Assert.Equal(StaffBoostFailure.ExpectedValuesMismatch, liveMismatch.Failure);
        Assert.Equal(121, ReadUInt16(liveReader, StaffBlockAddress + (ulong)Layout.StaffCurrentAbilityOffset));
    }

    [Fact]
    public void Staff_boost_rejects_player_candidates_and_changed_live_identity()
    {
        var (reader, staff) = CreateStaffMemory(ca: 120, pa: 150, liveUid: StaffUid + 1);
        var changedIdentity = CreateService(staff).Execute(Request(120, 150), GameVersion, reader, reader);

        Assert.False(changedIdentity.Succeeded);
        Assert.Equal(StaffBoostFailure.LiveIdentityMismatch, changedIdentity.Failure);

        var index = new StaffMutationIndex();
        index.Replace(SourceRequestId, GameVersion, new[] { staff with { Facet = PersonFacet.Player } });
        var service = new StaffBoostOperationService(
            new LayoutRegistry(new[] { Layout }),
            index,
            supportsExactGameBuild: _ => true);
        var playerCandidate = service.Execute(Request(120, 150), GameVersion, reader, reader);

        Assert.False(playerCandidate.Succeeded);
        Assert.Equal(StaffBoostFailure.StaffNotFound, playerCandidate.Failure);
    }

    [Fact]
    public void Staff_boost_is_advertised_only_for_the_live_proved_exact_build()
    {
        var (reader, candidate) = CreateStaffMemory(ca: 120, pa: 150);
        var index = new StaffMutationIndex();
        index.Replace(SourceRequestId, GameVersion, new[] { candidate });
        var service = new StaffBoostOperationService(new LayoutRegistry(new[] { Layout }), index);

        var result = service.Execute(Request(120, 150), "26.3.1", reader, reader);

        Assert.True(service.SupportsExactGameBuild(GameVersion));
        Assert.True(service.HasSupportedLiveIndex(GameVersion));
        Assert.False(service.SupportsExactGameBuild("26.3.1"));
        Assert.False(result.Succeeded);
        Assert.Equal(StaffBoostFailure.UnsupportedGameBuild, result.Failure);
        Assert.Equal(120, ReadUInt16(reader, StaffBlockAddress + (ulong)Layout.StaffCurrentAbilityOffset));
    }

    [Fact]
    public void Staff_boost_restores_a_corrupt_readback_and_reports_unverified_rollback_when_restore_fails()
    {
        var (restoredReader, restoredCandidate) = CreateStaffMemory(ca: 120, pa: 150);
        var restored = CreateService(restoredCandidate).Execute(
            Request(120, 150),
            GameVersion,
            restoredReader,
            new CorruptFirstWriteMemoryWriter(restoredReader));

        Assert.False(restored.Succeeded);
        Assert.Equal(StaffBoostFailure.MutationFailed, restored.Failure);
        Assert.Equal("restored", restored.BoostResult!.Rollback);
        Assert.Equal(120, ReadUInt16(restoredReader, StaffBlockAddress + (ulong)Layout.StaffCurrentAbilityOffset));

        var (uncertainReader, uncertainCandidate) = CreateStaffMemory(ca: 120, pa: 150);
        var uncertain = CreateService(uncertainCandidate).Execute(
            Request(120, 150),
            GameVersion,
            uncertainReader,
            new PartialThenFailMemoryWriter(uncertainReader));

        Assert.False(uncertain.Succeeded);
        Assert.Equal(StaffBoostFailure.PartialRollbackUnverified, uncertain.Failure);
        Assert.Equal("partial-unverified", uncertain.BoostResult!.Outcome);
        Assert.Equal("unverified", uncertain.BoostResult.Rollback);
    }

    [Fact]
    public void Staff_candidate_index_updates_only_after_a_verified_write()
    {
        var (reader, candidate) = CreateStaffMemory(ca: 120, pa: 150);
        var service = CreateService(candidate);

        var first = service.Execute(Request(120, 150), GameVersion, reader, reader);
        var replay = service.Execute(Request(120, 150), GameVersion, reader, reader);
        var second = service.Execute(Request(130, 150), GameVersion, reader, reader);

        Assert.True(first.Succeeded);
        Assert.False(replay.Succeeded);
        Assert.Equal(StaffBoostFailure.ExpectedValuesMismatch, replay.Failure);
        Assert.True(second.Succeeded);
        Assert.Equal(140, second.BoostResult!.CurrentAbility);
    }

    private static StaffBoostOperationService CreateService(PersonCandidate candidate)
    {
        var index = new StaffMutationIndex();
        index.Replace(SourceRequestId, GameVersion, new[] { candidate });
        return new StaffBoostOperationService(
            new LayoutRegistry(new[] { Layout }),
            index,
            supportsExactGameBuild: _ => true);
    }

    private static BridgeRequest Request(
        int expectedCa,
        int expectedPa,
        string sourceRequestId = SourceRequestId) => new()
    {
        ProtocolVersion = BridgeProtocol.ProtocolVersion,
        RequestId = "staff-boost-1",
        CreatedAtUtc = DateTimeOffset.UtcNow,
        Operation = BridgeProtocol.OperationBoostStaffCurrentAbility,
        SourceRequestId = sourceRequestId,
        StaffUid = StaffUid,
        ExpectedCurrentAbility = expectedCa,
        ExpectedPotentialAbility = expectedPa,
    };

    private static (FakeMemoryReader Reader, PersonCandidate Candidate) CreateStaffMemory(
        int ca,
        int pa,
        uint liveUid = StaffUid)
    {
        var reader = new FakeMemoryReader();
        var personBytes = new byte[Layout.ObjectUidOffset + sizeof(uint) + 1];
        BitConverter.TryWriteBytes(personBytes.AsSpan(Layout.ObjectUidOffset), liveUid);
        reader.AddBytes(PersonAddress, personBytes);

        var staffBytes = new byte[Layout.StaffPotentialAbilityOffset + sizeof(ushort) + 1];
        BitConverter.TryWriteBytes(staffBytes.AsSpan(Layout.StaffCurrentAbilityOffset), (ushort)ca);
        BitConverter.TryWriteBytes(staffBytes.AsSpan(Layout.StaffPotentialAbilityOffset), (ushort)pa);
        reader.AddBytes(StaffBlockAddress, staffBytes);

        return (
            reader,
            new PersonCandidate(
                PersonAddress,
                StaffBlockAddress,
                StaffUid,
                ca,
                pa,
                ClassOffset: 0x100,
                PersonFacet.Staff));
    }

    private static ushort ReadUInt16(FakeMemoryReader reader, ulong address)
    {
        Assert.True(reader.TryReadUInt16(address, out var value));
        return value;
    }

    private sealed class CorruptFirstWriteMemoryWriter : IMemoryWriter
    {
        private readonly FakeMemoryReader _inner;
        private bool _first = true;

        public CorruptFirstWriteMemoryWriter(FakeMemoryReader inner) => _inner = inner;

        public bool TryWriteByte(ulong address, byte value, out int bytesWritten) =>
            _inner.TryWriteByte(address, value, out bytesWritten);

        public bool TryWriteUInt16(ulong address, ushort value, out int bytesWritten)
        {
            if (_first)
            {
                _first = false;
                return _inner.TryWriteUInt16(address, unchecked((ushort)(value + 1)), out bytesWritten);
            }

            return _inner.TryWriteUInt16(address, value, out bytesWritten);
        }
    }

    private sealed class PartialThenFailMemoryWriter : IMemoryWriter
    {
        private readonly FakeMemoryReader _inner;
        private bool _first = true;

        public PartialThenFailMemoryWriter(FakeMemoryReader inner) => _inner = inner;

        public bool TryWriteByte(ulong address, byte value, out int bytesWritten)
        {
            bytesWritten = 0;
            return false;
        }

        public bool TryWriteUInt16(ulong address, ushort value, out int bytesWritten)
        {
            if (_first)
            {
                _first = false;
                _inner.TryWriteByte(address, (byte)value, out bytesWritten);
                return false;
            }

            bytesWritten = 0;
            return false;
        }
    }
}
