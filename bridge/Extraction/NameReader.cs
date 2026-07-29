using FmDataBridge.Layouts;
using FmDataBridge.Memory;

namespace FmDataBridge.Extraction;

public static class NameReader
{
    /// <summary>
    /// Display name: common/known-as when present, otherwise "First Second".
    /// </summary>
    public static string? TryReadDisplayName(
        IMemoryReader reader,
        ulong personAddress,
        IFmMemoryLayout layout)
    {
        ArgumentNullException.ThrowIfNull(reader);
        ArgumentNullException.ThrowIfNull(layout);

        var common = FmStringReader.TryReadNested(
            reader,
            personAddress + (ulong)layout.CommonNameOffset);
        if (!string.IsNullOrEmpty(common))
        {
            return common;
        }

        var first = FmStringReader.TryReadNested(
            reader,
            personAddress + (ulong)layout.FirstNameOffset);
        var second = FmStringReader.TryReadNested(
            reader,
            personAddress + (ulong)layout.SecondNameOffset);
        var parts = new[] { first, second }.Where(s => !string.IsNullOrEmpty(s));
        var name = string.Join(" ", parts);
        return string.IsNullOrEmpty(name) ? null : name;
    }
}
