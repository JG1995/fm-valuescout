namespace FmDataBridge.Scanning;

public enum PersonFacet
{
    Player,
    Staff,
    HumanManager,
}

public readonly record struct PersonCandidate(
    ulong ObjectAddress,
    ulong BlockAddress,
    uint Uid,
    int Ca,
    int Pa,
    int ClassOffset,
    PersonFacet Facet);

public sealed record PersonScanResult(
    IReadOnlyList<PersonCandidate> Players,
    IReadOnlyList<PersonCandidate> Staff,
    IReadOnlyList<PersonCandidate> HumanManagers,
    IReadOnlyList<uint> PlayerStaffOverlapUids,
    bool StoppedEarly,
    bool Cancelled);
