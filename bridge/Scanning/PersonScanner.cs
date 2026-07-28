using FmDataBridge.Layouts;
using FmDataBridge.Memory;

namespace FmDataBridge.Scanning;

public static class PersonScanner
{
    public const uint InvalidUid = 0xFFFFFFFFu;
    public const int MinAbility = 1;
    public const int MaxAbility = 200;

    // ponytail: hard cap so live Load Data tests finish in seconds, not minutes
    // Upgrade to unlimited (or request-driven maxPlayers) when full-DB dumps are required — see BACKLOG High "Bridge scan performance"
    public const int DefaultMaxAccepted = 10_000;

    public static IReadOnlyList<PersonCandidate> Scan(
        IMemoryReader reader,
        IFmMemoryLayout layout,
        ModuleBounds gameAssembly,
        ModuleBounds? gamePlugin,
        IReadOnlyList<MemoryRegion> candidateRegions,
        ScanDiagnostics diagnostics,
        int? maxAccepted = DefaultMaxAccepted)
    {
        ArgumentNullException.ThrowIfNull(reader);
        ArgumentNullException.ThrowIfNull(layout);
        ArgumentNullException.ThrowIfNull(candidateRegions);
        ArgumentNullException.ThrowIfNull(diagnostics);
        if (maxAccepted is <= 0)
        {
            throw new ArgumentOutOfRangeException(nameof(maxAccepted), maxAccepted, "maxAccepted must be null or positive.");
        }

        diagnostics.RegionCount = candidateRegions.Count;
        diagnostics.LayoutVersionKey = layout.VersionKey;
        diagnostics.LayoutProvisional = layout.IsProvisional;
        diagnostics.MaxAccepted = maxAccepted;
        diagnostics.GameAssembly = new ModuleBoundsSnapshot(
            gameAssembly.BaseAddress,
            gameAssembly.EndAddress);
        if (gamePlugin is { } gp)
        {
            diagnostics.GamePlugin = new ModuleBoundsSnapshot(gp.BaseAddress, gp.EndAddress);
        }

        var playerOffsets = new HashSet<int>(layout.PlayerClassOffsets);
        var accepted = new Dictionary<uint, PersonCandidate>();
        var stopEarly = false;

        foreach (var region in candidateRegions)
        {
            if (stopEarly)
            {
                break;
            }

            if (region.Size < 0x10)
            {
                continue;
            }

            var end = region.BaseAddress + region.Size;
            for (var address = AlignUp(region.BaseAddress, 8);
                 address + 0x10 <= end;
                 address += 8)
            {
                diagnostics.BytesScanned += 8;

                if (!reader.TryReadUInt64(address, out var vtable))
                {
                    continue;
                }

                if (!IsModuleVtable(vtable, gameAssembly, gamePlugin))
                {
                    continue;
                }

                diagnostics.VtableHits++;

                if (!TryResolveDynamicOffset(reader, vtable, out var classOffset) || classOffset == 0)
                {
                    diagnostics.CandidatesRejected++;
                    continue;
                }

                if (classOffset is > 0 and < 0x2000)
                {
                    diagnostics.RecordClassOffsetHit(classOffset);
                }

                if (!playerOffsets.Contains(classOffset))
                {
                    diagnostics.CandidatesRejected++;
                    continue;
                }

                if (!reader.TryReadUInt32(address + (ulong)layout.ObjectUidOffset, out var uid)
                    || !IsValidUid(uid))
                {
                    diagnostics.CandidatesRejected++;
                    continue;
                }

                if ((ulong)classOffset > address)
                {
                    diagnostics.CandidatesRejected++;
                    continue;
                }

                var playerBase = address - (ulong)classOffset;
                if (!reader.TryReadUInt16(playerBase + (ulong)layout.CurrentAbilityOffset, out var ca)
                    || !IsValidAbility(ca))
                {
                    diagnostics.CandidatesRejected++;
                    continue;
                }

                if (!reader.TryReadUInt16(playerBase + (ulong)layout.PotentialAbilityOffset, out var pa)
                    || !IsValidAbility(pa))
                {
                    diagnostics.CandidatesRejected++;
                    continue;
                }

                if (accepted.ContainsKey(uid))
                {
                    diagnostics.DuplicatesSkipped++;
                    continue;
                }

                accepted[uid] = new PersonCandidate(address, uid, ca, pa, classOffset);
                diagnostics.CandidatesAccepted++;
                if (diagnostics.SampleUids.Count < 16)
                {
                    diagnostics.SampleUids.Add(uid);
                }

                if (maxAccepted is { } limit && accepted.Count >= limit)
                {
                    diagnostics.StoppedEarly = true;
                    stopEarly = true;
                    break;
                }
            }
        }

        return accepted.Values.OrderBy(c => c.Uid).ToList();
    }

    public static bool IsValidUid(uint uid) => uid != 0 && uid != InvalidUid;

    public static bool IsValidAbility(int value) => value is >= MinAbility and <= MaxAbility;

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

        return reader.TryReadInt32(meta + 4, out classOffset);
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

    private static ulong AlignUp(ulong value, ulong alignment) =>
        (value + (alignment - 1)) & ~(alignment - 1);
}
