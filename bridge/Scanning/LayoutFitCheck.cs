using FmDataBridge.Models;

namespace FmDataBridge.Scanning;

/// <summary>One field group's population check: the share of samples whose values look like FM data.</summary>
public sealed record LayoutFitGroup(string Name, int Passed, int Sample, double Required)
{
    /// <summary>A share needs a population; smaller groups are reported but not judged.</summary>
    public bool Judged => Sample >= LayoutFitCheck.MinSample;

    public bool Fits => !Judged || Passed >= Required * Sample;
}

/// <summary>The values of one read player that the fit check looks at.</summary>
public sealed record PlayerFitSample(
    int Ca,
    int Pa,
    int? GoalkeeperFamiliarity,
    IReadOnlyDictionary<string, int?> Attributes,
    IReadOnlyDictionary<string, int?> Personality,
    int? HeightCm);

/// <summary>
/// Checks, over everyone a scan read, that the layout's offsets still land on FM data. A patch
/// that inserts a field shifts the fields after it, so the old offsets read neighbouring values
/// whose population no longer looks like FM data. The scan is refused when a judged group fails.
/// </summary>
/// <remarks>
/// A 27.0 scan of 178,681 players and 95,482 staff measured CA ≤ PA 100%, personality 99.999%,
/// height 100%, goalkeepers 99.7%, outfield players 99.99%, staff CA ≤ PA 100%, and staff
/// attributes 94.0%. The thresholds sit below those rates and far above what shifted offsets read.
/// The players-found and name shares were not measured, so their thresholds only catch a collapse.
/// Fields that trade places within a run of same-range values (two personality bytes, CA and PA)
/// still pass.
/// </remarks>
public static class LayoutFitCheck
{
    // ponytail: groups are judged by population statistics, not against an independent source.
    // Upgrade to a save-file cross-check of CA, PA, and attributes if a patch's wrong values pass.

    public const int MinSample = 100;

    /// <summary>GK familiarity is almost always 1 or 20, so 10 splits keepers from outfielders.</summary>
    private const int KeeperFamiliarity = 10;

    /// <summary>Mean goalkeeping attribute (1–20) that separates keepers from outfielders.</summary>
    private const double KeeperScore = 6;

    // Spread over the attribute block (0x0B, 0x13, 0x15, 0x21) so a shift inside it also shows.
    private static readonly string[] KeeperAttributes = { "Handling", "OneOnOnes", "Reflexes", "Punching" };

    public static IReadOnlyList<LayoutFitGroup> Evaluate(
        int playerObjectsFound,
        int playerAbilityRejects,
        int playerCandidates,
        IReadOnlyCollection<PlayerFitSample> players,
        IReadOnlyCollection<StaffRecord> staff)
    {
        ArgumentNullException.ThrowIfNull(players);
        ArgumentNullException.ThrowIfNull(staff);

        int keepers = 0, keepersFit = 0, outfield = 0, outfieldFit = 0;
        foreach (var player in players)
        {
            if (player.GoalkeeperFamiliarity is not { } familiarity
                || KeeperScoreOf(player.Attributes) is not { } score)
            {
                continue;
            }

            if (familiarity >= KeeperFamiliarity)
            {
                keepers++;
                keepersFit += score >= KeeperScore ? 1 : 0;
            }
            else
            {
                outfield++;
                outfieldFit += score < KeeperScore ? 1 : 0;
            }
        }

        return new[]
        {
            new LayoutFitGroup(
                "players found",
                playerObjectsFound - playerAbilityRejects,
                playerObjectsFound,
                0.5),
            new LayoutFitGroup("names and birth dates", players.Count, playerCandidates, 0.5),
            new LayoutFitGroup("CA and PA", players.Count(p => p.Ca <= p.Pa), players.Count, 0.99),
            new LayoutFitGroup("goalkeeper attributes", keepersFit, keepers, 0.95),
            new LayoutFitGroup("outfield attributes", outfieldFit, outfield, 0.95),
            new LayoutFitGroup(
                "personality",
                players.Count(p => p.Personality.Values.All(value => value is not null)),
                players.Count,
                0.99),
            new LayoutFitGroup("height", players.Count(p => p.HeightCm is not null), players.Count, 0.99),
            new LayoutFitGroup("staff CA and PA", staff.Count(s => s.Ca <= s.Pa), staff.Count, 0.99),
            new LayoutFitGroup(
                "staff attributes",
                staff.Count(s => s.Attributes.Values.All(value => value is not null)),
                staff.Count,
                0.8),
        };
    }

    private static double? KeeperScoreOf(IReadOnlyDictionary<string, int?> attributes)
    {
        var sum = 0;
        var count = 0;
        foreach (var key in KeeperAttributes)
        {
            if (attributes.TryGetValue(key, out var value) && value is { } rating)
            {
                sum += rating;
                count++;
            }
        }

        return count == 0 ? null : (double)sum / count;
    }
}
