namespace FmDataBridge.Memory;

/// <summary>
/// Shared subdivision fill for caller-owned block reads.
/// </summary>
internal static class BlockReadHelper
{
    internal delegate bool ReadRange(
        ulong address,
        byte[] buffer,
        int offset,
        int length,
        out int bytesRead);

    public static bool TryFill(
        ulong address,
        byte[] buffer,
        int offset,
        int length,
        out int bytesRead,
        ReadRange direct,
        int minBlockSize = MemoryConstants.MinBlockReadSize)
    {
        ArgumentNullException.ThrowIfNull(buffer);
        ArgumentNullException.ThrowIfNull(direct);
        if (offset < 0 || length < 0 || offset > buffer.Length - length)
        {
            throw new ArgumentOutOfRangeException(nameof(length));
        }

        if (length == 0)
        {
            bytesRead = 0;
            return true;
        }

        Array.Clear(buffer, offset, length);
        return Fill(address, buffer, offset, length, out bytesRead, direct, minBlockSize);
    }

    private static bool Fill(
        ulong address,
        byte[] buffer,
        int offset,
        int length,
        out int bytesRead,
        ReadRange direct,
        int minBlockSize)
    {
        if (direct(address, buffer, offset, length, out bytesRead) && bytesRead == length)
        {
            return true;
        }

        if (length <= minBlockSize)
        {
            return bytesRead == length;
        }

        var half = length / 2;
        if (minBlockSize > 1)
        {
            var aligned = (half / minBlockSize) * minBlockSize;
            half = aligned > 0 ? aligned : minBlockSize;
        }

        var leftOk = Fill(address, buffer, offset, half, out var leftRead, direct, minBlockSize);
        var rightOk = Fill(
            address + (ulong)half,
            buffer,
            offset + half,
            length - half,
            out var rightRead,
            direct,
            minBlockSize);
        bytesRead = leftRead + rightRead;
        return leftOk && rightOk;
    }
}
