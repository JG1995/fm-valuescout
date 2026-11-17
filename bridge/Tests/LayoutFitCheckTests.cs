using FmDataBridge.Models;
using FmDataBridge.Scanning;
using Xunit;

namespace FmDataBridge.Tests;

public sealed class LayoutFitCheckTests
{
    [Fact]
    public void A_real_population_fits_every_group()
    {
        var groups = Evaluate(new Population());

        Assert.All(groups, group =>
        {
            Assert.True(group.Judged, group.Name);
            Assert.True(group.Fits, group.Name);
        });
    }

    public static TheoryData<string> Misreads() => new()
    {
        "players found",
        "names and birth dates",
        "CA and PA",
        "goalkeeper attributes",
        "outfield attributes",
        "personality",
        "height",
        "staff CA and PA",
        "staff attributes",
    };

    [Theory]
    [MemberData(nameof(Misreads))]
    public void A_misread_group_fails_alone(string group)
    {
        var population = group switch
        {
            "players found" => new Population { AbilityRejects = 900 },
            "names and birth dates" => new Population { PlayerCandidates = 3600 },
            "CA and PA" => new Population { Ca = 160 },
            "goalkeeper attributes" => new Population { KeeperRating = 2 },
            // Shifted position bytes read every player's GK familiarity as 1.
            "outfield attributes" => new Population { KeeperFamiliarity = 1 },
            "personality" => new Population { Personality = null },
            "height" => new Population { HeightCm = null },
            "staff CA and PA" => new Population { StaffCa = 130 },
            "staff attributes" => new Population { StaffAttribute = null },
            _ => throw new ArgumentOutOfRangeException(nameof(group)),
        };

        var failing = Evaluate(population).Where(g => !g.Fits).Select(g => g.Name);

        Assert.Equal(new[] { group }, failing);
    }

    [Fact]
    public void Groups_below_the_minimum_sample_are_reported_but_not_judged()
    {
        var groups = Evaluate(new Population { Players = 50, Personality = null });

        var personality = Assert.Single(groups, g => g.Name == "personality");
        Assert.Equal(0, personality.Passed);
        Assert.Equal(50, personality.Sample);
        Assert.False(personality.Judged);
        Assert.True(personality.Fits);
    }

    private static IReadOnlyList<LayoutFitGroup> Evaluate(Population population)
    {
        var players = Enumerable.Range(0, population.Players)
            .Select(index =>
            {
                var keeper = index % 10 == 0;
                var keeperRating = keeper ? population.KeeperRating : 2;
                return new PlayerFitSample(
                    population.Ca,
                    150,
                    keeper ? population.KeeperFamiliarity : 1,
                    new Dictionary<string, int?>
                    {
                        ["Handling"] = keeperRating,
                        ["OneOnOnes"] = keeperRating,
                        ["Reflexes"] = keeperRating,
                        ["Punching"] = keeperRating,
                        ["Passing"] = 12,
                    },
                    new Dictionary<string, int?> { ["Ambition"] = population.Personality, ["Loyalty"] = 10 },
                    population.HeightCm);
            })
            .ToList();
        var staff = Enumerable.Range(0, 150)
            .Select(_ => new StaffRecord
            {
                Ca = population.StaffCa,
                Pa = 120,
                Attributes = new Dictionary<string, int?>
                {
                    ["Attacking"] = population.StaffAttribute,
                    ["Tactical"] = 10,
                },
            })
            .ToList();

        return LayoutFitCheck.Evaluate(
            playerObjectsFound: population.Players + 10,
            population.AbilityRejects,
            population.PlayerCandidates ?? population.Players,
            players,
            staff);
    }

    private sealed record Population
    {
        /// <summary>One in ten is a keeper, so 1,200 players give 120 keepers.</summary>
        public int Players { get; init; } = 1200;

        public int AbilityRejects { get; init; } = 10;

        public int? PlayerCandidates { get; init; }

        public int Ca { get; init; } = 100;

        public int KeeperFamiliarity { get; init; } = 20;

        public int KeeperRating { get; init; } = 12;

        public int? Personality { get; init; } = 10;

        public int? HeightCm { get; init; } = 182;

        public int StaffCa { get; init; } = 100;

        public int? StaffAttribute { get; init; } = 10;
    }
}
