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

public readonly record struct ClubCandidate(ulong Address, string Name);

public sealed record PersonScanResult(
    IReadOnlyList<PersonCandidate> Players,
    IReadOnlyList<PersonCandidate> Staff,
    IReadOnlyList<PersonCandidate> HumanManagers,
    IReadOnlyList<ClubCandidate> Clubs,
    IReadOnlyList<uint> PlayerStaffOverlapUids,
    bool StoppedEarly,
    bool Cancelled);
