using System.Text.Json;
using FmDataBridge.Models;
using FmDataBridge.Protocol;

namespace FmDataBridge.Output;

public static class ProbeWriter
{
    private static readonly JsonSerializerOptions SerializerOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        WriteIndented = false,
    };

    /// <summary>
    /// Replaces <c>probe.json</c> only after a complete, non-empty capture serializes successfully.
    /// </summary>
    public static bool TryWriteReplaceOnSuccess(string bridgeDirectory, ProbeDocument document)
    {
        ArgumentNullException.ThrowIfNull(document);
        Directory.CreateDirectory(bridgeDirectory);

        if (document.PlayerCount == 0 || document.Players.Count == 0)
        {
            return false;
        }

        var path = BridgePaths.GetProbePath(bridgeDirectory);
        var tempPath = path + ".tmp";
        try
        {
            File.WriteAllText(tempPath, JsonSerializer.Serialize(document, SerializerOptions));
            File.Move(tempPath, path, overwrite: true);
            return true;
        }
        finally
        {
            TryDeleteTemp(tempPath);
        }
    }

    private static void TryDeleteTemp(string tempPath)
    {
        try
        {
            if (File.Exists(tempPath))
            {
                File.Delete(tempPath);
            }
        }
        catch
        {
            // A stale temporary file cannot replace a successful probe file.
        }
    }
}

public static class ProbeStatusWriter
{
    private static readonly JsonSerializerOptions SerializerOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        WriteIndented = true,
    };

    public static void Write(string bridgeDirectory, ProbeStatus status)
    {
        ArgumentNullException.ThrowIfNull(status);
        Directory.CreateDirectory(bridgeDirectory);

        var path = BridgePaths.GetProbeStatusPath(bridgeDirectory);
        var tempPath = path + ".tmp";
        try
        {
            File.WriteAllText(tempPath, JsonSerializer.Serialize(status, SerializerOptions));
            File.Move(tempPath, path, overwrite: true);
        }
        finally
        {
            TryDeleteTemp(tempPath);
        }
    }

    private static void TryDeleteTemp(string tempPath)
    {
        try
        {
            if (File.Exists(tempPath))
            {
                File.Delete(tempPath);
            }
        }
        catch
        {
            // A stale temporary file cannot replace a successful probe or status file.
        }
    }
}
