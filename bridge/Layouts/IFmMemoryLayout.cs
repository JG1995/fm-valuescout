namespace FmDataBridge.Layouts;

/// <summary>
/// Versioned FM memory field offsets for person/player discovery and identity.
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

    /// <summary>Nested string slot: first name (relative to person).</summary>
    int FirstNameOffset { get; }

    /// <summary>Nested string slot: second / family name (relative to person).</summary>
    int SecondNameOffset { get; }

    /// <summary>Nested string slot: common / known-as name (relative to person).</summary>
    int CommonNameOffset { get; }

    /// <summary>Pointer to nation object (relative to person).</summary>
    int NationPtrOffset { get; }

    /// <summary>Packed FM date of birth (u32) relative to person.</summary>
    int DobOffset { get; }

    /// <summary>Height in cm (u16) relative to player block base.</summary>
    int HeightOffset { get; }

    /// <summary>Base of 15 position suitability bytes relative to player block.</summary>
    int PositionsOffset { get; }

    /// <summary>Base of player attribute bytes (stored ×5) relative to player block.</summary>
    int AttrsOffset { get; }

    /// <summary>Left-foot attribute offset relative to <see cref="AttrsOffset"/>.</summary>
    int FootLeftAttrOffset { get; }

    /// <summary>Right-foot attribute offset relative to <see cref="AttrsOffset"/>.</summary>
    int FootRightAttrOffset { get; }

    /// <summary>Indirect string: nation short name relative to nation object.</summary>
    int NationShortNameOffset { get; }

    /// <summary>Indirect string: nation full name relative to nation object.</summary>
    int NationNameOffset { get; }

    /// <summary>Position key → byte offset from <see cref="PositionsOffset"/>.</summary>
    IReadOnlyList<PositionLayoutEntry> PositionEntries { get; }

    /// <summary>Visible attribute key → byte offset from <see cref="AttrsOffset"/> (stored ×5).</summary>
    IReadOnlyList<AttributeLayoutEntry> AttributeEntries { get; }

    /// <summary>Hidden attribute key → byte offset from <see cref="AttrsOffset"/> (stored ×5).</summary>
    IReadOnlyList<AttributeLayoutEntry> HiddenAttributeEntries { get; }

    /// <summary>Personality key → byte offset from person (raw 1–20).</summary>
    IReadOnlyList<AttributeLayoutEntry> PersonalityEntries { get; }

    /// <summary>Pointer to full-contract object (relative to person).</summary>
    int FullContractPtrOffset { get; }

    /// <summary>Weekly wage (u32 GBP) relative to contract object.</summary>
    int ContractWeeklyWageOffset { get; }

    /// <summary>Contract expiry packed FM date (u32) relative to contract object.</summary>
    int ContractExpiryOffset { get; }

    /// <summary>Transfer-status bitfield (byte) relative to contract object.</summary>
    int ContractStatusFlagsOffset { get; }

    /// <summary>FM market / guide value (u32 GBP) relative to player block.</summary>
    int MarketValueOffset { get; }

    /// <summary>Current reputation (u16) relative to player block.</summary>
    int CurrentReputationOffset { get; }

    /// <summary>World reputation (u16) relative to player block.</summary>
    int WorldReputationOffset { get; }

    /// <summary>
    /// True when offsets still need live confirmation on this machine's Steam build.
    /// </summary>
    bool IsProvisional { get; }
}

public readonly record struct PositionLayoutEntry(string Key, int Offset);

/// <summary>Named attribute field offset (relative base depends on the owning list).</summary>
public readonly record struct AttributeLayoutEntry(string Key, int Offset);
