namespace FmDataBridge.Scanning;

public sealed class ScanDiagnostics
{
    public string GameVersion { get; set; } = "";

    public string? LayoutVersionKey { get; set; }

    public bool LayoutProvisional { get; set; }

    public string? FailureReason { get; set; }

    public int RegionCount { get; set; }

    public long BytesScanned { get; set; }

    public int VtableHits { get; set; }

    public int CandidatesAccepted { get; set; }

    public int StaffCandidatesAccepted { get; set; }

    public int HumanManagerCandidatesAccepted { get; set; }

    public int PlayerStaffOverlapCount { get; set; }

    public int CandidatesRejected { get; set; }

    public int DuplicatesSkipped { get; set; }

    /// <summary>Candidates skipped because display name was empty.</summary>
    public int IdentitySkippedEmptyName { get; set; }

    /// <summary>Candidates skipped because DOB was missing or impossible.</summary>
    public int IdentitySkippedImpossibleDob { get; set; }

    public const int MaxSampleAttributeSnapshots = 5;

    /// <summary>Short attribute snapshots for known-player patch verification.</summary>
    public List<string> SampleAttributeSnapshots { get; } = new();

    public const int MaxSampleContractSnapshots = 5;

    /// <summary>Short contract/value snapshots for known-player patch verification.</summary>
    public List<string> SampleContractSnapshots { get; } = new();

    public const int MaxSampleClubSnapshots = 5;

    /// <summary>Short club/loan snapshots for known-player patch verification.</summary>
    public List<string> SampleClubSnapshots { get; } = new();

    /// <summary>Multi-club (loan) conflict samples from squad walk.</summary>
    public List<string> MultiClubSamples { get; } = new();

    public int ClubsWalked { get; set; }

    public int PlayersLinkedViaSquad { get; set; }

    public int ClubUnresolved { get; set; }

    public string? ClubResolutionWarning { get; set; }

    public string? GameDate { get; set; }

    public string? GameDateSource { get; set; }

    /// <summary>Accepted-player ceiling for this run; null means unlimited.</summary>
    public int? MaxAccepted { get; set; }

    /// <summary>True when the scanner stopped because <see cref="MaxAccepted"/> was reached.</summary>
    public bool StoppedEarly { get; set; }

    /// <summary>True when the walk was cancelled via <see cref="CancellationToken"/>.</summary>
    public bool Cancelled { get; set; }

    public long RegionEnumerationMs { get; set; }

    public long CandidateDiscoveryMs { get; set; }

    public long ExtractionMs { get; set; }

    public long ClubIndexingMs { get; set; }

    public long DumpWritingMs { get; set; }

    public long TotalMs { get; set; }

    public long ProcessMemoryCalls { get; set; }

    public long ProcessMemoryRequestedBytes { get; set; }

    public Dictionary<int, int> ClassOffsetHistogram { get; } = new();

    public List<uint> SampleUids { get; } = new();

    public ModuleBoundsSnapshot? GamePlugin { get; set; }

    public ModuleBoundsSnapshot? GameAssembly { get; set; }

    public void RecordClassOffsetHit(int classOffset)
    {
        ClassOffsetHistogram.TryGetValue(classOffset, out var count);
        ClassOffsetHistogram[classOffset] = count + 1;
    }
}

public readonly record struct ModuleBoundsSnapshot(ulong BaseAddress, ulong EndAddress);
