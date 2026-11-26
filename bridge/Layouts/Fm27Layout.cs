namespace FmDataBridge.Layouts;

/// <summary>
/// FM27 layout pin, used for every 27.x patch.
/// </summary>
/// <remarks>
/// Offsets ported from FMSuperScout <c>plugin/Fields.cs</c> (pinned to game_plugin 26.3.0/26.3.2)
/// with author permission — see <c>.wiki/notes/superscout-permission.md</c>.
/// FM 27.0 kept every FM 26.3 offset: a live FM27 save matched the in-game data on 2026-11-07.
/// A patch is not checked by version: each scan's <see cref="Scanning.LayoutFitCheck"/> refuses the
/// data when these offsets no longer read FM values.
/// Structure and scanner are independent; values are research provenance, not a vendored plugin.
/// </remarks>
public sealed class Fm27Layout : IFmMemoryLayout
{
    // ponytail: FM27 boosts rely on the FM 26.3.2 controlled write proof, not a new FM27 one.
    // Upgrade to a recorded FM27 write proof if any FM27 boost reports an unverified readback.

    public static Fm27Layout Instance { get; } = new();

    public string VersionKey => "27";

    public string DisplayName => "FM 27";

    public int ObjectUidOffset => 0x0C;

    // Verified on FM 27.1 (2026-11-20): unique across every player; two players, three staff, and
    // a club matched the indexes FM's own move action sent.
    public int ObjectDatabaseIndexOffset => 0x08;

    public IReadOnlyList<int> PlayerClassOffsets { get; } = new[]
    {
        0x288, // pure player
        0x380, // player who is also staff
    };

    public IReadOnlyList<int> StaffClassOffsets { get; } = new[]
    {
        0x100, // pure staff
    };

    // FM26.3 used 0x450. A live FM 27.1 scan (2026-11-20) saw no 0x450 sub-object at all, but four
    // objects with the person pair 0x440/0x468, like staff 0x100/0x128 and players 0x288/0x2B0.
    public IReadOnlyList<int> HumanManagerClassOffsets { get; } = new[]
    {
        0x440,
    };

    public int CurrentAbilityOffset => 0x264;

    public int PotentialAbilityOffset => 0x266;

    public int StaffCurrentAbilityOffset => 0xDA;

    public int StaffPotentialAbilityOffset => 0xDC;

    public int StaffAttrsOffset => 0x10;

    // Stable English keys for the audited FMSuperScout StaffAttrs offsets.
    // Most are stored ×5; WorkingWithYoungsters is a raw 1–20 byte.
    public IReadOnlyList<AttributeLayoutEntry> StaffAttributeEntries { get; } = new AttributeLayoutEntry[]
    {
        new("Attacking", 0x22),
        new("Defending", 0x23),
        new("Fitness", 0x24),
        new("Possession", 0x25),
        new("Technical", 0x26),
        new("Tactical", 0x27),
        new("SetPieces", 0x33),
        new("Determination", 0x0D),
        new("ManManagement", 0x1E),
        new("Motivating", 0x1F),
        new("JudgingPlayerAbility", 0x1C),
        new("JudgingPlayerPotential", 0x1D),
        new("JudgingStaffAbility", 0x32),
        new("Negotiating", 0x31),
        new("TacticalKnowledge", 0x21),
        new("Physiotherapy", 0x20),
        new("SportsScience", 0x2F),
        // ponytail: Authority uses the accepted only-unmapped-byte pin for FM26.3.
        // Upgrade to a corrected offset if a live FM comparison disproves 0x30.
        new("Authority", 0x30),
        new("DataAnalysis", 0x2C),
        new("WorkingWithYoungsters", 0x0C),
        new("GoalkeepingDistribution", 0x2A),
        new("GoalkeepingHandling", 0x29),
        new("GoalkeepingReflexes", 0x1B),
    };

    public int FirstNameOffset => 0x50;

    public int SecondNameOffset => 0x58;

    public int CommonNameOffset => 0x60;

    public int NationPtrOffset => 0x68;

    public int GenderOffset => 0x19;

    public byte FemaleGenderBit => 0x10;

    public int DobOffset => 0x88;

    // Appearance bytes verified on FM 27.1, 2026-11-26.
    public int AppearanceFlagsOffset => 0x18;

    public byte GeneratedAppearanceBit => 0x08;

    public byte KeepsGreyHairBit => 0x08;

    public int EthnicityOffset => 0x8D;

    public int HairColourOffset => 0x8E;

    public int HairLengthOffset => 0x8F;

    public int SkinToneOffset => 0x90;

    public int FacialHairOffset => 0x91;

    public int HairRegistryVersionOffset => 0x95;

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

    // Visible attrs at AttrsOffset; stored ×5. Foot slots 0x18/0x19 are not in this list.
    public IReadOnlyList<AttributeLayoutEntry> AttributeEntries { get; } = new AttributeLayoutEntry[]
    {
        new("Crossing", 0x00),
        new("Dribbling", 0x01),
        new("Finishing", 0x02),
        new("Heading", 0x03),
        new("LongShots", 0x04),
        new("Marking", 0x05),
        new("OffTheBall", 0x06),
        new("Passing", 0x07),
        new("PenaltyTaking", 0x08),
        new("Tackling", 0x09),
        new("Vision", 0x0A),
        new("Handling", 0x0B),
        new("AerialReach", 0x0C),
        new("CommandOfArea", 0x0D),
        new("Communication", 0x0E),
        new("Kicking", 0x0F),
        new("Throwing", 0x10),
        new("Anticipation", 0x11),
        new("Decisions", 0x12),
        new("OneOnOnes", 0x13),
        new("Positioning", 0x14),
        new("Reflexes", 0x15),
        new("FirstTouch", 0x16),
        new("Technique", 0x17),
        new("Flair", 0x1A),
        new("Corners", 0x1B),
        new("Teamwork", 0x1C),
        new("WorkRate", 0x1D),
        new("LongThrows", 0x1E),
        new("Eccentricity", 0x1F),
        new("RushingOut", 0x20),
        new("Punching", 0x21),
        new("Acceleration", 0x22),
        new("FreeKicks", 0x23),
        new("Strength", 0x24),
        new("Stamina", 0x25),
        new("Pace", 0x26),
        new("JumpingReach", 0x27),
        new("Leadership", 0x28),
        new("Balance", 0x2A),
        new("Bravery", 0x2B),
        new("Aggression", 0x2D),
        new("Agility", 0x2E),
        new("NaturalFitness", 0x32),
        new("Determination", 0x33),
        new("Composure", 0x34),
        new("Concentration", 0x35),
    };

    public IReadOnlyList<AttributeLayoutEntry> HiddenAttributeEntries { get; } = new AttributeLayoutEntry[]
    {
        new("Dirtiness", 0x29),
        new("Consistency", 0x2C),
        new("ImportantMatches", 0x2F),
        new("InjuryProneness", 0x30),
        new("Versatility", 0x31),
    };

    // Person-relative personality bytes (Pada); already on the 1–20 scale.
    public IReadOnlyList<AttributeLayoutEntry> PersonalityEntries { get; } = new AttributeLayoutEntry[]
    {
        new("Adaptability", 0x70),
        new("Ambition", 0x71),
        new("Loyalty", 0x72),
        new("Pressure", 0x73),
        new("Professionalism", 0x74),
        new("Sportsmanship", 0x75),
        new("Temperament", 0x76),
        new("Controversy", 0x77),
    };

    public int FullContractPtrOffset => 0xA8;

    public int ContractWeeklyWageOffset => 0x20;

    public int ContractExpiryOffset => 0x48;

    // personJobTypes enum byte (FMSuperScout Dumper.cs ReadStaff).
    public int ContractJobIdOffset => 0x26;

    public int ContractStatusFlagsOffset => 0x57;

    // PLAO_GUIDE_VALUE — SuperScout verified as FM's real transfer value (GBP).
    public int MarketValueOffset => 0x234;

    public int CurrentReputationOffset => 0x260;

    public int WorldReputationOffset => 0x262;

    // Contract → team → club (SuperScout Dumper.cs / Fields.cs).
    public int ContractTeamPtrOffset => 0x10;

    public int TeamClubPtrOffset => 0x30;

    // Human-manager person pointer used by the FMSuperScout club walk.
    public int TeamManagerPtrOffset => 0x80;

    public int TeamTypeOffset => 0x28;

    public int TeamReputationOffset => 0xA8;

    public int TeamCompPtrOffset => 0x50;

    public int TeamCompAltPtrOffset => 0x60;

    public int TeamSchedulePtrOffset => 0xA0;

    public int ClubTeamsBeginOffset => 0x18;

    public int ClubTeamsEndOffset => 0x20;

    // Verified on FM 27.1: Real Madrid lists Castilla and Real Madrid C; Benfica, Porto, and
    // Sporting list their B clubs.
    public int ClubReserveTeamsBeginOffset => 0x30;

    public int ClubReserveTeamsEndOffset => 0x38;

    public int TeamSquadBeginOffset => 0x38;

    public int TeamSquadEndOffset => 0x40;

    public int ClubNameOffset => 0xC0;

    public int ClubShortNameOffset => 0xC8;

    public int CompNameOffset => 0x40;

    public int CompShortNameOffset => 0x48;

    public int ScheduleNextMatchOffset => 0x94;

    public int ScheduleNextMatchAltOffset => 0x18;

    // Ported from a public pin; still confirm identity fields on 2–3 known players after first dump.
    public bool IsProvisional => true;
}
