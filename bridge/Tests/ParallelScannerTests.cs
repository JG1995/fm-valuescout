using System.Text;
using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Protocol;
using FmDataBridge.Scanning;
using FmDataBridge.Tests.Fakes;
using Xunit;

namespace FmDataBridge.Tests;

public sealed class ParallelScannerTests
{
    private const ulong GameAssemblyBase = 0x180000000UL;
    private const ulong GameAssemblyEnd = 0x180100000UL;
    private const int PlayerClassOffset = 0x288;
    private const int StaffClassOffset = 0x100;
    private const int HumanManagerClassOffset = 0x450;
    private const ulong FirstRegionBase = 0x100000UL;
    private const ulong SecondRegionBase = 0x200000UL;
    private const ulong ThirdRegionBase = 0x300000UL;
    private const ulong FourthRegionBase = 0x400000UL;
    private const ulong RegionSize = 0x2000UL;

    private sealed class MultiCoreFactAttribute : FactAttribute
    {
        public MultiCoreFactAttribute()
        {
            if (Environment.ProcessorCount < 3)
            {
                Skip = "parallel scan needs two worker slots";
            }
        }
    }

    [MultiCoreFact]
    public void Person_scanner_overlaps_independent_region_reads()
    {
        var layout = Fm263Layout.Instance;
        var inner = new FakeMemoryReader();
        PlacePlayer(inner, layout, FirstRegionBase, uid: 101, slot: 1);
        PlacePlayer(inner, layout, SecondRegionBase, uid: 102, slot: 2);
        var reader = new BlockingMemoryReader(inner, FirstRegionBase, SecondRegionBase);

        var scanTask = Task.Run(() =>
            PersonScanner.Scan(
                reader,
                layout,
                new ModuleBounds("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd),
                gamePlugin: null,
                new[] { CandidateRegion(FirstRegionBase), CandidateRegion(SecondRegionBase) },
                new ScanDiagnostics()));

        var overlapped = reader.WaitForBothRegionReads(TimeSpan.FromSeconds(1));
        reader.ReleaseBlockedReads();
        var scan = scanTask.GetAwaiter().GetResult();

        Assert.True(overlapped);
        Assert.Equal(new uint[] { 101, 102 }, scan.Players.Select(candidate => candidate.Uid));
    }

    [Fact]
    public void Scan_worker_policy_caps_and_reduces_workers_from_available_memory()
    {
        var plentifulMemory = new SystemMemoryStatus(
            ScanWorkerPolicy.LowMemoryThresholdBytes,
            AvailableCommitBytes: 0,
            MemoryLoadPercent: 0);
        var lowMemory = new SystemMemoryStatus(
            ScanWorkerPolicy.LowMemoryThresholdBytes - 1,
            AvailableCommitBytes: 0,
            MemoryLoadPercent: 0);
        var zeroPhysicalMemory = new SystemMemoryStatus(
            AvailablePhysicalBytes: 0,
            AvailableCommitBytes: 1,
            MemoryLoadPercent: 100);

        Assert.Equal(0, ScanWorkerPolicy.GetWorkerCount(0, processorCount: 32, plentifulMemory));
        Assert.Equal(1, ScanWorkerPolicy.GetWorkerCount(16, processorCount: 2, plentifulMemory));
        Assert.Equal(8, ScanWorkerPolicy.GetWorkerCount(16, processorCount: 32, plentifulMemory));
        Assert.Equal(2, ScanWorkerPolicy.GetWorkerCount(16, processorCount: 32, lowMemory));
        Assert.Equal(2, ScanWorkerPolicy.GetWorkerCount(16, processorCount: 32, zeroPhysicalMemory));
        Assert.Equal(8, ScanWorkerPolicy.GetWorkerCount(16, processorCount: 32, default));
    }

    [MultiCoreFact]
    public void Person_scanner_parallel_matches_serial_values_and_semantic_diagnostics()
    {
        var layout = Fm263Layout.Instance;
        var reader = new FakeMemoryReader();
        var regions = new[]
        {
            CandidateRegion(SecondRegionBase),
            CandidateRegion(FirstRegionBase),
            CandidateRegion(ThirdRegionBase),
            CandidateRegion(FourthRegionBase),
            CandidateRegion(ThirdRegionBase),
        };
        foreach (var region in regions)
        {
            reader.AddRegion(region);
        }

        PlacePlayer(reader, layout, SecondRegionBase + 0x400, uid: 101, slot: 1);
        PlacePlayer(reader, layout, FirstRegionBase + 0x400, uid: 101, slot: 2);
        PlacePlayer(reader, layout, FourthRegionBase + 0x400, uid: 102, slot: 3);
        PlaceStaff(reader, layout, FirstRegionBase + 0x1000, StaffClassOffset, uid: 101, slot: 4);
        PlaceStaff(reader, layout, ThirdRegionBase + 0x400, StaffClassOffset, uid: 201, slot: 5);
        PlaceStaff(reader, layout, FourthRegionBase + 0x1000, HumanManagerClassOffset, uid: 301, slot: 6);
        PlaceStaff(reader, layout, ThirdRegionBase + 0x1000, HumanManagerClassOffset, uid: 301, slot: 7);
        PlaceClub(reader, layout, ThirdRegionBase + 0x1800, uid: 401, slot: 8, name: "Parallel FC");
        reader.AddUnreadableRange(ThirdRegionBase + 0x1D00, 0x100);

        var serialDiagnostics = new ScanDiagnostics();
        var serial = PersonScanner.Scan(
            new SerialMemoryReader(reader),
            layout,
            GameAssembly(),
            gamePlugin: null,
            regions,
            serialDiagnostics);
        var parallelDiagnostics = new ScanDiagnostics();
        var parallel = PersonScanner.Scan(
            reader,
            layout,
            GameAssembly(),
            gamePlugin: null,
            regions,
            parallelDiagnostics);

        Assert.Equal(serial.Players.ToArray(), parallel.Players.ToArray());
        Assert.Equal(serial.Staff.ToArray(), parallel.Staff.ToArray());
        Assert.Equal(serial.HumanManagers.ToArray(), parallel.HumanManagers.ToArray());
        Assert.Equal(serial.Clubs.ToArray(), parallel.Clubs.ToArray());
        Assert.Equal(serial.PlayerStaffOverlapUids.ToArray(), parallel.PlayerStaffOverlapUids.ToArray());
        Assert.Equal(serial.ReadQuality, parallel.ReadQuality);
        Assert.Equal(serialDiagnostics.SampleUids, parallelDiagnostics.SampleUids);
        Assert.Equal(
            serialDiagnostics.ClassOffsetHistogram.OrderBy(pair => pair.Key).ToArray(),
            parallelDiagnostics.ClassOffsetHistogram.OrderBy(pair => pair.Key).ToArray());
        Assert.Equal(serialDiagnostics.BytesScanned, parallelDiagnostics.BytesScanned);
        Assert.Equal(serialDiagnostics.VtableHits, parallelDiagnostics.VtableHits);
        Assert.Equal(serialDiagnostics.CandidatesAccepted, parallelDiagnostics.CandidatesAccepted);
        Assert.Equal(serialDiagnostics.StaffCandidatesAccepted, parallelDiagnostics.StaffCandidatesAccepted);
        Assert.Equal(serialDiagnostics.HumanManagerCandidatesAccepted, parallelDiagnostics.HumanManagerCandidatesAccepted);
        Assert.Equal(serialDiagnostics.PlayerStaffOverlapCount, parallelDiagnostics.PlayerStaffOverlapCount);
        Assert.Equal(serialDiagnostics.ClubCandidatesAccepted, parallelDiagnostics.ClubCandidatesAccepted);
        Assert.Equal(serialDiagnostics.ClubCandidatesRejected, parallelDiagnostics.ClubCandidatesRejected);
        Assert.Equal(serialDiagnostics.ClubCandidateDuplicatesSkipped, parallelDiagnostics.ClubCandidateDuplicatesSkipped);
        Assert.Equal(serialDiagnostics.CandidatesRejected, parallelDiagnostics.CandidatesRejected);
        Assert.Equal(serialDiagnostics.DuplicatesSkipped, parallelDiagnostics.DuplicatesSkipped);
        Assert.Equal(serialDiagnostics.ReadQuality, parallelDiagnostics.ReadQuality);
        Assert.Equal(1, serialDiagnostics.ScanWorkerCount);
        Assert.True(parallelDiagnostics.ScanWorkerCount > 1);
        Assert.Equal(MemoryConstants.DefaultScanBlockSize, parallelDiagnostics.ScanWorkerBufferBytes);
    }

    [MultiCoreFact]
    public void Person_scanner_parallel_preserves_candidates_at_scan_block_boundaries()
    {
        var layout = Fm263Layout.Instance;
        var reader = new BoundaryMemoryReader();
        var regions = new[]
        {
            LargeCandidateRegion(0x10000000UL),
            LargeCandidateRegion(0x20000000UL),
        };
        for (var index = 0; index < regions.Length; index++)
        {
            var region = regions[index];
            reader.AddRegion(region);
            PlaceBoundaryPlayer(
                reader,
                layout,
                region.BaseAddress + (ulong)MemoryConstants.DefaultScanBlockSize - sizeof(ulong),
                uid: (uint)(100 + index),
                slot: index + 9);
        }

        var serial = PersonScanner.Scan(
            new SerialMemoryReader(reader),
            layout,
            GameAssembly(),
            gamePlugin: null,
            regions,
            new ScanDiagnostics());
        var parallel = PersonScanner.Scan(
            reader,
            layout,
            GameAssembly(),
            gamePlugin: null,
            regions,
            new ScanDiagnostics());

        Assert.Equal(new uint[] { 100, 101 }, serial.Players.Select(candidate => candidate.Uid));
        Assert.Equal(serial.Players.ToArray(), parallel.Players.ToArray());
        Assert.Equal(serial.ReadQuality, parallel.ReadQuality);
    }

    [MultiCoreFact]
    public void Capped_person_scan_stays_serial_and_keeps_candidate_region_order()
    {
        var layout = Fm263Layout.Instance;
        var reader = new FakeMemoryReader();
        PlacePlayer(reader, layout, FirstRegionBase, uid: 101, slot: 1);
        PlacePlayer(reader, layout, SecondRegionBase, uid: 102, slot: 2);
        var diagnostics = new ScanDiagnostics();

        var result = PersonScanner.Scan(
            reader,
            layout,
            GameAssembly(),
            gamePlugin: null,
            new[] { CandidateRegion(SecondRegionBase), CandidateRegion(FirstRegionBase) },
            diagnostics,
            maxAccepted: 1);

        Assert.Equal(new uint[] { 102 }, result.Players.Select(candidate => candidate.Uid));
        Assert.True(result.StoppedEarly);
        Assert.Equal(1, diagnostics.ScanWorkerCount);
    }

    [MultiCoreFact]
    public void Person_scanner_cancellation_stops_parallel_workers()
    {
        var layout = Fm263Layout.Instance;
        var inner = new FakeMemoryReader();
        PlacePlayer(inner, layout, FirstRegionBase, uid: 101, slot: 1);
        PlacePlayer(inner, layout, SecondRegionBase, uid: 102, slot: 2);
        var reader = new BlockingMemoryReader(inner, FirstRegionBase, SecondRegionBase);
        using var cancellation = new CancellationTokenSource();
        var diagnostics = new ScanDiagnostics();

        var scanTask = Task.Run(() =>
            PersonScanner.Scan(
                reader,
                layout,
                GameAssembly(),
                gamePlugin: null,
                new[] { CandidateRegion(FirstRegionBase), CandidateRegion(SecondRegionBase) },
                diagnostics,
                cancellationToken: cancellation.Token));

        var overlapped = reader.WaitForBothRegionReads(TimeSpan.FromSeconds(1));
        cancellation.Cancel();
        reader.ReleaseBlockedReads();
        var result = scanTask.GetAwaiter().GetResult();

        Assert.True(overlapped);
        Assert.True(result.Cancelled);
        Assert.True(diagnostics.Cancelled);
        Assert.Empty(result.Players);
    }

    [MultiCoreFact]
    public void Person_scanner_propagates_worker_scalar_read_exceptions()
    {
        var layout = Fm263Layout.Instance;
        var inner = new FakeMemoryReader();
        PlacePlayer(inner, layout, FirstRegionBase, uid: 101, slot: 1);
        PlacePlayer(inner, layout, SecondRegionBase, uid: 102, slot: 2);
        var failedMetadataAddress = VtableForSlot(1) - sizeof(ulong);
        var reader = new ThrowingScalarReader(inner, failedMetadataAddress);

        var exception = Assert.Throws<InvalidOperationException>(() =>
            PersonScanner.Scan(
                reader,
                layout,
                GameAssembly(),
                gamePlugin: null,
                new[] { CandidateRegion(FirstRegionBase), CandidateRegion(SecondRegionBase) },
                new ScanDiagnostics()));

        Assert.Equal("forced worker scalar failure", exception.Message);
    }

    private sealed class SerialMemoryReader : IMemoryReader
    {
        private readonly IMemoryReader _inner;

        public SerialMemoryReader(IMemoryReader inner)
        {
            _inner = inner;
        }

        public string ReadSource => _inner.ReadSource;

        public IEnumerable<MemoryRegion> EnumerateRegions() => _inner.EnumerateRegions();

        public bool TryRead(ulong address, Span<byte> destination, out int bytesRead) =>
            _inner.TryRead(address, destination, out bytesRead);

        public bool TryReadBlock(ulong address, byte[] buffer, int offset, int length, out int bytesRead) =>
            _inner.TryReadBlock(address, buffer, offset, length, out bytesRead);

        public bool TryReadBlockWithCoverage(
            ulong address,
            byte[] buffer,
            int offset,
            int length,
            out BlockReadResult result) =>
            _inner.TryReadBlockWithCoverage(address, buffer, offset, length, out result);
    }

    private sealed class ThrowingScalarReader : IMemoryReader
    {
        private readonly IMemoryReader _inner;
        private readonly ulong _failedAddress;

        public ThrowingScalarReader(IMemoryReader inner, ulong failedAddress)
        {
            _inner = inner;
            _failedAddress = failedAddress;
        }

        public bool SupportsConcurrentReads => true;

        public string ReadSource => _inner.ReadSource;

        public IEnumerable<MemoryRegion> EnumerateRegions() => _inner.EnumerateRegions();

        public bool TryRead(ulong address, Span<byte> destination, out int bytesRead)
        {
            if (address == _failedAddress)
            {
                throw new InvalidOperationException("forced worker scalar failure");
            }

            return _inner.TryRead(address, destination, out bytesRead);
        }

        public bool TryReadBlock(ulong address, byte[] buffer, int offset, int length, out int bytesRead) =>
            _inner.TryReadBlock(address, buffer, offset, length, out bytesRead);

        public bool TryReadBlockWithCoverage(
            ulong address,
            byte[] buffer,
            int offset,
            int length,
            out BlockReadResult result) =>
            _inner.TryReadBlockWithCoverage(address, buffer, offset, length, out result);
    }

    private sealed class BoundaryMemoryReader : IMemoryReader
    {
        private readonly List<MemoryRegion> _regions = new();
        private readonly List<(ulong Address, byte[] Bytes)> _segments = new();

        public bool SupportsConcurrentReads => true;

        public void AddRegion(MemoryRegion region) => _regions.Add(region);

        public void AddBytes(ulong address, byte[] bytes) => _segments.Add((address, bytes));

        public IEnumerable<MemoryRegion> EnumerateRegions() => _regions;

        public bool TryRead(ulong address, Span<byte> destination, out int bytesRead)
        {
            CopySegments(address, destination);
            bytesRead = destination.Length;
            return true;
        }

        public bool TryReadBlock(ulong address, byte[] buffer, int offset, int length, out int bytesRead)
        {
            _ = TryReadBlockWithCoverage(address, buffer, offset, length, out var result);
            bytesRead = result.ReadableBytes;
            return true;
        }

        public bool TryReadBlockWithCoverage(
            ulong address,
            byte[] buffer,
            int offset,
            int length,
            out BlockReadResult result)
        {
            CopySegments(address, buffer.AsSpan(offset, length));
            result = BlockReadResult.FromReadablePrefix(length, length);
            return true;
        }

        private void CopySegments(ulong address, Span<byte> destination)
        {
            destination.Clear();
            var requestEnd = address + (ulong)destination.Length;
            foreach (var (segmentAddress, bytes) in _segments)
            {
                var segmentEnd = segmentAddress + (ulong)bytes.Length;
                if (segmentEnd <= address || segmentAddress >= requestEnd)
                {
                    continue;
                }

                var start = Math.Max(address, segmentAddress);
                var end = Math.Min(requestEnd, segmentEnd);
                var sourceOffset = (int)(start - segmentAddress);
                var destinationOffset = (int)(start - address);
                var length = (int)(end - start);
                bytes.AsSpan(sourceOffset, length).CopyTo(destination.Slice(destinationOffset, length));
            }
        }
    }

    private sealed class BlockingMemoryReader : IMemoryReader
    {
        private readonly IMemoryReader _inner;
        private readonly ulong _firstRegionBase;
        private readonly ulong _secondRegionBase;
        private readonly ManualResetEventSlim _bothRegionReads = new();
        private readonly ManualResetEventSlim _release = new();
        private int _blockedReads;

        public BlockingMemoryReader(IMemoryReader inner, ulong firstRegionBase, ulong secondRegionBase)
        {
            _inner = inner;
            _firstRegionBase = firstRegionBase;
            _secondRegionBase = secondRegionBase;
        }

        public bool SupportsConcurrentReads => true;

        public string ReadSource => _inner.ReadSource;

        public IEnumerable<MemoryRegion> EnumerateRegions() => _inner.EnumerateRegions();

        public bool TryRead(ulong address, Span<byte> destination, out int bytesRead) =>
            _inner.TryRead(address, destination, out bytesRead);

        public bool TryReadBlock(ulong address, byte[] buffer, int offset, int length, out int bytesRead) =>
            _inner.TryReadBlock(address, buffer, offset, length, out bytesRead);

        public bool TryReadBlockWithCoverage(
            ulong address,
            byte[] buffer,
            int offset,
            int length,
            out BlockReadResult result)
        {
            if (address == _firstRegionBase || address == _secondRegionBase)
            {
                if (Interlocked.Increment(ref _blockedReads) == 2)
                {
                    _bothRegionReads.Set();
                }

                _release.Wait();
            }

            return _inner.TryReadBlockWithCoverage(address, buffer, offset, length, out result);
        }

        public bool WaitForBothRegionReads(TimeSpan timeout) => _bothRegionReads.Wait(timeout);

        public void ReleaseBlockedReads() => _release.Set();
    }

    private static MemoryRegion CandidateRegion(ulong baseAddress) =>
        new(
            baseAddress,
            RegionSize,
            MemoryConstants.PageReadWrite,
            MemoryConstants.MemPrivate,
            MemoryConstants.MemCommit);

    private static MemoryRegion LargeCandidateRegion(ulong baseAddress) =>
        new(
            baseAddress,
            (ulong)MemoryConstants.DefaultScanBlockSize + 0x100,
            MemoryConstants.PageReadWrite,
            MemoryConstants.MemPrivate,
            MemoryConstants.MemCommit);

    private static void PlacePlayer(FakeMemoryReader reader, IFmMemoryLayout layout, ulong blockAddress, uint uid, int slot)
    {
        reader.AddRegion(CandidateRegion(blockAddress));

        var objectAddress = blockAddress + PlayerClassOffset;
        var vtable = VtableForSlot(slot);
        var metadata = GameAssemblyBase + 0x2000UL + (ulong)(slot * 0x100);
        var metadataBytes = new byte[8];
        BitConverter.TryWriteBytes(metadataBytes.AsSpan(sizeof(int)), PlayerClassOffset);
        reader.AddBytes(metadata, metadataBytes);
        reader.AddBytes(vtable - sizeof(ulong), BitConverter.GetBytes(metadata));

        var header = new byte[Math.Max(0x10, layout.ObjectUidOffset + sizeof(uint))];
        BitConverter.TryWriteBytes(header.AsSpan(), vtable);
        BitConverter.TryWriteBytes(header.AsSpan(layout.ObjectUidOffset), uid);
        reader.AddBytes(objectAddress, header);

        var abilities = new byte[layout.PotentialAbilityOffset + sizeof(ushort)];
        BitConverter.TryWriteBytes(abilities.AsSpan(layout.CurrentAbilityOffset), (ushort)120);
        BitConverter.TryWriteBytes(abilities.AsSpan(layout.PotentialAbilityOffset), (ushort)160);
        reader.AddBytes(blockAddress, abilities);
    }

    private static void PlaceStaff(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        ulong blockAddress,
        int classOffset,
        uint uid,
        int slot)
    {
        reader.AddRegion(CandidateRegion(blockAddress));

        var vtable = VtableForSlot(slot);
        var metadata = GameAssemblyBase + 0x2000UL + (ulong)(slot * 0x100);
        var metadataBytes = new byte[8];
        BitConverter.TryWriteBytes(metadataBytes.AsSpan(sizeof(int)), classOffset);
        reader.AddBytes(metadata, metadataBytes);
        reader.AddBytes(vtable - sizeof(ulong), BitConverter.GetBytes(metadata));

        var objectAddress = blockAddress + (ulong)classOffset;
        var header = new byte[Math.Max(0x10, layout.ObjectUidOffset + sizeof(uint))];
        BitConverter.TryWriteBytes(header.AsSpan(), vtable);
        BitConverter.TryWriteBytes(header.AsSpan(layout.ObjectUidOffset), uid);
        reader.AddBytes(objectAddress, header);

        var abilities = new byte[layout.StaffPotentialAbilityOffset + sizeof(ushort)];
        BitConverter.TryWriteBytes(abilities.AsSpan(layout.StaffCurrentAbilityOffset), (ushort)100);
        BitConverter.TryWriteBytes(abilities.AsSpan(layout.StaffPotentialAbilityOffset), (ushort)140);
        reader.AddBytes(blockAddress, abilities);
    }

    private static void PlaceClub(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        ulong address,
        uint uid,
        int slot,
        string name)
    {
        var vtable = VtableForSlot(slot);
        var metadata = GameAssemblyBase + 0x2000UL + (ulong)(slot * 0x100);
        var metadataBytes = new byte[8];
        BitConverter.TryWriteBytes(metadataBytes.AsSpan(sizeof(int)), 0x600);
        reader.AddBytes(metadata, metadataBytes);
        reader.AddBytes(vtable - sizeof(ulong), BitConverter.GetBytes(metadata));

        var header = new byte[Math.Max(0x10, layout.ObjectUidOffset + sizeof(uint))];
        BitConverter.TryWriteBytes(header.AsSpan(), vtable);
        BitConverter.TryWriteBytes(header.AsSpan(layout.ObjectUidOffset), uid);
        reader.AddBytes(address, header);

        var teamVector = address + 0x300;
        reader.AddBytes(address + (ulong)layout.ClubTeamsBeginOffset, BitConverter.GetBytes(teamVector));
        reader.AddBytes(
            address + (ulong)layout.ClubTeamsEndOffset,
            BitConverter.GetBytes(teamVector + sizeof(ulong)));
        reader.AddBytes(teamVector, BitConverter.GetBytes(0x160000UL));

        var nameObject = address + 0x200;
        reader.AddBytes(address + (ulong)layout.ClubNameOffset, BitConverter.GetBytes(nameObject));
        var utf8 = Encoding.UTF8.GetBytes(name + "\0");
        var payload = new byte[sizeof(uint) + utf8.Length];
        utf8.CopyTo(payload, sizeof(uint));
        reader.AddBytes(nameObject, payload);
    }

    private static void PlaceBoundaryPlayer(
        BoundaryMemoryReader reader,
        IFmMemoryLayout layout,
        ulong objectAddress,
        uint uid,
        int slot)
    {
        var vtable = VtableForSlot(slot);
        var metadata = GameAssemblyBase + 0x2000UL + (ulong)(slot * 0x100);
        var metadataBytes = new byte[8];
        BitConverter.TryWriteBytes(metadataBytes.AsSpan(sizeof(int)), PlayerClassOffset);
        reader.AddBytes(metadata, metadataBytes);
        reader.AddBytes(vtable - sizeof(ulong), BitConverter.GetBytes(metadata));

        var header = new byte[Math.Max(0x10, layout.ObjectUidOffset + sizeof(uint))];
        BitConverter.TryWriteBytes(header.AsSpan(), vtable);
        BitConverter.TryWriteBytes(header.AsSpan(layout.ObjectUidOffset), uid);
        reader.AddBytes(objectAddress, header);

        var blockAddress = objectAddress - PlayerClassOffset;
        var abilities = new byte[layout.PotentialAbilityOffset + sizeof(ushort)];
        BitConverter.TryWriteBytes(abilities.AsSpan(layout.CurrentAbilityOffset), (ushort)120);
        BitConverter.TryWriteBytes(abilities.AsSpan(layout.PotentialAbilityOffset), (ushort)160);
        reader.AddBytes(blockAddress, abilities);
    }

    private static ModuleBounds GameAssembly() =>
        new("GameAssembly.dll", GameAssemblyBase, GameAssemblyEnd);

    private static ulong VtableForSlot(int slot) =>
        GameAssemblyBase + 0x1000UL + (ulong)(slot * 0x100);
}
