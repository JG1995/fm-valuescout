using System.Buffers;
using FmDataBridge.Layouts;
using FmDataBridge.Memory;

namespace FmDataBridge.Extraction;

public enum IdentityRejectReason
{
    EmptyName,
    ImpossibleDob,
}

public sealed class PlayerIdentity
{
    public string Name { get; init; } = "";

    public int BirthYear { get; init; }

    public int BirthDayOfYear { get; init; }

    public IReadOnlyList<string> Nationalities { get; init; } = Array.Empty<string>();

    /// <summary>UID from the primary nation object; null when unread or invalid.</summary>
    public uint? NationUid { get; init; }

    public int? HeightCm { get; init; }

    public string PreferredFoot { get; init; } = "";

    public IReadOnlyDictionary<string, int> Positions { get; init; } =
        new Dictionary<string, int>();
}

public static class PlayerIdentityReader
{
    public const int MinHeightCm = 140;
    public const int MaxHeightCm = 220;
    public const int NaturalPositionFloor = 15;

    public static PlayerIdentity? TryRead(
        IMemoryReader reader,
        ulong personAddress,
        ulong playerBlockBase,
        IFmMemoryLayout layout,
        out IdentityRejectReason? rejectReason)
    {
        ArgumentNullException.ThrowIfNull(reader);
        ArgumentNullException.ThrowIfNull(layout);
        rejectReason = null;

        var name = NameReader.TryReadDisplayName(reader, personAddress, layout);
        if (string.IsNullOrWhiteSpace(name))
        {
            rejectReason = IdentityRejectReason.EmptyName;
            return null;
        }

        if (!reader.TryReadUInt32(personAddress + (ulong)layout.DobOffset, out var dobRaw))
        {
            rejectReason = IdentityRejectReason.ImpossibleDob;
            return null;
        }

        var (birthYear, birthDoy) = FmDateDecoder.Decode(dobRaw);
        if (!FmDateDecoder.IsPlausible(birthYear, birthDoy))
        {
            rejectReason = IdentityRejectReason.ImpossibleDob;
            return null;
        }

        var nationalities = NationReader.TryRead(reader, personAddress, layout);
        var nationUid = NationReader.TryReadUid(reader, personAddress, layout);
        var heightCm = TryReadHeight(reader, playerBlockBase, layout);
        var preferredFoot = ReadPreferredFoot(reader, playerBlockBase, layout);
        var positions = ReadNaturalPositions(reader, playerBlockBase, layout);

        return new PlayerIdentity
        {
            Name = name,
            BirthYear = birthYear,
            BirthDayOfYear = birthDoy,
            Nationalities = nationalities,
            NationUid = nationUid,
            HeightCm = heightCm,
            PreferredFoot = preferredFoot,
            Positions = positions,
        };
    }

    private static int? TryReadHeight(IMemoryReader reader, ulong playerBlockBase, IFmMemoryLayout layout)
    {
        if (!reader.TryReadUInt16(playerBlockBase + (ulong)layout.HeightOffset, out var height))
        {
            return null;
        }

        return height is >= MinHeightCm and <= MaxHeightCm ? height : null;
    }

    private static string ReadPreferredFoot(
        IMemoryReader reader,
        ulong playerBlockBase,
        IFmMemoryLayout layout)
    {
        var leftRaw = reader.TryReadByte(
            playerBlockBase + (ulong)layout.AttrsOffset + (ulong)layout.FootLeftAttrOffset,
            out var leftByte)
            ? leftByte
            : (byte)0;
        var rightRaw = reader.TryReadByte(
            playerBlockBase + (ulong)layout.AttrsOffset + (ulong)layout.FootRightAttrOffset,
            out var rightByte)
            ? rightByte
            : (byte)0;
        var left = AttributeScale.DecodeScaled(leftRaw);
        var right = AttributeScale.DecodeScaled(rightRaw);

        if (right >= 14 && left >= 14)
        {
            return "either";
        }

        return right >= left ? "right" : "left";
    }

    private static IReadOnlyDictionary<string, int> ReadNaturalPositions(
        IMemoryReader reader,
        ulong playerBlockBase,
        IFmMemoryLayout layout)
    {
        var entries = layout.PositionEntries;
        var min = int.MaxValue;
        var max = 0;
        foreach (var entry in entries)
        {
            if (entry.Offset < min)
            {
                min = entry.Offset;
            }

            if (entry.Offset > max)
            {
                max = entry.Offset;
            }
        }

        var length = max - min + 1;
        var buffer = ArrayPool<byte>.Shared.Rent(length);
        try
        {
            reader.TryReadBlock(
                playerBlockBase + (ulong)layout.PositionsOffset + (ulong)min,
                buffer,
                0,
                length,
                out _);

            var rated = new List<(string Key, int Rating)>();
            foreach (var entry in entries)
            {
                var raw = buffer[entry.Offset - min];
                if (raw < 1)
                {
                    continue;
                }

                rated.Add((entry.Key, raw));
            }

            if (rated.Count == 0)
            {
                return new Dictionary<string, int>();
            }

            var top = rated.Max(x => x.Rating);
            var threshold = Math.Max(NaturalPositionFloor, top - 2);
            var natural = rated
                .Where(x => x.Rating >= threshold)
                .OrderByDescending(x => x.Rating)
                .ThenBy(x => x.Key, StringComparer.Ordinal)
                .ToDictionary(x => x.Key, x => x.Rating, StringComparer.Ordinal);

            if (natural.Count == 0)
            {
                var best = rated.OrderByDescending(x => x.Rating).First();
                natural[best.Key] = best.Rating;
            }

            return natural;
        }
        finally
        {
            ArrayPool<byte>.Shared.Return(buffer);
        }
    }
}
