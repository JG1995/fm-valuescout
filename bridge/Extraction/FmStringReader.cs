using System.Buffers;
using System.Text;
using FmDataBridge.Memory;

namespace FmDataBridge.Extraction;

/// <summary>
/// FM nested / indirect UTF-8 string helpers (Il2Cpp string layout: UTF-8 at ptr+4).
/// </summary>
public static class FmStringReader
{
    public const int DefaultMaxLength = 128;

    public static string? TryReadNested(IMemoryReader reader, ulong slotAddress, int maxLength = DefaultMaxLength)
    {
        ArgumentNullException.ThrowIfNull(reader);
        if (!reader.TryReadUInt64(slotAddress, out var outer) || outer == 0)
        {
            return null;
        }

        if (!reader.TryReadUInt64(outer, out var inner) || inner == 0)
        {
            return null;
        }

        return TryReadCString(reader, inner + 4, maxLength);
    }

    public static string? TryReadIndirect(IMemoryReader reader, ulong address, int maxLength = DefaultMaxLength)
    {
        ArgumentNullException.ThrowIfNull(reader);
        if (!reader.TryReadUInt64(address, out var ptr) || ptr == 0)
        {
            return null;
        }

        return TryReadCString(reader, ptr + 4, maxLength);
    }

    public static string? TryReadCString(IMemoryReader reader, ulong address, int maxLength = DefaultMaxLength)
    {
        ArgumentNullException.ThrowIfNull(reader);
        if (maxLength <= 0)
        {
            return null;
        }

        // One bounded block read; uncleared/inaccessible gaps stay zero and act as terminators
        // (same practical stop as the former byte-by-byte short-region path).
        var buffer = ArrayPool<byte>.Shared.Rent(maxLength);
        try
        {
            reader.TryReadBlock(address, buffer, 0, maxLength, out _);

            var n = 0;
            while (n < maxLength && buffer[n] != 0)
            {
                n++;
            }

            if (n == 0)
            {
                return null;
            }

            var s = Encoding.UTF8.GetString(buffer, 0, n).Trim();
            return s.Length == 0 ? null : s;
        }
        finally
        {
            ArrayPool<byte>.Shared.Return(buffer);
        }
    }
}
