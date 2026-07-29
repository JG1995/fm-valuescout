namespace FmDataBridge.Extraction;

/// <summary>
/// Age from birth year/day-of-year against an in-game date.
/// </summary>
public static class PlayerAge
{
    public static int? At(int birthYear, int birthDayOfYear, int gameYear, int gameDayOfYear)
    {
        if (birthYear <= 0 || !FmDateDecoder.IsPlausible(birthYear, birthDayOfYear))
        {
            return null;
        }

        if (!FmDateDecoder.IsPlausible(gameYear, gameDayOfYear))
        {
            return null;
        }

        var age = gameYear - birthYear - (birthDayOfYear <= gameDayOfYear ? 0 : 1);
        return age is >= 0 and <= 80 ? age : null;
    }
}
