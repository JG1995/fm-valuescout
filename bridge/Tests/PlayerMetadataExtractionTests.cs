using FmDataBridge.Extraction;
using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Models;
using FmDataBridge.Protocol;
using FmDataBridge.Tests.Fakes;
using Xunit;

namespace FmDataBridge.Tests;

public sealed class PlayerMetadataExtractionTests
{
    private const ulong PersonAddress = 0x200000;

    [Fact]
    public void Gender_reader_distinguishes_male_female_and_unread_values()
    {
        var layout = Fm27Layout.Instance;
        var unreadReader = new FakeMemoryReader();
        var maleReader = new FakeMemoryReader();
        var femaleReader = new FakeMemoryReader();

        Assert.Equal(PlayerGender.Unknown, PlayerGenderReader.Read(unreadReader, PersonAddress, layout));

        maleReader.AddBytes(PersonAddress + (ulong)layout.GenderOffset, new byte[] { 0x02 });
        Assert.Equal(PlayerGender.Male, PlayerGenderReader.Read(maleReader, PersonAddress, layout));

        femaleReader.AddBytes(PersonAddress + (ulong)layout.GenderOffset, new byte[] { 0x12 });
        Assert.Equal(PlayerGender.Female, PlayerGenderReader.Read(femaleReader, PersonAddress, layout));
    }

    [Fact]
    public void Appearance_reader_returns_fm_generated_appearance_and_gender_in_one_read()
    {
        var layout = Fm27Layout.Instance;
        var inner = new FakeMemoryReader();
        PlaceAppearance(inner, flags: 0x08, genderByte: 0x02);
        var reader = new CountingMemoryReader(inner);

        var looks = PlayerAppearanceReader.Read(reader, PersonAddress, layout);

        Assert.Equal(new[] { 0, 3, 2, 11, 0, 2, 0 }, looks.Appearance);
        Assert.Equal(PlayerGender.Male, looks.Gender);
        Assert.Equal(1, reader.CallCount);
    }

    [Fact]
    public void Appearance_reader_reports_kept_grey_hair_and_female_gender_from_the_gender_byte()
    {
        var layout = Fm27Layout.Instance;
        var reader = new FakeMemoryReader();
        PlaceAppearance(reader, flags: 0x08, genderByte: 0x1A);

        var looks = PlayerAppearanceReader.Read(reader, PersonAddress, layout);

        Assert.Equal(new[] { 0, 3, 2, 11, 0, 2, 1 }, looks.Appearance);
        Assert.Equal(PlayerGender.Female, looks.Gender);
    }

    [Fact]
    public void Appearance_reader_returns_null_without_the_generated_appearance_bit_or_bytes()
    {
        var layout = Fm27Layout.Instance;
        var realPlayer = new FakeMemoryReader();
        PlaceAppearance(realPlayer, flags: 0xF7, genderByte: 0x02);

        var looks = PlayerAppearanceReader.Read(realPlayer, PersonAddress, layout);

        Assert.Null(looks.Appearance);
        Assert.Equal(PlayerGender.Male, looks.Gender);

        var unread = PlayerAppearanceReader.Read(new FakeMemoryReader(), PersonAddress, layout);
        Assert.Null(unread.Appearance);
        Assert.Equal(PlayerGender.Unknown, unread.Gender);

        var flagsOnly = new FakeMemoryReader();
        flagsOnly.AddBytes(PersonAddress + 0x18, new byte[] { 0x08, 0x02 });
        var partial = PlayerAppearanceReader.Read(flagsOnly, PersonAddress, layout);
        Assert.Null(partial.Appearance);
        Assert.Equal(PlayerGender.Male, partial.Gender);
    }

    /// <summary>Real FM 27.1 newgen bytes at person +0x8C..+0x97.</summary>
    private static void PlaceAppearance(FakeMemoryReader reader, byte flags, byte genderByte)
    {
        reader.AddBytes(PersonAddress + 0x18, new[] { flags, genderByte });
        reader.AddBytes(
            PersonAddress + 0x8C,
            new byte[] { 0xff, 0x00, 0x0b, 0x02, 0x03, 0x00, 0x06, 0x00, 0x04, 0x02, 0x00, 0x00 });
    }

    [Fact]
    public void Player_database_scope_has_closed_wire_values_and_explicit_unknown_policy()
    {
        Assert.True(PlayerDatabaseScopes.TryParse("men", out var men));
        Assert.True(PlayerDatabaseScopes.TryParse("women", out var women));
        Assert.True(PlayerDatabaseScopes.TryParse("both", out var both));
        Assert.False(PlayerDatabaseScopes.TryParse("mixed", out _));
        Assert.False(PlayerDatabaseScopes.TryParse(null, out _));

        Assert.True(PlayerDatabaseScopes.Includes(men, PlayerGender.Male));
        Assert.True(PlayerDatabaseScopes.Includes(men, PlayerGender.Unknown));
        Assert.False(PlayerDatabaseScopes.Includes(men, PlayerGender.Female));
        Assert.False(PlayerDatabaseScopes.Includes(women, PlayerGender.Male));
        Assert.False(PlayerDatabaseScopes.Includes(women, PlayerGender.Unknown));
        Assert.True(PlayerDatabaseScopes.Includes(women, PlayerGender.Female));
        Assert.True(PlayerDatabaseScopes.Includes(both, PlayerGender.Male));
        Assert.True(PlayerDatabaseScopes.Includes(both, PlayerGender.Unknown));
        Assert.True(PlayerDatabaseScopes.Includes(both, PlayerGender.Female));
    }
}
