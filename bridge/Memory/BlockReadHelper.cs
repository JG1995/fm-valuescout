namespace FmDataBridge.Memory;

/// <summary>
/// Shared subdivision fill for caller-owned block reads.
/// </summary>
internal static class BlockReadHelper
{
    internal delegate BlockReadResult ReadRange(
        ulong address,
        byte[] buffer,
        int offset,
        int length);

    public static bool TryFill(
        ulong address,
        byte[] buffer,
        int offset,
        int length,
        out BlockReadResult result,
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
            result = BlockReadResult.Empty;
            return true;
        }

        Array.Clear(buffer, offset, length);
        result = Fill(address, buffer, offset, length, direct, minBlockSize);
        ClearUnreadBytes(buffer, offset, length, result);
        return result.IsComplete;
    }

    private static void ClearUnreadBytes(
        byte[] buffer,
        int offset,
        int length,
        BlockReadResult result)
    {
        var clearedThrough = 0;
        foreach (var range in result.ReadableRanges)
        {
            if (range.Offset > clearedThrough)
            {
                Array.Clear(buffer, offset + clearedThrough, range.Offset - clearedThrough);
            }

            clearedThrough = range.Offset + range.Length;
        }

        if (clearedThrough < length)
        {
            Array.Clear(buffer, offset + clearedThrough, length - clearedThrough);
        }
    }

    private static BlockReadResult Fill(
        ulong address,
        byte[] buffer,
        int offset,
        int length,
        ReadRange direct,
        int minBlockSize)
    {
        var directResult = direct(address, buffer, offset, length);
        if (directResult.IsComplete || length <= minBlockSize)
        {
            return directResult;
        }

        var half = length / 2;
        if (minBlockSize > 1)
        {
            var aligned = (half / minBlockSize) * minBlockSize;
            half = aligned > 0 ? aligned : minBlockSize;
        }

        var left = Fill(address, buffer, offset, half, direct, minBlockSize);
        var right = Fill(
            address + (ulong)half,
            buffer,
            offset + half,
            length - half,
            direct,
            minBlockSize);
        return BlockReadResult.Combine(length, left, right, half);
    }
}
