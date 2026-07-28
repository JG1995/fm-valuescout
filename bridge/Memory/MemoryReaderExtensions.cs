namespace FmDataBridge.Memory;

public static class MemoryReaderExtensions
{
    public static bool TryReadUInt32(this IMemoryReader reader, ulong address, out uint value)
    {
        ArgumentNullException.ThrowIfNull(reader);

        Span<byte> buffer = stackalloc byte[sizeof(uint)];
        if (!reader.TryRead(address, buffer, out _))
        {
            value = 0;
            return false;
        }

        value = BitConverter.ToUInt32(buffer);
        return true;
    }

    public static bool TryReadInt32(this IMemoryReader reader, ulong address, out int value)
    {
        if (!reader.TryReadUInt32(address, out var raw))
        {
            value = 0;
            return false;
        }

        value = unchecked((int)raw);
        return true;
    }

    public static bool TryReadUInt64(this IMemoryReader reader, ulong address, out ulong value)
    {
        ArgumentNullException.ThrowIfNull(reader);

        Span<byte> buffer = stackalloc byte[sizeof(ulong)];
        if (!reader.TryRead(address, buffer, out _))
        {
            value = 0;
            return false;
        }

        value = BitConverter.ToUInt64(buffer);
        return true;
    }
}
