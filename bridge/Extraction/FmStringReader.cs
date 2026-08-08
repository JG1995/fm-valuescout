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

        return TryAdd(inner, sizeof(uint), out var stringAddress)
            ? TryReadCString(reader, stringAddress, maxLength)
            : null;
    }

    public static string? TryReadIndirect(IMemoryReader reader, ulong address, int maxLength = DefaultMaxLength)
    {
        ArgumentNullException.ThrowIfNull(reader);
        if (!reader.TryReadUInt64(address, out var ptr) || ptr == 0)
        {
            return null;
        }

        return TryAdd(ptr, sizeof(uint), out var stringAddress)
            ? TryReadCString(reader, stringAddress, maxLength)
            : null;
    }

    public static string? TryReadCString(IMemoryReader reader, ulong address, int maxLength = DefaultMaxLength)
    {
        ArgumentNullException.ThrowIfNull(reader);
        if (maxLength <= 0)
        {
            return null;
        }

        if (!TryAdd(address, (ulong)(maxLength - 1), out _))
        {
            return null;
        }

        var buffer = ArrayPool<byte>.Shared.Rent(maxLength);
        try
        {
            if (reader.TryReadBlock(address, buffer, 0, maxLength, out _))
            {
                var fromBlock = DecodeCStringPrefix(buffer, maxLength);
                if (fromBlock != null)
                {
                    return fromBlock;
                }
            }

            // Block read failed or yielded empty — byte-by-byte for short regions near boundaries.
            return ReadCStringByteByByte(reader, address, maxLength);
        }
        finally
        {
            ArrayPool<byte>.Shared.Return(buffer);
        }
    }

    private static string? DecodeCStringPrefix(byte[] buffer, int maxLength)
    {
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

    private static string? ReadCStringByteByByte(IMemoryReader reader, ulong address, int maxLength)
    {
        var buffer = ArrayPool<byte>.Shared.Rent(maxLength);
        try
        {
            var n = 0;
            while (n < maxLength)
            {
                if (!TryAdd(address, (ulong)n, out var byteAddress)
                    || !reader.TryReadByte(byteAddress, out var b))
                {
                    break;
                }

                if (b == 0)
                {
                    break;
                }

                buffer[n++] = b;
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

    private static bool TryAdd(ulong address, ulong offset, out ulong result)
    {
        result = 0;
        if (offset > ulong.MaxValue - address)
        {
            return false;
        }

        result = address + offset;
        return true;
    }
}
