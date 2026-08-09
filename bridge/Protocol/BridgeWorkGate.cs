namespace FmDataBridge.Protocol;

/// <summary>
/// Serializes full dumps and the two bounded player-boost actions inside the bridge process.
/// </summary>
internal sealed class BridgeWorkGate
{
    private int _busy;

    public bool IsBusy => Volatile.Read(ref _busy) != 0;

    public bool TryEnter() => Interlocked.CompareExchange(ref _busy, 1, 0) == 0;

    public void Exit() => Volatile.Write(ref _busy, 0);
}
