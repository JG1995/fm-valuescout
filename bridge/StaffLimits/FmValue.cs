namespace FmDataBridge.StaffLimits;

/// <summary>
/// A managed copy of one of FM's typed values. FM recycles its values once an update has been
/// handled, so the channel hook copies what it needs while FM still owns the value. Values the
/// staff-limit read does not use, such as references, are left out.
/// </summary>
internal abstract record FmValue;

internal sealed record FmNumber(long Value) : FmValue;

internal sealed record FmText(string Value) : FmValue;

/// <summary>FM's record: values under four-character keys read as big-endian numbers.</summary>
internal sealed record FmRecord(IReadOnlyDictionary<uint, FmValue> Items) : FmValue
{
    public T? Get<T>(uint key)
        where T : FmValue =>
        Items.TryGetValue(key, out var value) ? value as T : null;
}

internal sealed record FmList(IReadOnlyList<FmValue> Items) : FmValue;
