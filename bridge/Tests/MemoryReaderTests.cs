using FmDataBridge.Memory;
using FmDataBridge.Tests.Fakes;
using Xunit;

namespace FmDataBridge.Tests;

public sealed class MemoryReaderTests
{
    private const ulong MaxRegionSize = 64UL * 1024 * 1024;

    /// <summary>
    /// Marks a test skipped on non-Windows hosts (xunit 2.x has no built-in Skip.If).
    /// </summary>
    private sealed class WindowsFactAttribute : FactAttribute
    {
        public WindowsFactAttribute()
        {
            if (!OperatingSystem.IsWindows())
            {
                Skip = "WindowsMemoryReader requires kernel32 ReadProcessMemory";
            }
        }
    }

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
    public void TryReadBlock_fills_caller_owned_buffer_on_full_success()
    {
        var reader = new FakeMemoryReader();
        var payload = new byte[0x2000];
        for (var i = 0; i < payload.Length; i++)
        {
            payload[i] = (byte)(i & 0xFF);
        }

        reader.AddBytes(0x10000, payload);

        var buffer = new byte[0x1800];
        var ok = reader.TryReadBlock(0x10000, buffer, 0, buffer.Length, out var bytesRead);

        Assert.True(ok);
        Assert.Equal(buffer.Length, bytesRead);
        Assert.Equal(payload.AsSpan(0, buffer.Length).ToArray(), buffer);
    }

    [Fact]
    public void TryReadBlock_returns_short_read_at_region_edge()
    {
        var reader = new FakeMemoryReader();
        reader.AddBytes(0x10000, new byte[] { 1, 2, 3, 4, 5, 6, 7, 8 });

        var buffer = new byte[16];
        buffer[8] = 0xEE;
        var ok = reader.TryReadBlock(0x10000, buffer, 0, 16, out var bytesRead);

        Assert.False(ok);
        Assert.Equal(8, bytesRead);
        Assert.Equal(new byte[] { 1, 2, 3, 4, 5, 6, 7, 8 }, buffer.AsSpan(0, 8).ToArray());
        Assert.Equal(0, buffer[8]);
    }

    [Fact]
    public void TryReadBlock_returns_false_for_invalid_address()
    {
        var reader = new FakeMemoryReader();
        reader.AddBytes(0x10000, new byte[] { 1, 2, 3, 4 });

        var buffer = new byte[4];
        buffer[0] = 0xFF;
        var ok = reader.TryReadBlock(0x20000, buffer, 0, 4, out var bytesRead);

        Assert.False(ok);
        Assert.Equal(0, bytesRead);
        Assert.Equal(0, buffer[0]);
    }

    [Fact]
    public void TryReadBlock_subdivides_between_one_and_two_pages_without_missing_trailing_page()
    {
        var reader = new FakeMemoryReader();
        var left = new byte[MemoryConstants.MinBlockReadSize];
        var right = new byte[MemoryConstants.MinBlockReadSize];
        for (var i = 0; i < left.Length; i++)
        {
            left[i] = 0x31;
            right[i] = 0x32;
        }

        reader.AddBytes(0x20000, left);
        reader.AddBytes(0x20000 + (ulong)(MemoryConstants.MinBlockReadSize * 2), right);

        // 2.5 pages: after a page-aligned first split, the right remainder is in (page, 2×page).
        var length = (MemoryConstants.MinBlockReadSize * 2) + (MemoryConstants.MinBlockReadSize / 2);
        var buffer = new byte[length];
        var ok = reader.TryReadBlock(0x20000, buffer, 0, length, out var bytesRead);

        Assert.False(ok);
        Assert.Equal(MemoryConstants.MinBlockReadSize + (MemoryConstants.MinBlockReadSize / 2), bytesRead);
        Assert.Equal(left, buffer.AsSpan(0, left.Length).ToArray());
        Assert.True(
            buffer.AsSpan(left.Length, MemoryConstants.MinBlockReadSize).ToArray().All(b => b == 0));
        Assert.Equal(
            right.AsSpan(0, MemoryConstants.MinBlockReadSize / 2).ToArray(),
            buffer.AsSpan(MemoryConstants.MinBlockReadSize * 2, MemoryConstants.MinBlockReadSize / 2).ToArray());
    }

    [Fact]
    public void TryReadBlock_subdivides_failed_block_around_inaccessible_gap()
    {
        var reader = new FakeMemoryReader();
        var left = new byte[MemoryConstants.MinBlockReadSize];
        var right = new byte[MemoryConstants.MinBlockReadSize];
        for (var i = 0; i < left.Length; i++)
        {
            left[i] = 0x11;
            right[i] = 0x22;
        }

        reader.AddBytes(0x10000, left);
        // Gap of one min-block between left and right — inaccessible for a spanning read.
        reader.AddBytes(0x10000 + (ulong)(MemoryConstants.MinBlockReadSize * 2), right);

        var length = MemoryConstants.MinBlockReadSize * 3;
        var buffer = new byte[length];
        var ok = reader.TryReadBlock(0x10000, buffer, 0, length, out var bytesRead);

        Assert.False(ok);
        Assert.Equal(MemoryConstants.MinBlockReadSize * 2, bytesRead);
        Assert.Equal(left, buffer.AsSpan(0, left.Length).ToArray());
        Assert.True(buffer.AsSpan(left.Length, MemoryConstants.MinBlockReadSize).ToArray().All(b => b == 0));
        Assert.Equal(
            right,
            buffer.AsSpan(MemoryConstants.MinBlockReadSize * 2, right.Length).ToArray());
    }

    [Fact]
    public void TryReadBlock_writes_into_caller_buffer_offset()
    {
        var reader = new FakeMemoryReader();
        reader.AddBytes(0x10000, new byte[] { 10, 20, 30, 40 });

        var buffer = new byte[] { 1, 1, 1, 1, 1, 1, 1, 1 };
        var ok = reader.TryReadBlock(0x10000, buffer, 2, 4, out var bytesRead);

        Assert.True(ok);
        Assert.Equal(4, bytesRead);
        Assert.Equal(new byte[] { 1, 1, 10, 20, 30, 40, 1, 1 }, buffer);
    }

    [WindowsFact]
    public void TryReadBlock_on_windows_fills_from_current_process_without_requiring_span_copy()
    {
        var source = new byte[64];
        for (var i = 0; i < source.Length; i++)
        {
            source[i] = (byte)(0xA0 + i);
        }

        var reader = new WindowsMemoryReader();
        var buffer = new byte[32];
        bool ok;
        int bytesRead;
        unsafe
        {
            fixed (byte* ptr = source)
            {
                ok = reader.TryReadBlock((ulong)ptr, buffer, 0, buffer.Length, out bytesRead);
            }
        }

        Assert.True(ok);
        Assert.Equal(buffer.Length, bytesRead);
        Assert.Equal(source.AsSpan(0, buffer.Length).ToArray(), buffer);
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
