using System.Text;
using FmDataBridge.Extraction;
using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Tests.Fakes;
using Xunit;

namespace FmDataBridge.Tests;

public sealed class ExtractionBatchingTests
{
    private const ulong PlayerBlockBase = 0x200000UL;
    private const int PlayerClassOffset = 0x288;
    private static readonly ulong PersonAddress = PlayerBlockBase + (ulong)PlayerClassOffset;

    [Fact]
    public void Attribute_reader_batches_contiguous_attr_and_personality_reads()
    {
        var layout = Fm263Layout.Instance;
        var inner = new FakeMemoryReader();
        WriteContiguousAttrs(inner, layout, acceleration: 13, pace: 14, consistency: 12);
        WriteContiguousPersonality(inner, layout, ambition: 16, loyalty: 10);
        var reader = new CountingMemoryReader(inner);

        var attrs = PlayerAttributeReader.Read(reader, PersonAddress, PlayerBlockBase, layout);

        Assert.Equal(13, attrs.Attributes["Acceleration"]);
        Assert.Equal(14, attrs.Attributes["Pace"]);
        Assert.Equal(12, attrs.HiddenAttributes["Consistency"]);
        Assert.Equal(16, attrs.Personality["Ambition"]);
        Assert.Equal(10, attrs.Personality["Loyalty"]);

        var perField =
            layout.AttributeEntries.Count
            + layout.HiddenAttributeEntries.Count
            + layout.PersonalityEntries.Count;
        Assert.True(
            reader.CallCount < perField / 4,
            $"expected batched field reads, got CallCount={reader.CallCount} vs per-field={perField}");
        Assert.True(
            reader.CallCount <= 3,
            $"expected attrs+hidden+personality in at most 3 reads, got {reader.CallCount}");
    }

    [Fact]
    public void Identity_reader_batches_contiguous_position_reads()
    {
        var layout = Fm263Layout.Instance;
        var inner = new FakeMemoryReader();
        PlaceIdentityScaffold(inner, layout, name: "Batch Pos");
        WriteContiguousPositions(inner, layout, ("ST", 20), ("AMC", 18), ("MC", 10));
        var reader = new CountingMemoryReader(inner);
        var callsBefore = reader.CallCount;

        var identity = PlayerIdentityReader.TryRead(
            reader,
            PersonAddress,
            PlayerBlockBase,
            layout,
            out var reject);

        Assert.Null(reject);
        Assert.NotNull(identity);
        Assert.Equal(20, identity!.Positions["ST"]);
        Assert.Equal(18, identity.Positions["AMC"]);
        Assert.False(identity.Positions.ContainsKey("MC"));

        var positionCalls = reader.CallCount - callsBefore;
        // Full identity also reads name pointers/strings, DOB, nation, height, feet —
        // but positions alone must not be one call per PositionEntries slot.
        Assert.True(
            positionCalls < layout.PositionEntries.Count,
            $"expected position batching within identity reads; CallCount delta={positionCalls}");
    }

    [Fact]
    public void Fm_string_reader_batches_bounded_cstring_reads()
    {
        var inner = new FakeMemoryReader();
        const ulong address = 0x5000;
        var utf8 = Encoding.UTF8.GetBytes("Batched Name\0");
        inner.AddBytes(address, utf8);
        // Pad so a maxLength read is contiguous for the counting assertion path.
        var pad = new byte[FmStringReader.DefaultMaxLength - utf8.Length];
        inner.AddBytes(address + (ulong)utf8.Length, pad);
        var reader = new CountingMemoryReader(inner);

        var value = FmStringReader.TryReadCString(reader, address);

        Assert.Equal("Batched Name", value);
        Assert.Equal(1, reader.CallCount);
    }

    [Fact]
    public void Fm_string_reader_falls_back_to_byte_reads_when_block_read_fails()
    {
        var inner = new FakeMemoryReader();
        const ulong address = 0x6000;
        inner.AddBytes(address, Encoding.UTF8.GetBytes("Boundary\0"));
        var reader = new FailFullBlockReader(inner);

        var value = FmStringReader.TryReadCString(reader, address);

        Assert.Equal("Boundary", value);
    }

    private sealed class FailFullBlockReader : IMemoryReader
    {
        private readonly FakeMemoryReader _inner;

        public FailFullBlockReader(FakeMemoryReader inner) => _inner = inner;

        public IEnumerable<MemoryRegion> EnumerateRegions() => _inner.EnumerateRegions();

        public bool TryRead(ulong address, Span<byte> destination, out int bytesRead) =>
            _inner.TryRead(address, destination, out bytesRead);

        public bool TryReadBlock(
            ulong address,
            byte[] buffer,
            int offset,
            int length,
            out int bytesRead)
        {
            bytesRead = 0;
            return false;
        }
    }

    private static void WriteContiguousAttrs(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        int acceleration,
        int pace,
        int consistency)
    {
        var maxOff = layout.AttributeEntries.Max(e => e.Offset);
        maxOff = Math.Max(maxOff, layout.HiddenAttributeEntries.Max(e => e.Offset));
        var blob = new byte[maxOff + 1];
        foreach (var entry in layout.AttributeEntries)
        {
            var v = entry.Key switch
            {
                "Acceleration" => acceleration,
                "Pace" => pace,
                _ => 10,
            };
            blob[entry.Offset] = (byte)(v * 5);
        }

        foreach (var entry in layout.HiddenAttributeEntries)
        {
            var v = entry.Key == "Consistency" ? consistency : 8;
            blob[entry.Offset] = (byte)(v * 5);
        }

        reader.AddBytes(PlayerBlockBase + (ulong)layout.AttrsOffset, blob);
    }

    private static void WriteContiguousPersonality(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        int ambition,
        int loyalty)
    {
        var min = layout.PersonalityEntries.Min(e => e.Offset);
        var max = layout.PersonalityEntries.Max(e => e.Offset);
        var blob = new byte[max - min + 1];
        foreach (var entry in layout.PersonalityEntries)
        {
            var v = entry.Key switch
            {
                "Ambition" => ambition,
                "Loyalty" => loyalty,
                _ => 12,
            };
            blob[entry.Offset - min] = (byte)v;
        }

        reader.AddBytes(PersonAddress + (ulong)min, blob);
    }

    private static void WriteContiguousPositions(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        params (string Key, int Rating)[] rated)
    {
        var max = layout.PositionEntries.Max(e => e.Offset);
        var blob = new byte[max + 1];
        foreach (var (key, rating) in rated)
        {
            var off = layout.PositionEntries.First(e => e.Key == key).Offset;
            blob[off] = (byte)rating;
        }

        reader.AddBytes(PlayerBlockBase + (ulong)layout.PositionsOffset, blob);
    }

    private static void PlaceIdentityScaffold(
        FakeMemoryReader reader,
        IFmMemoryLayout layout,
        string name)
    {
        var stringBase = PersonAddress + 0x10000;
        PlaceNestedString(reader, stringBase, name);
        reader.AddBytes(PersonAddress + (ulong)layout.CommonNameOffset, BitConverter.GetBytes(stringBase));
        uint dob = (2000u << 16) | 100u;
        reader.AddBytes(PersonAddress + (ulong)layout.DobOffset, BitConverter.GetBytes(dob));

        var heightBytes = new byte[2];
        BitConverter.TryWriteBytes(heightBytes, (ushort)180);
        reader.AddBytes(PlayerBlockBase + (ulong)layout.HeightOffset, heightBytes);

        reader.AddBytes(
            PlayerBlockBase + (ulong)layout.AttrsOffset + (ulong)layout.FootLeftAttrOffset,
            new byte[] { 25 });
        reader.AddBytes(
            PlayerBlockBase + (ulong)layout.AttrsOffset + (ulong)layout.FootRightAttrOffset,
            new byte[] { 90 });
    }

    private static void PlaceNestedString(FakeMemoryReader reader, ulong outerAddress, string value)
    {
        var inner = outerAddress + 0x40;
        reader.AddBytes(outerAddress, BitConverter.GetBytes(inner));
        var utf8 = Encoding.UTF8.GetBytes(value + "\0");
        var payload = new byte[4 + utf8.Length];
        utf8.CopyTo(payload, 4);
        reader.AddBytes(inner, payload);
        // Pad cstring region so batched maxLength reads succeed in nested name path.
        if (utf8.Length < FmStringReader.DefaultMaxLength)
        {
            reader.AddBytes(
                inner + 4 + (ulong)utf8.Length,
                new byte[FmStringReader.DefaultMaxLength - utf8.Length]);
        }
    }
}
