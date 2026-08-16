using FmDataBridge.Scanning;

namespace FmDataBridge.Mutations;

internal enum StaffMutationLookup
{
    Found,
    MissingIndex,
    SourceRequestMismatch,
    StaffNotFound,
}

internal readonly record struct IndexedStaffCandidate(
    string GameVersion,
    PersonCandidate Candidate);

/// <summary>
/// Keeps staff addresses only for the successful live dump that produced them.
/// The plugin work gate serializes access; this type never serializes its contents.
/// </summary>
internal sealed class StaffMutationIndex
{
    private string? _sourceRequestId;
    private string? _gameVersion;
    private IReadOnlyDictionary<uint, PersonCandidate> _staff =
        new Dictionary<uint, PersonCandidate>();

    public void Replace(
        string sourceRequestId,
        string gameVersion,
        IReadOnlyList<PersonCandidate> staff)
    {
        if (string.IsNullOrWhiteSpace(sourceRequestId))
        {
            throw new ArgumentException("A source request id is required.", nameof(sourceRequestId));
        }

        if (string.IsNullOrWhiteSpace(gameVersion))
        {
            throw new ArgumentException("A game version is required.", nameof(gameVersion));
        }
        ArgumentNullException.ThrowIfNull(staff);

        var next = new Dictionary<uint, PersonCandidate>();
        foreach (var candidate in staff)
        {
            if (candidate.Facet is PersonFacet.Staff or PersonFacet.HumanManager)
            {
                next[candidate.Uid] = candidate;
            }
        }

        _sourceRequestId = sourceRequestId;
        _gameVersion = gameVersion;
        _staff = next;
    }

    public void Clear()
    {
        _sourceRequestId = null;
        _gameVersion = null;
        _staff = new Dictionary<uint, PersonCandidate>();
    }

    public bool TryUpdateCurrentAbility(
        string sourceRequestId,
        uint staffUid,
        int expectedCurrentAbility,
        int expectedPotentialAbility,
        int currentAbility)
    {
        if (!string.Equals(_sourceRequestId, sourceRequestId, StringComparison.Ordinal)
            || !_staff.TryGetValue(staffUid, out var staff)
            || staff.Ca != expectedCurrentAbility
            || staff.Pa != expectedPotentialAbility
            || currentAbility is < 1 or > 200
            || currentAbility > expectedPotentialAbility)
        {
            return false;
        }

        var next = new Dictionary<uint, PersonCandidate>(_staff)
        {
            [staffUid] = staff with { Ca = currentAbility },
        };
        _staff = next;
        return true;
    }

    public bool HasCandidatesForGameVersion(string gameVersion) =>
        _sourceRequestId is not null
        && _staff.Count > 0
        && string.Equals(_gameVersion, gameVersion, StringComparison.Ordinal);

    public StaffMutationLookup TryGet(
        string sourceRequestId,
        uint staffUid,
        out IndexedStaffCandidate candidate)
    {
        candidate = default;
        if (_sourceRequestId is null || _gameVersion is null)
        {
            return StaffMutationLookup.MissingIndex;
        }

        if (!string.Equals(_sourceRequestId, sourceRequestId, StringComparison.Ordinal))
        {
            return StaffMutationLookup.SourceRequestMismatch;
        }

        if (!_staff.TryGetValue(staffUid, out var staff))
        {
            return StaffMutationLookup.StaffNotFound;
        }

        candidate = new IndexedStaffCandidate(_gameVersion, staff);
        return StaffMutationLookup.Found;
    }
}
