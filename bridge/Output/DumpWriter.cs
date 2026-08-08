using System.Text;
using System.Text.Json;
using FmDataBridge.Models;
using FmDataBridge.Protocol;

namespace FmDataBridge.Output;

public static class DumpWriter
{
    private static readonly JsonSerializerOptions SerializerOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        WriteIndented = false,
    };

    private static readonly JsonWriterOptions WriterOptions = new()
    {
        Indented = false,
    };

    /// <summary>
    /// Writes <paramref name="document"/> only when it contains at least one player.
    /// Never replaces an existing dump with an empty or failed result.
    /// </summary>
    /// <returns>True when the dump file was replaced.</returns>
    public static bool TryWriteReplaceOnSuccess(string bridgeDirectory, DumpDocument document)
    {
        ArgumentNullException.ThrowIfNull(document);
        Directory.CreateDirectory(bridgeDirectory);

        if (document.Players.Count == 0 || document.PlayerCount == 0)
        {
            return false;
        }

        var path = BridgePaths.GetDumpPath(bridgeDirectory);
        var tempPath = path + ".tmp";

        using (var stream = File.Create(tempPath))
        {
            WriteCompact(stream, document);
        }

        File.Move(tempPath, path, overwrite: true);
        return true;
    }

    /// <summary>
    /// Streams compact schema-v5 dump JSON to <paramref name="stream"/> without building a second full document string.
    /// </summary>
    public static void WriteCompact(Stream stream, DumpDocument document)
    {
        ArgumentNullException.ThrowIfNull(stream);
        ArgumentNullException.ThrowIfNull(document);

        using var writer = new Utf8JsonWriter(stream, WriterOptions);
        writer.WriteStartObject();
        writer.WriteNumber("schemaVersion", document.SchemaVersion);
        writer.WriteString("generatedAtUtc", document.GeneratedAtUtc);
        writer.WriteString("gameVersion", document.GameVersion);
        writer.WriteString("supportedGameVersion", document.SupportedGameVersion);
        writer.WriteString("bridgeVersion", document.BridgeVersion);
        writer.WriteNumber("protocolVersion", document.ProtocolVersion);
        if (document.GameDate is null)
        {
            writer.WriteNull("gameDate");
        }
        else
        {
            writer.WriteString("gameDate", document.GameDate);
        }

        writer.WriteString("gameDateSource", document.GameDateSource);
        writer.WriteBoolean("scanTruncated", document.ScanTruncated);
        if (document.MaxAccepted is { } maxAccepted)
        {
            writer.WriteNumber("maxAccepted", maxAccepted);
        }
        else
        {
            writer.WriteNull("maxAccepted");
        }

        writer.WriteNumber("playerCount", document.PlayerCount);
        writer.WritePropertyName("players");
        writer.WriteStartArray();
        foreach (var player in document.Players)
        {
            JsonSerializer.Serialize(writer, player, SerializerOptions);
            writer.Flush();
        }

        writer.WriteEndArray();
        writer.WriteEndObject();
        writer.Flush();
    }

    public static string Serialize(DumpDocument document)
    {
        ArgumentNullException.ThrowIfNull(document);
        using var stream = new MemoryStream();
        WriteCompact(stream, document);
        return Encoding.UTF8.GetString(stream.GetBuffer(), 0, (int)stream.Length);
    }
}

public static class DiagnosticsWriter
{
    public static void Write(string bridgeDirectory, string contents)
    {
        Directory.CreateDirectory(bridgeDirectory);
        var path = BridgePaths.GetDiagnosticsPath(bridgeDirectory);
        var tempPath = path + ".tmp";
        File.WriteAllText(tempPath, contents, Encoding.UTF8);
        File.Move(tempPath, path, overwrite: true);
    }

    public static string Format(Scanning.ScanDiagnostics diagnostics)
    {
        ArgumentNullException.ThrowIfNull(diagnostics);
        var sb = new StringBuilder();
        sb.AppendLine("FM ValueScout bridge diagnostics");
        sb.AppendLine($"generatedAtUtc={DateTimeOffset.UtcNow:O}");
        sb.AppendLine($"gameVersion={diagnostics.GameVersion}");
        sb.AppendLine($"layoutVersionKey={diagnostics.LayoutVersionKey ?? "(none)"}");
        sb.AppendLine($"layoutProvisional={diagnostics.LayoutProvisional}");
        if (!string.IsNullOrEmpty(diagnostics.FailureReason))
        {
            sb.AppendLine($"failureReason={diagnostics.FailureReason}");
        }

        sb.AppendLine($"regionCount={diagnostics.RegionCount}");
        sb.AppendLine($"bytesScanned={diagnostics.BytesScanned}");
        sb.AppendLine($"vtableHits={diagnostics.VtableHits}");
        sb.AppendLine($"candidatesAccepted={diagnostics.CandidatesAccepted}");
        sb.AppendLine($"staffCandidatesAccepted={diagnostics.StaffCandidatesAccepted}");
        sb.AppendLine($"humanManagerCandidatesAccepted={diagnostics.HumanManagerCandidatesAccepted}");
        sb.AppendLine($"playerStaffOverlapCount={diagnostics.PlayerStaffOverlapCount}");
        sb.AppendLine($"candidatesRejected={diagnostics.CandidatesRejected}");
        sb.AppendLine($"duplicatesSkipped={diagnostics.DuplicatesSkipped}");
        sb.AppendLine($"identitySkippedEmptyName={diagnostics.IdentitySkippedEmptyName}");
        sb.AppendLine($"identitySkippedImpossibleDob={diagnostics.IdentitySkippedImpossibleDob}");
        sb.AppendLine($"regionEnumerationMs={diagnostics.RegionEnumerationMs}");
        sb.AppendLine($"candidateDiscoveryMs={diagnostics.CandidateDiscoveryMs}");
        sb.AppendLine($"extractionMs={diagnostics.ExtractionMs}");
        sb.AppendLine($"clubIndexingMs={diagnostics.ClubIndexingMs}");
        sb.AppendLine($"dumpWritingMs={diagnostics.DumpWritingMs}");
        sb.AppendLine($"totalMs={diagnostics.TotalMs}");
        sb.AppendLine($"processMemoryCalls={diagnostics.ProcessMemoryCalls}");
        sb.AppendLine($"processMemoryRequestedBytes={diagnostics.ProcessMemoryRequestedBytes}");
        if (diagnostics.MaxAccepted is { } maxAccepted)
        {
            sb.AppendLine($"maxAccepted={maxAccepted}");
        }

        sb.AppendLine($"stoppedEarly={diagnostics.StoppedEarly}");

        if (diagnostics.GameAssembly is { } ga)
        {
            sb.AppendLine($"gameAssembly=0x{ga.BaseAddress:X}-0x{ga.EndAddress:X}");
        }

        if (diagnostics.GamePlugin is { } gp)
        {
            sb.AppendLine($"gamePlugin=0x{gp.BaseAddress:X}-0x{gp.EndAddress:X}");
        }

        sb.AppendLine("classOffsetHistogram:");
        foreach (var pair in diagnostics.ClassOffsetHistogram.OrderBy(p => p.Key))
        {
            sb.AppendLine($"  0x{pair.Key:X}={pair.Value}");
        }

        sb.AppendLine("sampleUids:");
        foreach (var uid in diagnostics.SampleUids)
        {
            sb.AppendLine($"  {uid}");
        }

        sb.AppendLine("attrsStoredTimesFive=decode floor(raw/5+0.5); null if unread or outside 1..20");
        sb.AppendLine("personalityRaw=1..20 or null");
        sb.AppendLine("contractNull=free agent or unread; money 0xFFFFFFFF/300M → null");
        sb.AppendLine(
            "clubNull=unresolved/free agent; onLoan when current≠parent; teamLevel senior/reserve/youth");
        sb.AppendLine($"clubsWalked={diagnostics.ClubsWalked}");
        sb.AppendLine($"playersLinkedViaSquad={diagnostics.PlayersLinkedViaSquad}");
        sb.AppendLine($"clubUnresolved={diagnostics.ClubUnresolved}");
        if (!string.IsNullOrEmpty(diagnostics.ClubResolutionWarning))
        {
            sb.AppendLine($"clubResolutionWarning={diagnostics.ClubResolutionWarning}");
        }

        sb.AppendLine($"gameDate={diagnostics.GameDate ?? "(none)"}");
        sb.AppendLine($"gameDateSource={diagnostics.GameDateSource ?? "unknown"}");
        sb.AppendLine("sampleAttributes:");
        foreach (var sample in diagnostics.SampleAttributeSnapshots)
        {
            sb.AppendLine($"  {sample}");
        }

        sb.AppendLine("sampleContracts:");
        foreach (var sample in diagnostics.SampleContractSnapshots)
        {
            sb.AppendLine($"  {sample}");
        }

        sb.AppendLine("sampleClubs:");
        foreach (var sample in diagnostics.SampleClubSnapshots)
        {
            sb.AppendLine($"  {sample}");
        }

        sb.AppendLine("multiClubSamples:");
        foreach (var sample in diagnostics.MultiClubSamples)
        {
            sb.AppendLine($"  {sample}");
        }

        if (!string.IsNullOrEmpty(diagnostics.FailureReason)
            && diagnostics.FailureReason.Contains("unsupported", StringComparison.OrdinalIgnoreCase))
        {
            sb.AppendLine(
                "hint=Add or update a layout under bridge/Layouts for this FM major.minor, then rebuild the plugin.");
        }
        else if (diagnostics.CandidatesAccepted == 0 && diagnostics.LayoutProvisional)
        {
            sb.AppendLine(
                "hint=Layout is provisional. Repin UID/CA/PA offsets using the class-offset histogram and known players.");
        }
        else if (!string.IsNullOrEmpty(diagnostics.ClubResolutionWarning))
        {
            sb.AppendLine(
                "hint=Many players lack club links. Confirm contract→team→club offsets and squad-array pins.");
        }

        return sb.ToString();
    }
}
