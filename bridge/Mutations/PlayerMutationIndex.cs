using FmDataBridge.Scanning;

namespace FmDataBridge.Mutations;

internal enum PlayerMutationLookup
{
    Found,
    MissingIndex,
    SourceRequestMismatch,
    PlayerNotFound,
}

internal readonly record struct IndexedPlayerCandidate(
    string GameVersion,
    PersonCandidate Candidate);

/// <summary>
/// Keeps player addresses only for the one successful live dump that produced them.
/// The plugin work gate serializes access; this type never serializes its contents.
/// </summary>
internal sealed class PlayerMutationIndex
{
    private string? _sourceRequestId;
    private string? _gameVersion;
    private IReadOnlyDictionary<uint, PersonCandidate> _players =
        new Dictionary<uint, PersonCandidate>();

    public void Replace(
        string sourceRequestId,
        string gameVersion,
        IReadOnlyList<PersonCandidate> players)
    {
        if (string.IsNullOrWhiteSpace(sourceRequestId))
        {
            throw new ArgumentException("A source request id is required.", nameof(sourceRequestId));
        }

        if (string.IsNullOrWhiteSpace(gameVersion))
        {
            throw new ArgumentException("A game version is required.", nameof(gameVersion));
        }
        ArgumentNullException.ThrowIfNull(players);

        var next = new Dictionary<uint, PersonCandidate>();
        foreach (var player in players)
        {
            if (player.Facet == PersonFacet.Player)
            {
                next[player.Uid] = player;
            }
        }

        _sourceRequestId = sourceRequestId;
        _gameVersion = gameVersion;
        _players = next;
    }

    public void Clear()
    {
        _sourceRequestId = null;
        _gameVersion = null;
        _players = new Dictionary<uint, PersonCandidate>();
    }

    /// <summary>
    /// Advances the cached CA only after a verified live write so another confirmed boost from the same snapshot can
    /// still satisfy the stale-value precondition. A failed update leaves the existing index untouched for the caller
    /// to clear conservatively.
    /// </summary>
    public bool TryUpdateCurrentAbility(
        string sourceRequestId,
        uint playerUid,
        int expectedCurrentAbility,
        int expectedPotentialAbility,
        int currentAbility)
    {
        if (!string.Equals(_sourceRequestId, sourceRequestId, StringComparison.Ordinal)
            || !_players.TryGetValue(playerUid, out var player)
            || player.Ca != expectedCurrentAbility
            || player.Pa != expectedPotentialAbility
            || currentAbility is < 1 or > 200
            || currentAbility > expectedPotentialAbility)
        {
            return false;
        }

        var next = new Dictionary<uint, PersonCandidate>(_players)
        {
            [playerUid] = player with { Ca = currentAbility },
        };
        _players = next;
        return true;
    }

    public bool HasCandidatesForGameVersion(string gameVersion) =>
        _sourceRequestId is not null
        && _players.Count > 0
        && string.Equals(_gameVersion, gameVersion, StringComparison.Ordinal);

    public PlayerMutationLookup TryGet(
        string sourceRequestId,
        uint playerUid,
        out IndexedPlayerCandidate candidate)
    {
        candidate = default;
        if (_sourceRequestId is null || _gameVersion is null)
        {
            return PlayerMutationLookup.MissingIndex;
        }

        if (!string.Equals(_sourceRequestId, sourceRequestId, StringComparison.Ordinal))
        {
            return PlayerMutationLookup.SourceRequestMismatch;
        }

        if (!_players.TryGetValue(playerUid, out var player))
        {
            return PlayerMutationLookup.PlayerNotFound;
        }

        candidate = new IndexedPlayerCandidate(_gameVersion, player);
        return PlayerMutationLookup.Found;
    }
}
