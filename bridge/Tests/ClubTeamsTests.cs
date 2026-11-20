using FmDataBridge.Mutations;
using Xunit;

namespace FmDataBridge.Tests;

public sealed class ClubTeamsTests
{
    [Fact]
    public void A_level_never_resolves_to_the_youth_candidates_team()
    {
        var teams = new[]
        {
            new ClubTeam(0x100, 0, new HashSet<ulong> { 1, 2 }),
            new ClubTeam(0x200, 12, new HashSet<ulong>()),
            new ClubTeam(0x300, 22, new HashSet<ulong> { 3 }),
        };

        Assert.Equal(12, ClubTeams.AtLevel(teams, "youth")?.Type);
        Assert.Null(ClubTeams.AtLevel(teams.Where(team => team.Type != 12).ToArray(), "youth"));
    }

    [Fact]
    public void Two_squad_teams_at_a_level_resolve_to_the_one_with_more_players()
    {
        var teams = new[]
        {
            new ClubTeam(0x100, 12, new HashSet<ulong> { 1 }),
            new ClubTeam(0x200, 18, new HashSet<ulong> { 2, 3 }),
        };

        Assert.Equal(18, ClubTeams.AtLevel(teams, "youth")?.Type);
    }
}
