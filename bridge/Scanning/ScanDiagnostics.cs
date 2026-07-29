namespace FmDataBridge.Scanning;

public readonly record struct PersonCandidate(ulong ObjectAddress, uint Uid, int Ca, int Pa, int ClassOffset);

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

    /// <summary>Accepted-player ceiling for this run; null means unlimited.</summary>
    public int? MaxAccepted { get; set; }

    /// <summary>True when the scanner stopped because <see cref="MaxAccepted"/> was reached.</summary>
    public bool StoppedEarly { get; set; }

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
