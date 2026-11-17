namespace FmDataBridge.Layouts;

public sealed class LayoutRegistry
{
    private readonly Dictionary<string, IFmMemoryLayout> _layouts;

    public LayoutRegistry(IEnumerable<IFmMemoryLayout> layouts)
    {
        ArgumentNullException.ThrowIfNull(layouts);
        _layouts = new Dictionary<string, IFmMemoryLayout>(StringComparer.Ordinal);
        foreach (var layout in layouts)
        {
            _layouts[layout.VersionKey] = layout;
        }
    }

    public static LayoutRegistry CreateDefault() =>
        new(new IFmMemoryLayout[] { Fm27Layout.Instance });

    public bool TryResolve(string versionKey, out IFmMemoryLayout layout)
    {
        if (string.IsNullOrWhiteSpace(versionKey))
        {
            layout = null!;
            return false;
        }

        var key = NormalizeVersionKey(versionKey);
        return _layouts.TryGetValue(key, out layout!);
    }

    public bool TryResolveFromGameVersion(string gameVersion, out IFmMemoryLayout layout) =>
        TryResolve(NormalizeVersionKey(gameVersion), out layout);

    /// <summary>
    /// Maps <c>27.1.0.2505117</c> or <c>27.1</c> to the major key <c>27</c>. Patches within a
    /// major share its layout; the scan's fit check, not the version, decides whether it still reads.
    /// </summary>
    public static string NormalizeVersionKey(string gameVersion)
    {
        if (string.IsNullOrWhiteSpace(gameVersion))
        {
            return string.Empty;
        }

        var major = gameVersion.Trim().Split('.')[0];
        return int.TryParse(major, out _) ? major : gameVersion.Trim();
    }
}
