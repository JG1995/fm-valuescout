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

    public int FirstNameOffset => 0x50;

    public int SecondNameOffset => 0x58;

    public int CommonNameOffset => 0x60;

    public int NationPtrOffset => 0x68;

    public int DobOffset => 0x88;

    public int HeightOffset => 0x22E;

    public int PositionsOffset => 0x150;

    public int AttrsOffset => 0x15F;

    public int FootLeftAttrOffset => 0x18;

    public int FootRightAttrOffset => 0x19;

    public int NationShortNameOffset => 0x20;

    public int NationNameOffset => 0x30;

    public IReadOnlyList<PositionLayoutEntry> PositionEntries { get; } = new PositionLayoutEntry[]
    {
        new("GK", 0x00),
        new("SW", 0x01),
        new("DL", 0x02),
        new("DC", 0x03),
        new("DR", 0x04),
        new("DM", 0x05),
        new("ML", 0x06),
        new("MC", 0x07),
        new("MR", 0x08),
        new("AML", 0x09),
        new("AMC", 0x0A),
        new("AMR", 0x0B),
        new("ST", 0x0C),
        new("WBL", 0x0D),
        new("WBR", 0x0E),
    };

    // Ported from a public pin; still confirm identity fields on 2–3 known players after first dump.
    public bool IsProvisional => true;
}
