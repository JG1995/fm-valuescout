namespace FmDataBridge.Extraction;

/// <summary>
/// FM packed date: year = raw &gt;&gt; 16, day-of-year = raw &amp; 0x1FF.
/// </summary>
public static class FmDateDecoder
{
    public static (int Year, int DayOfYear) Decode(uint raw)
    {
        var year = (int)(raw >> 16);
        var doy = (int)(raw & 0x1FF);
        if (!IsPlausible(year, doy))
        {
            return (0, 0);
        }

        return (year, doy);
    }

    public static bool IsPlausible(int year, int dayOfYear) =>
        year is >= 1900 and <= 2100 && dayOfYear is >= 1 and <= 366;
}
