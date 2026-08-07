namespace FmDataBridge.Protocol;

/// <summary>
/// Versioned request for a bounded capture of explicitly selected player UIDs.
/// </summary>
public sealed class ProbeRequest
{
    public int ProtocolVersion { get; init; }

    public string RequestId { get; init; } = "";

    public DateTimeOffset CreatedAtUtc { get; init; }

    public uint[] Uids { get; init; } = Array.Empty<uint>();
}
