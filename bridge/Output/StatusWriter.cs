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

    public static string Serialize(BridgeStatus status)
    {
        ArgumentNullException.ThrowIfNull(status);
        return JsonSerializer.Serialize(SanitizeError(status), SerializerOptions);
    }

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

    private static BridgeStatus SanitizeError(BridgeStatus status)
    {
        if (status.Error is not { } error || !ContainsMachineLocalPath(error))
        {
            return status;
        }

        return new BridgeStatus
        {
            ProtocolVersion = status.ProtocolVersion,
            PluginVersion = status.PluginVersion,
            State = status.State,
            UpdatedAtUtc = status.UpdatedAtUtc,
            GamePluginModulePresent = status.GamePluginModulePresent,
            GameAssemblyModulePresent = status.GameAssemblyModulePresent,
            RequestId = status.RequestId,
            PlayersFound = status.PlayersFound,
            Error = "scan failed unexpectedly",
            ScanTruncated = status.ScanTruncated,
            MaxAccepted = status.MaxAccepted,
            PlayerBoostsSupported = status.PlayerBoostsSupported,
            StaffBoostsSupported = status.StaffBoostsSupported,
            PlayerBoost = status.PlayerBoost,
            StaffBoost = status.StaffBoost,
        };
    }

    private static bool ContainsMachineLocalPath(string error)
    {
        for (var index = 0; index + 2 < error.Length; index++)
        {
            if (char.IsLetter(error[index])
                && error[index + 1] == ':'
                && (error[index + 2] == '\\' || error[index + 2] == '/'))
            {
                return true;
            }
        }

        return error.Contains("\\\\", StringComparison.Ordinal)
            || error.Contains("/home/", StringComparison.Ordinal)
            || error.Contains("/Users/", StringComparison.Ordinal)
            || error.Contains("/tmp/", StringComparison.Ordinal);
    }
}
