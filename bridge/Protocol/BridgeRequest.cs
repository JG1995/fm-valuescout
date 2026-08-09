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

    /// <summary>
    /// Optional accepted-player cap. <c>null</c> (or omitted) means unlimited;
    /// a positive integer stops after that many accepted players.
    /// </summary>
    public int? MaxAccepted { get; init; }

    /// <summary>Closed player database scope: men, women, or both.</summary>
    public string PlayerDatabaseScope { get; init; } = PlayerDatabaseScopes.Men;

    /// <summary>Successful full-dump request that supplied the live player candidate.</summary>
    public string? SourceRequestId { get; init; }

    /// <summary>Player identity for an action-specific boost.</summary>
    public uint? PlayerUid { get; init; }

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
}
