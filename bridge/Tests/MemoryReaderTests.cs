using FmDataBridge.Memory;
using FmDataBridge.Tests.Fakes;
using Xunit;

namespace FmDataBridge.Tests;

public sealed class MemoryReaderTests
{
    private const ulong MaxRegionSize = 64UL * 1024 * 1024;

    [Fact]
    public void Region_filter_keeps_committed_private_readwrite_pages()
    {
        var regions = new[]
        {
            MakeRegion(0x1000, 0x1000, MemoryConstants.PageReadWrite, MemoryConstants.MemPrivate, MemoryConstants.MemCommit),
            MakeRegion(0x2000, 0x1000, MemoryConstants.PageReadOnly, MemoryConstants.MemPrivate, MemoryConstants.MemCommit),
            MakeRegion(0x3000, 0x1000, MemoryConstants.PageReadWrite, MemoryConstants.MemImage, MemoryConstants.MemCommit),
            MakeRegion(0x4000, 0x1000, MemoryConstants.PageReadWrite, MemoryConstants.MemPrivate, MemoryConstants.MemReserve),
            MakeRegion(0x5000, 0x1000, MemoryConstants.PageReadWrite | MemoryConstants.PageGuard, MemoryConstants.MemPrivate, MemoryConstants.MemCommit),
            MakeRegion(0x6000, 0x1000, MemoryConstants.PageNoAccess, MemoryConstants.MemPrivate, MemoryConstants.MemCommit),
            MakeRegion(0x7000, 0x1000, MemoryConstants.PageExecuteReadWrite, MemoryConstants.MemPrivate, MemoryConstants.MemCommit),
        };

        var candidates = RegionEnumerator.SelectCandidates(regions, MaxRegionSize);

        Assert.Equal(2, candidates.Count);
        Assert.Equal(0x1000UL, candidates[0].BaseAddress);
        Assert.Equal(0x7000UL, candidates[1].BaseAddress);
    }

    [Fact]
    public void Region_filter_excludes_regions_over_max_size()
    {
        var regions = new[]
        {
            MakeRegion(0x1000, MaxRegionSize, MemoryConstants.PageReadWrite, MemoryConstants.MemPrivate, MemoryConstants.MemCommit),
            MakeRegion(0x2000, MaxRegionSize + 1, MemoryConstants.PageReadWrite, MemoryConstants.MemPrivate, MemoryConstants.MemCommit),
        };

        var candidates = RegionEnumerator.SelectCandidates(regions, MaxRegionSize);

        Assert.Single(candidates);
        Assert.Equal(0x1000UL, candidates[0].BaseAddress);
    }

    [Fact]
    public void Candidate_regions_come_from_reader_enumeration()
    {
        var reader = new FakeMemoryReader();
        reader.AddRegion(MakeRegion(0x1000, 0x1000, MemoryConstants.PageReadWrite, MemoryConstants.MemPrivate, MemoryConstants.MemCommit));
        reader.AddRegion(MakeRegion(0x2000, 0x1000, MemoryConstants.PageReadOnly, MemoryConstants.MemPrivate, MemoryConstants.MemCommit));

        var candidates = RegionEnumerator.GetCandidateRegions(reader, MaxRegionSize);

        Assert.Single(candidates);
        Assert.Equal(0x1000UL, candidates[0].BaseAddress);
    }

    [Fact]
    public void TryRead_returns_false_when_address_is_out_of_range()
    {
        var reader = new FakeMemoryReader();
        reader.AddBytes(0x1000, new byte[] { 1, 2, 3, 4 });

        Span<byte> buffer = stackalloc byte[4];
        var ok = reader.TryRead(0x2000, buffer, out var bytesRead);

        Assert.False(ok);
        Assert.Equal(0, bytesRead);
    }

    [Fact]
    public void TryRead_returns_false_on_short_read()
    {
        var reader = new FakeMemoryReader();
        reader.AddBytes(0x1000, new byte[] { 1, 2 });

        Span<byte> buffer = stackalloc byte[4];
        var ok = reader.TryRead(0x1000, buffer, out var bytesRead);

        Assert.False(ok);
        Assert.Equal(2, bytesRead);
        Assert.Equal(1, buffer[0]);
        Assert.Equal(2, buffer[1]);
    }

    [Fact]
    public void TryRead_copies_full_buffer_when_bytes_available()
    {
        var reader = new FakeMemoryReader();
        reader.AddBytes(0x1000, new byte[] { 10, 20, 30, 40, 50 });

        Span<byte> buffer = stackalloc byte[4];
        var ok = reader.TryRead(0x1001, buffer, out var bytesRead);

        Assert.True(ok);
        Assert.Equal(4, bytesRead);
        Assert.Equal(new byte[] { 20, 30, 40, 50 }, buffer.ToArray());
    }

    [Fact]
    public void TryReadUInt32_reads_little_endian_value()
    {
        var reader = new FakeMemoryReader();
        reader.AddBytes(0x1000, new byte[] { 0x78, 0x56, 0x34, 0x12 });

        Assert.True(reader.TryReadUInt32(0x1000, out var value));
        Assert.Equal(0x12345678u, value);
    }

    [Fact]
    public void TryReadUInt32_fails_when_bytes_unavailable()
    {
        var reader = new FakeMemoryReader();
        reader.AddBytes(0x1000, new byte[] { 0x01, 0x02 });

        Assert.False(reader.TryReadUInt32(0x1000, out var value));
        Assert.Equal(0u, value);
    }

    [Fact]
    public void Module_locator_finds_game_plugin_and_game_assembly_bounds()
    {
        var modules = new[]
        {
            new ProcessModuleInfo("fm.exe", 0x400000, 0x10000),
            new ProcessModuleInfo("game_plugin.dll", 0x7FF00000, 0x200000),
            new ProcessModuleInfo("GameAssembly.dll", 0x180000000, 0x4000000),
        };

        Assert.True(ModuleLocator.TryFind(modules, ModuleLocator.GamePluginModuleName, out var gamePlugin));
        Assert.Equal(0x7FF00000UL, gamePlugin.BaseAddress);
        Assert.Equal(0x7FF00000UL + 0x200000UL, gamePlugin.EndAddress);

        Assert.True(ModuleLocator.TryFind(modules, ModuleLocator.GameAssemblyModuleName, out var gameAssembly));
        Assert.Equal(0x180000000UL, gameAssembly.BaseAddress);
        Assert.Equal(0x180000000UL + 0x4000000UL, gameAssembly.EndAddress);
    }

    [Fact]
    public void Module_locator_returns_false_when_module_missing()
    {
        var modules = new[]
        {
            new ProcessModuleInfo("GameAssembly.dll", 0x180000000, 0x1000),
        };

        Assert.False(ModuleLocator.TryFind(modules, ModuleLocator.GamePluginModuleName, out _));
    }

    private static MemoryRegion MakeRegion(
        ulong baseAddress,
        ulong size,
        uint protect,
        uint type,
        uint state) =>
        new(baseAddress, size, protect, type, state);
}
