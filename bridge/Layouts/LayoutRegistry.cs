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
        new(new IFmMemoryLayout[] { Fm263Layout.Instance });

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
    /// Maps <c>26.3.2.2329565</c> or <c>26.3</c> to the major.minor key <c>26.3</c>.
    /// </summary>
    public static string NormalizeVersionKey(string gameVersion)
    {
        if (string.IsNullOrWhiteSpace(gameVersion))
        {
            return string.Empty;
        }

        var parts = gameVersion.Trim().Split('.');
        if (parts.Length >= 2
            && int.TryParse(parts[0], out _)
            && int.TryParse(parts[1], out _))
        {
            return $"{parts[0]}.{parts[1]}";
        }

        return gameVersion.Trim();
    }
}
