using System.Buffers.Binary;
using FmDataBridge.Extraction;
using FmDataBridge.Layouts;
using FmDataBridge.Memory;
using FmDataBridge.Protocol;

namespace FmDataBridge.Scanning;

public static class PersonScanner
{
    public const uint InvalidUid = 0xFFFFFFFFu;
    public const int MinAbility = 1;
    public const int MaxAbility = 200;

    /// <summary>
    /// Default diagnostic/test cap when a caller wants a bounded scan.
    /// Production Load Data passes request <c>maxAccepted: null</c> (unlimited).
    /// </summary>
    public const int DefaultMaxAccepted = 500;

    /// <summary>Minimum object header span covering vtable + UID for in-buffer reads.</summary>
    private const int MinObjectHeaderBytes = 0x10;

    public static PersonScanResult Scan(
        IMemoryReader reader,
        IFmMemoryLayout layout,
        ModuleBounds gameAssembly,
        ModuleBounds? gamePlugin,
        IReadOnlyList<MemoryRegion> candidateRegions,
        ScanDiagnostics diagnostics,
        int? maxAccepted = null,
        PlayerDatabaseScope playerDatabaseScope = PlayerDatabaseScope.Men,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(reader);
        ArgumentNullException.ThrowIfNull(layout);
        ArgumentNullException.ThrowIfNull(candidateRegions);
        ArgumentNullException.ThrowIfNull(diagnostics);
        if (maxAccepted is <= 0)
        {
            throw new ArgumentOutOfRangeException(nameof(maxAccepted), maxAccepted, "maxAccepted must be null or positive.");
        }

        if (layout.ObjectUidOffset is < 0 or > int.MaxValue - sizeof(uint))
        {
            throw new ArgumentOutOfRangeException(
                nameof(layout),
                "ObjectUidOffset must fit a four-byte object header read.");
        }

        diagnostics.RegionCount = candidateRegions.Count;
        diagnostics.LayoutVersionKey = layout.VersionKey;
        diagnostics.LayoutProvisional = layout.IsProvisional;
        diagnostics.MaxAccepted = maxAccepted;
        diagnostics.PlayerDatabaseScope = PlayerDatabaseScopes.ToWireValue(playerDatabaseScope);
        diagnostics.GameAssembly = new ModuleBoundsSnapshot(
            gameAssembly.BaseAddress,
            gameAssembly.EndAddress);
        if (gamePlugin is { } gp)
        {
            diagnostics.GamePlugin = new ModuleBoundsSnapshot(gp.BaseAddress, gp.EndAddress);
        }

        var playerOffsets = new HashSet<int>(layout.PlayerClassOffsets);
        var staffOffsets = new HashSet<int>(layout.StaffClassOffsets);
        var managerOffsets = new HashSet<int>(layout.HumanManagerClassOffsets);
        var players = new Dictionary<uint, PersonCandidate>();
        var staff = new Dictionary<uint, PersonCandidate>();
        var managers = new Dictionary<uint, PersonCandidate>();
        var clubs = new Dictionary<ulong, ClubCandidate>();
        var classOffsetByVtable = new Dictionary<ulong, int>();
        var atCap = false;
        var stoppedDueToCap = false;
        var headerBytes = Math.Max(MinObjectHeaderBytes, layout.ObjectUidOffset + sizeof(uint));
        if (!TryAlignUp((ulong)headerBytes, 8, out var overlap))
        {
            throw new ArgumentOutOfRangeException(nameof(layout), "ObjectUidOffset cannot produce an aligned scan header.");
        }

        var buffer = new byte[MemoryConstants.DefaultScanBlockSize];

        foreach (var region in candidateRegions)
        {
            if (stoppedDueToCap)
            {
                break;
            }

            if (cancellationToken.IsCancellationRequested)
            {
                diagnostics.Cancelled = true;
                break;
            }

            if (region.Size < (ulong)headerBytes)
            {
                continue;
            }

            if (!TryAdd(region.BaseAddress, region.Size, out var end)
                || !TryAlignUp(region.BaseAddress, 8, out var blockStart)
                || blockStart > end
                || end - blockStart < (ulong)headerBytes)
            {
                diagnostics.CandidatesRejected++;
                continue;
            }

            while (end - blockStart >= (ulong)headerBytes)
            {
                if (stoppedDueToCap)
                {
                    break;
                }

                if (cancellationToken.IsCancellationRequested)
                {
                    diagnostics.Cancelled = true;
                    return BuildResult(players, staff, managers, clubs, diagnostics);
                }

                var remaining = end - blockStart;
                var toRead = (int)Math.Min((ulong)buffer.Length, remaining);
                if (toRead < headerBytes)
                {
                    break;
                }

                // Failed/partial fills leave cleared gaps as zero — scan the full requested length.
                _ = reader.TryReadBlock(blockStart, buffer, 0, toRead, out _);

                for (var local = 0;
                     local + headerBytes <= toRead && blockStart + (ulong)local + (ulong)headerBytes <= end;
                     local += 8)
                {
                    if (cancellationToken.IsCancellationRequested)
                    {
                        diagnostics.Cancelled = true;
                        return BuildResult(players, staff, managers, clubs, diagnostics);
                    }

                    diagnostics.BytesScanned += 8;

                    var address = blockStart + (ulong)local;
                    var vtable = BinaryPrimitives.ReadUInt64LittleEndian(buffer.AsSpan(local, sizeof(ulong)));

                    if (!IsModuleVtable(vtable, gameAssembly, gamePlugin))
                    {
                        continue;
                    }

                    diagnostics.VtableHits++;

                    if (!TryResolveDynamicOffsetCached(reader, vtable, classOffsetByVtable, out var classOffset)
                        || classOffset == 0)
                    {
                        diagnostics.CandidatesRejected++;
                        continue;
                    }

                    if (classOffset is > 0 and < 0x2000)
                    {
                        diagnostics.RecordClassOffsetHit(classOffset);
                    }

                    var uid = BinaryPrimitives.ReadUInt32LittleEndian(
                        buffer.AsSpan(local + layout.ObjectUidOffset, sizeof(uint)));
                    if (!IsValidUid(uid))
                    {
                        diagnostics.CandidatesRejected++;
                        continue;
                    }

                    if (!TryGetFacet(classOffset, playerOffsets, staffOffsets, managerOffsets, out var facet))
                    {
                        diagnostics.CandidatesRejected++;
                        if (TryReadClubCandidate(reader, layout, address, out var clubCandidate))
                        {
                            if (clubs.TryAdd(address, clubCandidate))
                            {
                                diagnostics.ClubCandidatesAccepted++;
                            }
                            else
                            {
                                diagnostics.ClubCandidateDuplicatesSkipped++;
                            }
                        }
                        else
                        {
                            diagnostics.ClubCandidatesRejected++;
                        }

                        continue;
                    }
                    if (!TrySubtract(address, classOffset, out var blockAddress))
                    {
                        diagnostics.CandidatesRejected++;
                        continue;
                    }

                    var abilityOffsets = facet == PersonFacet.Player
                        ? (layout.CurrentAbilityOffset, layout.PotentialAbilityOffset)
                        : (layout.StaffCurrentAbilityOffset, layout.StaffPotentialAbilityOffset);
                    if (!TryReadAbilities(reader, blockAddress, abilityOffsets.Item1, abilityOffsets.Item2, out var ca, out var pa))
                    {
                        diagnostics.CandidatesRejected++;
                        continue;
                    }

                    var candidate = new PersonCandidate(address, blockAddress, uid, ca, pa, classOffset, facet);
                    if (facet == PersonFacet.Player)
                    {
                        var gender = PlayerGenderReader.Read(reader, address, layout);
                        if (!PlayerDatabaseScopes.Includes(playerDatabaseScope, gender))
                        {
                            diagnostics.PlayersExcludedByDatabaseScope++;
                            continue;
                        }

                        if (players.ContainsKey(uid))
                        {
                            KeepLowestAddress(players, candidate);
                            diagnostics.DuplicatesSkipped++;
                            continue;
                        }

                        if (atCap)
                        {
                            stoppedDueToCap = true;
                            break;
                        }

                        players[uid] = candidate;
                        diagnostics.CandidatesAccepted++;
                        if (diagnostics.SampleUids.Count < 16)
                        {
                            diagnostics.SampleUids.Add(uid);
                        }

                        if (maxAccepted is { } limit && players.Count >= limit)
                        {
                            atCap = true;
                        }

                        continue;
                    }

                    if (staff.ContainsKey(uid))
                    {
                        KeepLowestAddress(staff, candidate);
                        diagnostics.DuplicatesSkipped++;
                    }
                    else
                    {
                        staff[uid] = candidate;
                        diagnostics.StaffCandidatesAccepted++;
                    }

                    if (facet == PersonFacet.HumanManager)
                    {
                        if (managers.ContainsKey(uid))
                        {
                            KeepLowestAddress(managers, candidate);
                            diagnostics.DuplicatesSkipped++;
                        }
                        else
                        {
                            managers[uid] = candidate;
                            diagnostics.HumanManagerCandidatesAccepted++;
                        }
                    }
                }

                if (stoppedDueToCap)
                {
                    break;
                }

                var advance = (ulong)toRead > overlap
                    ? toRead - (int)overlap
                    : 8;
                if (advance < 8)
                {
                    advance = 8;
                }

                if (!TryAdd(blockStart, (ulong)advance, out blockStart))
                {
                    diagnostics.CandidatesRejected++;
                    break;
                }
            }
        }

        diagnostics.StoppedEarly = stoppedDueToCap;
        return BuildResult(players, staff, managers, clubs, diagnostics);
    }

    public static bool IsValidUid(uint uid) => uid != 0 && uid != InvalidUid;

    public static bool IsValidAbility(int value) => value is >= MinAbility and <= MaxAbility;

    private static PersonScanResult BuildResult(
        IReadOnlyDictionary<uint, PersonCandidate> players,
        IReadOnlyDictionary<uint, PersonCandidate> staff,
        IReadOnlyDictionary<uint, PersonCandidate> managers,
        IReadOnlyDictionary<ulong, ClubCandidate> clubs,
        ScanDiagnostics diagnostics)
    {
        var overlapUids = staff.Keys
            .Where(players.ContainsKey)
            .OrderBy(uid => uid)
            .ToList();
        diagnostics.PlayerStaffOverlapCount = overlapUids.Count;
        diagnostics.ClubDiscoveryIncomplete = diagnostics.StoppedEarly || diagnostics.Cancelled;

        return new PersonScanResult(
            OrderCandidates(players.Values),
            OrderCandidates(staff.Values.Where(candidate => !players.ContainsKey(candidate.Uid))),
            OrderCandidates(managers.Values),
            clubs.Values.OrderBy(candidate => candidate.Address).ToList(),
            overlapUids,
            diagnostics.StoppedEarly,
            diagnostics.Cancelled);
    }

    private static List<PersonCandidate> OrderCandidates(IEnumerable<PersonCandidate> candidates) =>
        candidates
            .OrderBy(candidate => candidate.Uid)
            .ThenBy(candidate => candidate.ObjectAddress)
            .ToList();

    private static void KeepLowestAddress(
        IDictionary<uint, PersonCandidate> candidates,
        PersonCandidate candidate)
    {
        if (candidates.TryGetValue(candidate.Uid, out var existing)
            && candidate.ObjectAddress < existing.ObjectAddress)
        {
            candidates[candidate.Uid] = candidate;
        }
    }

    private static bool TryGetFacet(
        int classOffset,
        ISet<int> playerOffsets,
        ISet<int> staffOffsets,
        ISet<int> managerOffsets,
        out PersonFacet facet)
    {
        if (playerOffsets.Contains(classOffset))
        {
            facet = PersonFacet.Player;
            return true;
        }

        if (staffOffsets.Contains(classOffset))
        {
            facet = PersonFacet.Staff;
            return true;
        }

        if (managerOffsets.Contains(classOffset))
        {
            facet = PersonFacet.HumanManager;
            return true;
        }

        facet = default;
        return false;
    }

    private static bool TryReadClubCandidate(
        IMemoryReader reader,
        IFmMemoryLayout layout,
        ulong address,
        out ClubCandidate candidate)
    {
        candidate = default;
        if (!TryReadPointerAt(reader, address, layout.ClubTeamsBeginOffset, out var teamsBegin)
            || !TryReadPointerAt(reader, address, layout.ClubTeamsEndOffset, out var teamsEnd)
            || teamsBegin == 0
            || teamsEnd <= teamsBegin
            || teamsBegin % sizeof(ulong) != 0
            || teamsEnd % sizeof(ulong) != 0
            || (teamsEnd - teamsBegin) % sizeof(ulong) != 0)
        {
            return false;
        }

        var teamCount = (teamsEnd - teamsBegin) / sizeof(ulong);
        if (teamCount is 0 or > SquadClubIndex.MaxTeamsPerClub)
        {
            return false;
        }

        var name = ClubNameReader.TryRead(reader, address, layout);
        if (name is null)
        {
            return false;
        }

        candidate = new ClubCandidate(address, name);
        return true;
    }

    private static bool TryReadAbilities(
        IMemoryReader reader,
        ulong blockAddress,
        int currentAbilityOffset,
        int potentialAbilityOffset,
        out int ca,
        out int pa)
    {
        ca = 0;
        pa = 0;
        if (!TryAdd(blockAddress, currentAbilityOffset, out var currentAbilityAddress)
            || !reader.TryReadUInt16(currentAbilityAddress, out var currentAbility)
            || !IsValidAbility(currentAbility)
            || !TryAdd(blockAddress, potentialAbilityOffset, out var potentialAbilityAddress)
            || !reader.TryReadUInt16(potentialAbilityAddress, out var potentialAbility)
            || !IsValidAbility(potentialAbility))
        {
            return false;
        }

        ca = currentAbility;
        pa = potentialAbility;
        return true;
    }

    private static bool TryReadPointerAt(
        IMemoryReader reader,
        ulong address,
        int offset,
        out ulong value)
    {
        value = 0;
        return TryAdd(address, offset, out var fieldAddress)
            && reader.TryReadUInt64(fieldAddress, out value);
    }

    private static bool TrySubtract(ulong address, int offset, out ulong result)
    {
        result = 0;
        if (offset < 0 || (ulong)offset > address)
        {
            return false;
        }

        result = address - (ulong)offset;
        return true;
    }

    private static bool TryAdd(ulong address, int offset, out ulong result)
    {
        result = 0;
        return offset >= 0 && TryAdd(address, (ulong)offset, out result);
    }

    private static bool TryAdd(ulong address, ulong offset, out ulong result)
    {
        result = 0;
        if (offset > ulong.MaxValue - address)
        {
            return false;
        }

        result = address + offset;
        return true;
    }

    private static bool TryAlignUp(ulong value, ulong alignment, out ulong aligned)
    {
        aligned = 0;
        var adjustment = alignment - 1;
        if (value > ulong.MaxValue - adjustment)
        {
            return false;
        }

        aligned = (value + adjustment) & ~adjustment;
        return true;
    }

    /// <summary>
    /// Il2Cpp: meta = *(vtable - 8); dynamic class offset = *(int*)(meta + 4).
    /// </summary>
    public static bool TryResolveDynamicOffset(IMemoryReader reader, ulong vtable, out int classOffset)
    {
        classOffset = 0;
        if (vtable < 8)
        {
            return false;
        }

        if (!reader.TryReadUInt64(vtable - 8, out var meta) || meta == 0)
        {
            return false;
        }

        return TryAdd(meta, sizeof(int), out var classOffsetAddress)
            && reader.TryReadInt32(classOffsetAddress, out classOffset);
    }

    private static bool TryResolveDynamicOffsetCached(
        IMemoryReader reader,
        ulong vtable,
        Dictionary<ulong, int> cache,
        out int classOffset)
    {
        if (cache.TryGetValue(vtable, out classOffset))
        {
            return classOffset != 0;
        }

        if (!TryResolveDynamicOffset(reader, vtable, out classOffset) || classOffset == 0)
        {
            cache[vtable] = 0;
            classOffset = 0;
            return false;
        }

        cache[vtable] = classOffset;
        return true;
    }

    private static bool IsModuleVtable(
        ulong vtable,
        ModuleBounds gameAssembly,
        ModuleBounds? gamePlugin)
    {
        if (vtable >= gameAssembly.BaseAddress && vtable < gameAssembly.EndAddress)
        {
            return true;
        }

        return gamePlugin is { } gp
            && vtable >= gp.BaseAddress
            && vtable < gp.EndAddress;
    }

}
