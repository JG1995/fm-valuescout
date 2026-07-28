using System.Diagnostics;
using FmDataBridge.Memory;

namespace FmDataBridge.Output;

public readonly record struct ModulePresenceSignals(
    bool GamePluginModulePresent,
    bool GameAssemblyModulePresent);

public static class ModulePresence
{
    public static ModulePresenceSignals Detect()
    {
        var names = new List<string>();
        foreach (ProcessModule module in Process.GetCurrentProcess().Modules)
        {
            if (module.ModuleName is { } name)
            {
                names.Add(name);
            }
        }

        return DetectFromModuleNames(names);
    }

    public static ModulePresenceSignals DetectFromModuleNames(IEnumerable<string> moduleNames)
    {
        var set = new HashSet<string>(moduleNames, StringComparer.OrdinalIgnoreCase);
        return new ModulePresenceSignals(
            set.Contains(ModuleLocator.GamePluginModuleName),
            set.Contains(ModuleLocator.GameAssemblyModuleName));
    }

    /// <summary>
    /// Presence flags from the same locator used during a scan (authoritative once modules are loaded).
    /// </summary>
    public static ModulePresenceSignals FromBounds(ModulePresenceBounds known) =>
        new(known.GamePlugin is not null, known.GameAssembly is not null);
}
