namespace FmDataBridge.Models;

/// <summary>
/// One deterministic, bridge-internal human manager selected from pinned candidates.
/// </summary>
public sealed record HumanManager
{
    public uint Uid { get; init; }

    public string Name { get; init; } = "";

    public string? Club { get; init; }

    public int? ClubReputation { get; init; }
}
