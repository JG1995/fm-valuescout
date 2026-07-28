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
        WriteIndented = true,
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
            // ponytail: serialize full player list in one shot
            // Upgrade to Utf8JsonWriter streaming if dumps exceed ~100k players or memory spikes
            JsonSerializer.Serialize(stream, document, SerializerOptions);
        }

        File.Move(tempPath, path, overwrite: true);
        return true;
    }

    public static string Serialize(DumpDocument document) =>
        JsonSerializer.Serialize(document, SerializerOptions);
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
        sb.AppendLine($"candidatesRejected={diagnostics.CandidatesRejected}");
        sb.AppendLine($"duplicatesSkipped={diagnostics.DuplicatesSkipped}");
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

        return sb.ToString();
    }
}
