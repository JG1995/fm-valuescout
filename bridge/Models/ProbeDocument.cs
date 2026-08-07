namespace FmDataBridge.Models;

/// <summary>
/// Schema-v1 artifact for developer-only, UID-scoped FM memory research.
/// </summary>
public sealed class ProbeDocument
{
    public int SchemaVersion { get; init; }

    public string GeneratedAtUtc { get; init; } = "";

    public string GameVersion { get; init; } = "";

    public string SupportedGameVersion { get; init; } = "";

    public string BridgeVersion { get; init; } = "";

    public int ProtocolVersion { get; init; }

    public string RequestId { get; init; } = "";

    public IReadOnlyList<uint> RequestedUids { get; init; } = Array.Empty<uint>();

    public ProbeModule? GameAssembly { get; init; }

    public ProbeModule? GamePlugin { get; init; }

    public int PlayerCount { get; init; }

    public IReadOnlyList<ProbePlayer> Players { get; init; } = Array.Empty<ProbePlayer>();
}

public sealed class ProbeModule
{
    public string Name { get; init; } = "";

    public ulong BaseAddress { get; init; }

    public ulong EndAddress { get; init; }
}

public sealed class ProbePlayer
{
    public uint Uid { get; init; }

    public ulong CandidateAddress { get; init; }

    public int ClassOffset { get; init; }

    public ulong PlayerBlockAddress { get; init; }

    public int RequestedBytes { get; init; }

    public int ReadableBytes { get; init; }

    public IReadOnlyList<ProbeMemoryRange> Ranges { get; init; } = Array.Empty<ProbeMemoryRange>();
}

public sealed class ProbeMemoryRange
{
    public string AddressBasis { get; init; } = "";

    public string RelativePath { get; init; } = "";

    public string? SourcePointerPath { get; init; }

    public ulong Address { get; init; }

    public int RequestedLength { get; init; }

    public int PointerDepth { get; init; }

    public IReadOnlyList<ProbeReadableSpan> ReadableSpans { get; init; } = Array.Empty<ProbeReadableSpan>();
}

public sealed class ProbeReadableSpan
{
    public int Offset { get; init; }

    public string BytesBase64 { get; init; } = "";
}
