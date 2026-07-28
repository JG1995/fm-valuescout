namespace FmDataBridge.Memory;

public readonly record struct MemoryRegion(
    ulong BaseAddress,
    ulong Size,
    uint Protect,
    uint Type,
    uint State);
