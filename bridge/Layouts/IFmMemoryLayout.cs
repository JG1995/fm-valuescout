namespace FmDataBridge.Layouts;

/// <summary>
/// Versioned FM memory field offsets for person/player discovery.
/// </summary>
public interface IFmMemoryLayout
{
    /// <summary>Major.minor key, e.g. <c>26.3</c>.</summary>
    string VersionKey { get; }

    string DisplayName { get; }

    /// <summary>Unique ID on the person/object header.</summary>
    int ObjectUidOffset { get; }

    /// <summary>Known person→player class offsets (from Il2Cpp meta+4).</summary>
    IReadOnlyList<int> PlayerClassOffsets { get; }

    /// <summary>Current ability (u16) relative to player block base (<c>person - classOffset</c>).</summary>
    int CurrentAbilityOffset { get; }

    /// <summary>Potential ability (u16) relative to player block base.</summary>
    int PotentialAbilityOffset { get; }

    /// <summary>
    /// True when offsets still need live confirmation on this machine's Steam build.
    /// </summary>
    bool IsProvisional { get; }
}
