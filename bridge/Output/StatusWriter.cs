using System.Text.Json;
using FmDataBridge.Protocol;

namespace FmDataBridge.Output;

public static class StatusWriter
{
    private static readonly JsonSerializerOptions SerializerOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        WriteIndented = true,
    };

    public static string Serialize(BridgeStatus status) =>
        JsonSerializer.Serialize(status, SerializerOptions);

    public static bool TryRead(string bridgeDirectory, out BridgeStatus? status)
    {
        status = null;
        var path = BridgePaths.GetStatusPath(bridgeDirectory);
        if (!File.Exists(path))
        {
            return false;
        }

        try
        {
            status = JsonSerializer.Deserialize<BridgeStatus>(File.ReadAllText(path), SerializerOptions);
            return status is not null;
        }
        catch (JsonException)
        {
            return false;
        }
    }

    public static void Write(string bridgeDirectory, BridgeStatus status)
    {
        Directory.CreateDirectory(bridgeDirectory);
        var path = BridgePaths.GetStatusPath(bridgeDirectory);
        var tempPath = path + ".tmp";
        File.WriteAllText(tempPath, Serialize(status));
        File.Move(tempPath, path, overwrite: true);
    }
}
