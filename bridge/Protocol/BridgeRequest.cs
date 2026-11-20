namespace FmDataBridge.Protocol;

/// <summary>
/// Versioned request written by the Tauri app under the bridge data directory.
/// </summary>
public sealed class BridgeRequest
{
    public int ProtocolVersion { get; init; }

    public string RequestId { get; init; } = "";

    public DateTimeOffset CreatedAtUtc { get; init; }

    /// <summary>One closed operation defined by <see cref="BridgeProtocol"/>.</summary>
    public string Operation { get; init; } = "";

    /// <summary>Closed player database scope: men, women, or both.</summary>
    public string PlayerDatabaseScope { get; init; } = PlayerDatabaseScopes.Men;

    /// <summary>Successful full-dump request that supplied the live player candidate.</summary>
    public string? SourceRequestId { get; init; }

    /// <summary>Player identity for an action-specific boost.</summary>
    public uint? PlayerUid { get; init; }

    /// <summary>Staff identity for the closed staff CA boost.</summary>
    public uint? StaffUid { get; init; }

    /// <summary>CA observed by the source dump; used only as a stale-value precondition.</summary>
    public int? ExpectedCurrentAbility { get; init; }

    /// <summary>PA observed by the source dump; used only as a stale-value precondition.</summary>
    public int? ExpectedPotentialAbility { get; init; }

    /// <summary>Closed CA increment: only 5 or 10 is accepted for a CA boost.</summary>
    public int? CurrentAbilityIncrement { get; init; }

    /// <summary>Known snapshot Ambition for Wonderkid Mentality; null means do not modify it.</summary>
    public int? ExpectedAmbition { get; init; }

    /// <summary>Known snapshot Professionalism for Wonderkid Mentality; null means do not modify it.</summary>
    public int? ExpectedProfessionalism { get; init; }

    /// <summary>Known snapshot Determination for Wonderkid Mentality; null means do not modify it.</summary>
    public int? ExpectedDetermination { get; init; }

    /// <summary>The managed club whose own teams a squad move uses.</summary>
    public uint? ClubUid { get; init; }

    /// <summary>The players a squad move moves, each to a team level of <see cref="ClubUid"/>.</summary>
    public IReadOnlyList<SquadMoveItem>? SquadMoves { get; init; }
}

/// <summary>One player to move to the club's team at a level: senior, reserve, or youth.</summary>
public sealed class SquadMoveItem
{
    public uint PlayerUid { get; init; }

    public string TargetLevel { get; init; } = "";
}
