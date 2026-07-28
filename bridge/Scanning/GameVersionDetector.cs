using System.Diagnostics;
using FmDataBridge.Memory;

namespace FmDataBridge.Scanning;

public static class GameVersionDetector
{
    /// <summary>
    /// Best-effort FileVersionInfo from a loaded module path. Returns false when unavailable.
    /// </summary>
    public static bool TryDetectFromModules(
        IEnumerable<ProcessModuleInfo> modules,
        Func<string, string?>? resolveFilePath,
        out string gameVersion)
    {
        ArgumentNullException.ThrowIfNull(modules);
        gameVersion = "";

        foreach (var module in modules)
        {
            if (!string.Equals(
                    module.ModuleName,
                    ModuleLocator.GamePluginModuleName,
                    StringComparison.OrdinalIgnoreCase))
            {
                continue;
            }

            var path = resolveFilePath?.Invoke(module.ModuleName);
            if (string.IsNullOrWhiteSpace(path) || !File.Exists(path))
            {
                return false;
            }

            var info = FileVersionInfo.GetVersionInfo(path);
            var raw = info.FileVersion ?? info.ProductVersion;
            if (string.IsNullOrWhiteSpace(raw))
            {
                return false;
            }

            gameVersion = raw.Trim();
            return true;
        }

        return false;
    }

    public static bool TryDetectFromCurrentProcess(out string gameVersion)
    {
        gameVersion = "";
        try
        {
            foreach (ProcessModule module in Process.GetCurrentProcess().Modules)
            {
                if (!string.Equals(
                        module.ModuleName,
                        ModuleLocator.GamePluginModuleName,
                        StringComparison.OrdinalIgnoreCase))
                {
                    continue;
                }

                if (string.IsNullOrWhiteSpace(module.FileName) || !File.Exists(module.FileName))
                {
                    return false;
                }

                var info = FileVersionInfo.GetVersionInfo(module.FileName);
                var raw = info.FileVersion ?? info.ProductVersion;
                if (string.IsNullOrWhiteSpace(raw))
                {
                    return false;
                }

                gameVersion = raw.Trim();
                return true;
            }
        }
        catch
        {
            return false;
        }

        return false;
    }
}
