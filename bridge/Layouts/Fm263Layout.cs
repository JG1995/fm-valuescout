namespace FmDataBridge.Layouts;

/// <summary>
/// FM 26.3 layout pin.
/// </summary>
/// <remarks>
/// Offsets ported from FMSuperScout <c>plugin/Fields.cs</c> (pinned to game_plugin 26.3.0/26.3.2)
/// with author permission — see <c>.wiki/notes/superscout-permission.md</c>.
/// Structure and scanner are independent; values are research provenance, not a vendored plugin.
/// </remarks>
public sealed class Fm263Layout : IFmMemoryLayout
{
    public static Fm263Layout Instance { get; } = new();

    public string VersionKey => "26.3";

    public string DisplayName => "FM 26.3";

    public int ObjectUidOffset => 0x0C;

    public IReadOnlyList<int> PlayerClassOffsets { get; } = new[]
    {
        0x288, // pure player
        0x380, // player who is also staff
    };

    public int CurrentAbilityOffset => 0x264;

    public int PotentialAbilityOffset => 0x266;

    // Ported from a public pin; still confirm CA/PA on 2–3 known players after first dump.
    public bool IsProvisional => true;
}
