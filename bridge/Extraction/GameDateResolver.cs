namespace FmDataBridge.Extraction;

public readonly record struct GameDateResolution(
    string? GameDate,
    string Source,
    string Basis,
    int Year,
    int DayOfYear);

/// <summary>
/// Derive an in-game date from next-fixture consensus, with a cohort/system-date fallback.
/// </summary>
public static class GameDateResolver
{
    public const string SourceDerived = "derived";
    public const string SourceUnknown = "unknown";
    public const string BasisNextFixtureConsensus = "next-fixture-consensus";
    public const string BasisBirthCohortAndSystemDate = "birth-cohort-and-system-date";
    public const string BasisUnknown = "unknown";

    public static GameDateResolution Resolve(
        IReadOnlyDictionary<uint, int> dateVotes,
        int youngestBirthCohortYear,
        DateTime? systemNow = null)
    {
        if (dateVotes.Count > 0)
        {
            var best = dateVotes.OrderByDescending(kv => kv.Value).ThenBy(kv => kv.Key).First();
            var (year, doy) = FmDateDecoder.Decode(best.Key);
            if (year >= 2020 && FmDateDecoder.IsPlausible(year, doy) && TryIso(year, doy, out var iso))
            {
                return new GameDateResolution(
                    iso,
                    SourceDerived,
                    BasisNextFixtureConsensus,
                    year,
                    doy);
            }
        }

        var now = systemNow ?? DateTime.UtcNow;
        var gameYear = youngestBirthCohortYear > 0
            ? youngestBirthCohortYear + 16
            : now.Year;
        if (gameYear is < 2000 or > 2100)
        {
            gameYear = now.Year;
        }

        var doyFallback = now.DayOfYear;
        if (TryIso(gameYear, doyFallback, out var derivedIso))
        {
            return new GameDateResolution(
                derivedIso,
                SourceDerived,
                BasisBirthCohortAndSystemDate,
                gameYear,
                doyFallback);
        }

        return new GameDateResolution(null, SourceUnknown, BasisUnknown, 0, 0);
    }

    public static int YoungestBirthCohortYear(IEnumerable<int> birthYears, int minCohortSize = 30)
    {
        var hist = new Dictionary<int, int>();
        foreach (var year in birthYears)
        {
            if (year is < 1990 or > 2100)
            {
                continue;
            }

            hist.TryGetValue(year, out var count);
            hist[year] = count + 1;
        }

        var youngest = 0;
        foreach (var (year, count) in hist)
        {
            if (count >= minCohortSize && year > youngest)
            {
                youngest = year;
            }
        }

        return youngest;
    }

    private static bool TryIso(int year, int dayOfYear, out string iso)
    {
        try
        {
            var date = new DateTime(year, 1, 1).AddDays(dayOfYear - 1);
            iso = date.ToString("yyyy-MM-dd");
            return true;
        }
        catch
        {
            iso = "";
            return false;
        }
    }
}
