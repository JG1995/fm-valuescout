namespace FmDataBridge.Memory;

public readonly record struct ProcessModuleInfo(string ModuleName, ulong BaseAddress, ulong Size);

public readonly record struct ModuleBounds(string ModuleName, ulong BaseAddress, ulong EndAddress);

public readonly record struct ModulePresenceBounds(
    ModuleBounds? GamePlugin,
    ModuleBounds? GameAssembly);

public static class ModuleLocator
{
    public const string GamePluginModuleName = "game_plugin.dll";
    public const string GameAssemblyModuleName = "GameAssembly.dll";

    public static bool TryFind(
        IEnumerable<ProcessModuleInfo> modules,
        string moduleName,
        out ModuleBounds bounds)
    {
        ArgumentNullException.ThrowIfNull(modules);
        if (string.IsNullOrEmpty(moduleName))
        {
            throw new ArgumentException("Module name is required.", nameof(moduleName));
        }

        foreach (var module in modules)
        {
            if (!string.Equals(module.ModuleName, moduleName, StringComparison.OrdinalIgnoreCase))
            {
                continue;
            }

            bounds = new ModuleBounds(
                module.ModuleName,
                module.BaseAddress,
                module.BaseAddress + module.Size);
            return true;
        }

        bounds = default;
        return false;
    }

    /// <summary>
    /// Locates <see cref="GamePluginModuleName"/> and <see cref="GameAssemblyModuleName"/>
    /// from a process module list (e.g. current process while hosted in FM).
    /// </summary>
    public static ModulePresenceBounds LocateKnownModules(IEnumerable<ProcessModuleInfo> modules)
    {
        ArgumentNullException.ThrowIfNull(modules);

        ModuleBounds? gamePlugin = TryFind(modules, GamePluginModuleName, out var plugin)
            ? plugin
            : null;
        ModuleBounds? gameAssembly = TryFind(modules, GameAssemblyModuleName, out var assembly)
            ? assembly
            : null;
        return new ModulePresenceBounds(gamePlugin, gameAssembly);
    }
}
