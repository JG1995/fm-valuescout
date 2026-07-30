using System.Buffers.Binary;
using FmDataBridge.Layouts;
using FmDataBridge.Memory;

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

    public static IReadOnlyList<PersonCandidate> Scan(
        IMemoryReader reader,
        IFmMemoryLayout layout,
        ModuleBounds gameAssembly,
        ModuleBounds? gamePlugin,
        IReadOnlyList<MemoryRegion> candidateRegions,
        ScanDiagnostics diagnostics,
        int? maxAccepted = null,
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
        var classOffsetByVtable = new Dictionary<ulong, int>();
        var atCap = false;
        var stoppedDueToCap = false;
        var headerBytes = Math.Max(MinObjectHeaderBytes, layout.ObjectUidOffset + sizeof(uint));
        var overlap = (ulong)AlignUp((uint)headerBytes, 8);
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

            var end = region.BaseAddress + region.Size;
            for (var blockStart = AlignUp(region.BaseAddress, 8);
                 blockStart + (ulong)headerBytes <= end;
                 )
            {
                if (stoppedDueToCap)
                {
                    break;
                }

                if (cancellationToken.IsCancellationRequested)
                {
                    diagnostics.Cancelled = true;
                    return accepted.Values.OrderBy(c => c.Uid).ToList();
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
                        return accepted.Values.OrderBy(c => c.Uid).ToList();
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

                    if (!playerOffsets.Contains(classOffset))
                    {
                        diagnostics.CandidatesRejected++;
                        continue;
                    }

                    var uid = BinaryPrimitives.ReadUInt32LittleEndian(
                        buffer.AsSpan(local + layout.ObjectUidOffset, sizeof(uint)));
                    if (!IsValidUid(uid))
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

                    if (atCap)
                    {
                        stoppedDueToCap = true;
                        break;
                    }

                    accepted[uid] = new PersonCandidate(address, uid, ca, pa, classOffset);
                    diagnostics.CandidatesAccepted++;
                    if (diagnostics.SampleUids.Count < 16)
                    {
                        diagnostics.SampleUids.Add(uid);
                    }

                    if (maxAccepted is { } limit && accepted.Count >= limit)
                    {
                        atCap = true;
                    }
                }

                if (stoppedDueToCap)
                {
                    break;
                }

                var advance = toRead - (int)overlap;
                if (advance < 8)
                {
                    advance = 8;
                }

                blockStart += (ulong)advance;
            }
        }

        if (diagnostics.Cancelled)
        {
            return accepted.Values.OrderBy(c => c.Uid).ToList();
        }

        diagnostics.StoppedEarly = stoppedDueToCap;
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

    private static ulong AlignUp(ulong value, ulong alignment) =>
        (value + (alignment - 1)) & ~(alignment - 1);

    private static uint AlignUp(uint value, uint alignment) =>
        (value + (alignment - 1)) & ~(alignment - 1);
}
