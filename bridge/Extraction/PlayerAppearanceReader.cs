using System.Buffers;
using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Models;

namespace FmDataBridge.Extraction;

/// <summary>Gender and the FM-generated appearance, both read from one person block.</summary>
public sealed record PlayerLooks(PlayerGender Gender, IReadOnlyList<int>? Appearance);

public static class PlayerAppearanceReader
{
    public static PlayerLooks Read(
        IMemoryReader reader,
        ulong personAddress,
        IFmMemoryLayout layout)
    {
        ArgumentNullException.ThrowIfNull(reader);
        ArgumentNullException.ThrowIfNull(layout);

        // Contract order of the appearance array; keeps-grey-hair follows as the seventh value.
        var appearanceOffsets = new[]
        {
            layout.EthnicityOffset,
            layout.SkinToneOffset,
            layout.HairLengthOffset,
            layout.HairColourOffset,
            layout.FacialHairOffset,
            layout.HairRegistryVersionOffset,
        };
        var start = Math.Min(layout.AppearanceFlagsOffset, layout.GenderOffset);
        var end = Math.Max(layout.AppearanceFlagsOffset, layout.GenderOffset);
        foreach (var offset in appearanceOffsets)
        {
            start = Math.Min(start, offset);
            end = Math.Max(end, offset);
        }

        var length = end - start + 1;
        var buffer = ArrayPool<byte>.Shared.Rent(length);
        try
        {
            var complete = reader.TryReadBlockWithCoverage(
                personAddress + (ulong)start,
                buffer,
                0,
                length,
                out var coverage);
            bool Readable(int offset) =>
                complete
                || (coverage.HasExactCoverage && coverage.CountReadableBytes(offset - start, 1) == 1);

            if (!Readable(layout.GenderOffset))
            {
                return new PlayerLooks(PlayerGender.Unknown, null);
            }

            var genderByte = buffer[layout.GenderOffset - start];
            var gender = PlayerGenderReader.Decode(genderByte, layout);
            if (!Readable(layout.AppearanceFlagsOffset)
                || (buffer[layout.AppearanceFlagsOffset - start] & layout.GeneratedAppearanceBit) == 0
                || !appearanceOffsets.All(Readable))
            {
                return new PlayerLooks(gender, null);
            }

            var appearance = new int[appearanceOffsets.Length + 1];
            for (var i = 0; i < appearanceOffsets.Length; i++)
            {
                appearance[i] = buffer[appearanceOffsets[i] - start];
            }

            appearance[^1] = (genderByte & layout.KeepsGreyHairBit) != 0 ? 1 : 0;
            return new PlayerLooks(gender, appearance);
        }
        finally
        {
            ArrayPool<byte>.Shared.Return(buffer);
        }
    }
}
