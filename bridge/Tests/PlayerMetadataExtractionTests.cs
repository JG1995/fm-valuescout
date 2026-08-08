using FmDataBridge.Extraction;
using FmDataBridge.Layouts;
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
        var layout = Fm263Layout.Instance;
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
