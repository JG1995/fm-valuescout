using System.Diagnostics;
using System.Reflection;
using BepInEx.Logging;
using FmDataBridge.StaffLimits;
using HarmonyLib;

namespace FmDataBridge.GameActions;

/// <summary>
/// The channel read, and the staff-limit read built on it. A read opens FM's channels on references
/// as its screens do, copies the values FM sends through a hook on the UI's channel-data handler,
/// and closes the channels once FM stops sending. FM drops data for channels its UI did not open,
/// so only the hook sees these values. The staff-limit read opens the staff groups of the club's
/// teams, as the Staff overview does.
/// </summary>
internal sealed partial class GameActionDispatcher : IStaffLimitsReader
{
    private const int TeamReferenceType = 28;
    private const uint GuidKey = 1198877028; // "Guid"

    // FM sends a value several times as it fills it in; the spike saw 13 updates of a staff group
    // within a second. A read takes the last value once none has come for ChannelReadQuiet, stops
    // waiting for a channel FM never answers after ChannelReadPatience, and ends after ChannelReadLongest.
    private static readonly TimeSpan ChannelReadQuiet = TimeSpan.FromMilliseconds(600);
    private static readonly TimeSpan ChannelReadPatience = TimeSpan.FromSeconds(2);
    private static readonly TimeSpan ChannelReadLongest = TimeSpan.FromSeconds(6);

    private static ChannelRead? s_channelRead;
    private LimitTypes? _limitTypes;
    private int _sessionGuid;

    public bool StaffLimitsAvailable => StaffActionsAvailable && _limitTypes is not null;

    private void InstallStaffLimits(Harmony harmony, ManualLogSource log)
    {
        try
        {
            var limits = LimitTypes.Resolve(_types!);
            _limitTypes = limits;
            harmony.Patch(limits.ChannelDataChanged, prefix: Prefix(nameof(CaptureChannelData)));
            harmony.Patch(_types!.OpenChannel, prefix: Prefix(nameof(CaptureSessionGuid)));
            log.LogInfo("FM staff limits are available");
        }
        catch (Exception ex)
        {
            _limitTypes = null;
            log.LogWarning($"FM staff limits are unavailable: {ex.Message}");
        }
    }

    private static HarmonyMethod Prefix(string name) =>
        new(typeof(GameActionDispatcher).GetMethod(name, BindingFlags.Static | BindingFlags.NonPublic));

    // Harmony prefix on FM.UI.InteropDataHandler.OnChannelDataChanged, which receives every update
    // FM sends to its screens. It returns at once unless a channel read is open.
    private static void CaptureChannelData(object keys, object values)
    {
        if (Volatile.Read(ref s_channelRead) is not { } read
            || s_active?._limitTypes is not { } types)
        {
            return;
        }

        try
        {
            types.Copy(keys, values, read);
        }
        catch (Exception)
        {
            // A value the read cannot copy is missed; FM's own handling goes on unchanged.
        }
    }

    // Harmony prefix on the subsystem's OpenChannel: FM's screens name their references with the
    // session Guid, which changes with every load. The staff-limit read names its teams the same way.
    private static void CaptureSessionGuid(object reference)
    {
        if (reference is null || s_active is not { _limitTypes: { } types } dispatcher)
        {
            return;
        }

        try
        {
            if (types.TryReadGuid(reference, out var guid) && guid != 0)
            {
                Volatile.Write(ref dispatcher._sessionGuid, guid);
            }
        }
        catch (Exception)
        {
            // The read then names its teams without a Guid, as FM's actions accept.
        }
    }

    public bool TryReadStaffGroups(
        IReadOnlyList<int> teamIndexes,
        IReadOnlyList<uint> properties,
        out IReadOnlyDictionary<(int TeamIndex, uint Property), FmValue> values,
        out string? error)
    {
        values = new Dictionary<(int TeamIndex, uint Property), FmValue>();
        if (!StaffLimitsAvailable)
        {
            error = "FM's staff limits are unavailable in this game build";
            return false;
        }

        var types = _types!;
        var limits = _limitTypes!;
        var work = new Work();
        var read = AddChannelRead(
            work,
            teamIndexes.SelectMany(team => properties.Select(property => (team, property))).ToArray(),
            team =>
            {
                var pairs = new List<(uint Id, int Value)> { (ReferenceTypeKey, TeamReferenceType) };
                var guid = Volatile.Read(ref _sessionGuid);
                if (guid != 0)
                {
                    pairs.Add((GuidKey, guid));
                }

                pairs.Add((DrefKey, (int)((TeamTable << 24) | (uint)team)));
                return limits.NewTeamReference(types, pairs);
            },
            TimeSpan.Zero);

        if (!Run(work, "staff limit read", out error, ChannelReadLongest))
        {
            return false;
        }

        values = read.Values();
        return true;
    }

    /// <summary>
    /// Adds a read to <paramref name="work"/>: after <paramref name="gap"/>, it opens each property on
    /// the reference <paramref name="reference"/> builds for each index, with the channel steps of
    /// FM's dialogs, waits until FM has settled, and closes every channel. The returned read holds
    /// the last value FM sent on each channel once the work is done.
    /// </summary>
    private ChannelRead AddChannelRead(
        Work work,
        IReadOnlyList<(int Index, uint Property)> targets,
        Func<int, object> reference,
        TimeSpan gap)
    {
        var types = _types!;
        var staff = _staffTypes!;
        var channels = new List<(ulong RawKey, object Key, object Slot, int Index, uint Property)>();
        foreach (var (index, property) in targets)
        {
            var counter = Interlocked.Increment(ref _channelCounter);
            var rawKey = ChannelKeyPrefix | (ulong)counter;
            channels.Add((
                rawKey,
                staff.Key(rawKey),
                staff.DataKey((1UL << 32) | (ulong)(DataSlotBaseIndex + (counter % 10_000))),
                index,
                property));
        }

        var read = new ChannelRead(channels.ToDictionary(
            channel => channel.RawKey,
            channel => (channel.Index, channel.Property)));
        var opened = false;

        void Release(object subsystem)
        {
            Interlocked.CompareExchange(ref s_channelRead, null, read);
            if (!opened)
            {
                return;
            }

            opened = false;
            foreach (var channel in channels)
            {
                // Each call on its own, so one failure leaves no other channel open.
                TryInvoke(staff.CloseChannel, subsystem, channel.Key);
                TryInvoke(staff.ReleaseData, subsystem, channel.Slot);
                TryInvoke(staff.RemoveNode, subsystem, channel.Key);
            }
        }

        work.Then(gap, subsystem =>
        {
            opened = true;
            work.OnFailure(Release);
            read.Start();
            Volatile.Write(ref s_channelRead, read);
            foreach (var channel in channels)
            {
                var property = staff.Property(channel.Property);
                staff.AddNode.Invoke(subsystem, new[] { channel.Key, staff.RootKey(), property });
                staff.AddData.Invoke(subsystem, new[] { channel.Slot });
                staff.SetTarget.Invoke(subsystem, new[] { channel.Key, channel.Slot });
                types.OpenChannel.Invoke(subsystem, new[] { reference(channel.Index), property, channel.Key });
            }
        });
        work.Until(read.IsSettled);
        work.Then(TimeSpan.Zero, Release);
        return read;
    }

    private static void TryInvoke(MethodInfo method, object subsystem, object argument)
    {
        try
        {
            method.Invoke(subsystem, new[] { argument });
        }
        catch (Exception)
        {
            // Best effort: the channel or slot may already be gone.
        }
    }

    /// <summary>The channels one read opened and the last value FM sent on each.</summary>
    private sealed class ChannelRead
    {
        private readonly object _gate = new();
        private readonly IReadOnlyDictionary<ulong, (int Index, uint Property)> _channels;
        private readonly HashSet<ulong> _answered = new();
        private readonly Dictionary<ulong, FmValue> _values = new();
        private readonly Stopwatch _sinceOpen = new();
        private readonly Stopwatch _sinceUpdate = new();

        public ChannelRead(IReadOnlyDictionary<ulong, (int Index, uint Property)> channels) =>
            _channels = channels;

        public bool Expects(ulong key) => _channels.ContainsKey(key);

        public void Start()
        {
            lock (_gate)
            {
                _sinceOpen.Restart();
                _sinceUpdate.Restart();
            }
        }

        public void Receive(ulong key, FmValue? value)
        {
            lock (_gate)
            {
                _answered.Add(key);
                if (value is not null)
                {
                    _values[key] = value;
                }

                _sinceUpdate.Restart();
            }
        }

        public bool IsSettled()
        {
            lock (_gate)
            {
                var quiet = _sinceUpdate.Elapsed >= ChannelReadQuiet;
                return _sinceOpen.Elapsed >= ChannelReadLongest
                    || (quiet && (_answered.Count == _channels.Count || _sinceOpen.Elapsed >= ChannelReadPatience));
            }
        }

        public IReadOnlyDictionary<(int Index, uint Property), FmValue> Values()
        {
            lock (_gate)
            {
                return _values.ToDictionary(entry => _channels[entry.Key], entry => entry.Value);
            }
        }
    }

    /// <summary>
    /// The members the read uses: the team reference, the channel-data handler it hooks, and FM's
    /// typed values, records, lists, and numbers, which it copies into <see cref="FmValue"/>s.
    /// </summary>
    private sealed class LimitTypes
    {
        // Group record, job list, job record: deeper values, such as the holders' references, are skipped.
        private const int MaxDepth = 3;
        private const int MaxItems = 256;
        private const string StringType = "System.String";
        private const string NumberType = "SI.Core.DynamicNumber";

        private ConstructorInfo _teamReference = null!;
        private MethodInfo _referenceTryGetValue = null!;
        private PropertyInfo _pointer = null!;
        private MethodInfo _spanLength = null!;
        private MethodInfo _listCount = null!;
        private MethodInfo _listItem = null!;
        private MethodInfo _isNull = null!;
        private MethodInfo _dataType = null!;
        private MethodInfo _typeFullName = null!;
        private MethodInfo _get = null!;
        private MethodInfo _asString = null!;
        private MethodInfo _castRecord = null!;
        private MethodInfo _castList = null!;
        private MethodInfo _unboxNumber = null!;
        private MethodInfo _recordEnumerator = null!;
        private MethodInfo _moveNext = null!;
        private MethodInfo _current = null!;
        private MethodInfo _entryKey = null!;
        private MethodInfo _entryValue = null!;
        private FieldInfo _numberKind = null!;
        private FieldInfo _numberValue = null!;
        private FieldInfo _signed = null!;
        private FieldInfo _unsigned = null!;
        private FieldInfo _floating = null!;

        public MethodInfo ChannelDataChanged { get; private set; } = null!;

        public static LimitTypes Resolve(FmTypes types)
        {
            const BindingFlags Instance = BindingFlags.Public | BindingFlags.Instance;
            var handler = FmTypes.Find("FM.UI", "FM.UI.InteropDataHandler");
            var teamReference = FmTypes.Find("FM.UI", "FM.UI.TeamReference");
            var reference = FmTypes.Find("SI.Interop", "SI.Interop.InteropReference");
            var typedValue = FmTypes.Find("SI.Core", "SI.Core.TypedValue");
            var record = FmTypes.Find("SI.Core", "SI.Core.Record");
            var number = FmTypes.Find("SI.Core", "SI.Core.DynamicNumber");
            var union = FmTypes.Find("SI.Core", "SI.Core.DynamicNumber+Union");
            var il2CppType = FmTypes.Find("Il2Cppmscorlib", "Il2CppSystem.Type");
            var span = FmTypes.Find("Il2Cppmscorlib", "Il2CppSystem.ReadOnlySpan`1").MakeGenericType(typeof(ulong));
            var list = FmTypes.Find("Il2Cppmscorlib", "Il2CppSystem.Collections.Generic.List`1").MakeGenericType(typedValue);
            var objectBase = FmTypes.Find("Il2Cppmscorlib", "Il2CppSystem.Object").BaseType;
            while (objectBase is not null && objectBase.Name != "Il2CppObjectBase")
            {
                objectBase = objectBase.BaseType;
            }

            if (objectBase is null)
            {
                throw new InvalidOperationException("Il2CppObjectBase not found");
            }

            MethodInfo Generic(string name) => FmTypes.Member(
                objectBase.GetMethods(Instance).SingleOrDefault(method => method.Name == name
                    && method.IsGenericMethodDefinition
                    && method.GetParameters().Length == 0),
                $"Il2CppObjectBase.{name}");
            var tryCast = Generic("TryCast");
            var unbox = Generic("Unbox");
            var enumerator = FmTypes.Member(
                record.GetMethod("GetEnumerator", Instance, null, Type.EmptyTypes, null),
                "Record.GetEnumerator");
            var current = FmTypes.Member(enumerator.ReturnType.GetMethod("get_Current", Instance), "Record enumerator Current");

            return new LimitTypes
            {
                ChannelDataChanged = FmTypes.Member(handler.GetMethod("OnChannelDataChanged", Instance), "InteropDataHandler.OnChannelDataChanged"),
                _teamReference = FmTypes.Member(
                    teamReference.GetConstructor(new[] { types.PairListType }),
                    "TeamReference(List<Pair>)"),
                _referenceTryGetValue = FmTypes.Member(
                    reference.GetMethod("TryGetValue", new[] { typeof(uint), typeof(int).MakeByRefType() }),
                    "InteropReference.TryGetValue"),
                _pointer = FmTypes.Member(objectBase.GetProperty("Pointer", Instance), "Il2CppObjectBase.Pointer"),
                _spanLength = FmTypes.Member(span.GetMethod("get_Length", Instance), "ReadOnlySpan.Length"),
                _listCount = FmTypes.Member(list.GetMethod("get_Count", Instance), "List<TypedValue>.Count"),
                _listItem = FmTypes.Member(list.GetMethod("get_Item", Instance), "List<TypedValue>.Item"),
                _isNull = FmTypes.Member(typedValue.GetMethod("get_IsNull", Instance), "TypedValue.IsNull"),
                _dataType = FmTypes.Member(typedValue.GetMethod("get_DataType", Instance), "TypedValue.DataType"),
                _typeFullName = FmTypes.Member(il2CppType.GetMethod("get_FullName", Instance), "Type.FullName"),
                _get = FmTypes.Member(
                    typedValue.GetMethods(Instance).SingleOrDefault(method => method.Name == "Get"
                        && !method.IsGenericMethodDefinition
                        && method.GetParameters().Length == 0),
                    "TypedValue.Get()"),
                _asString = FmTypes.Member(typedValue.GetMethod("AsString", Instance, null, Type.EmptyTypes, null), "TypedValue.AsString"),
                _castRecord = tryCast.MakeGenericMethod(record),
                _castList = tryCast.MakeGenericMethod(list),
                _unboxNumber = unbox.MakeGenericMethod(number),
                _recordEnumerator = enumerator,
                _moveNext = FmTypes.Member(enumerator.ReturnType.GetMethod("MoveNext", Instance), "Record enumerator MoveNext"),
                _current = current,
                _entryKey = FmTypes.Member(current.ReturnType.GetMethod("get_Key", Instance), "Record entry Key"),
                _entryValue = FmTypes.Member(current.ReturnType.GetMethod("get_Value", Instance), "Record entry Value"),
                _numberKind = FmTypes.Member(number.GetField("_Kind_k__BackingField"), "DynamicNumber kind"),
                _numberValue = FmTypes.Member(number.GetField("m_value"), "DynamicNumber.m_value"),
                _signed = FmTypes.Member(union.GetField("SignedWhole"), "Union.SignedWhole"),
                _unsigned = FmTypes.Member(union.GetField("UnsignedWhole"), "Union.UnsignedWhole"),
                _floating = FmTypes.Member(union.GetField("FloatingPoint"), "Union.FloatingPoint"),
            };
        }

        public object NewTeamReference(FmTypes types, IEnumerable<(uint Id, int Value)> pairs) =>
            _teamReference.Invoke(new[] { types.NewPairList(pairs) });

        public bool TryReadGuid(object reference, out int guid)
        {
            var arguments = new object[] { GuidKey, 0 };
            var found = (bool)_referenceTryGetValue.Invoke(reference, arguments)!;
            guid = (int)arguments[1];
            return found;
        }

        /// <summary>Copies each value whose key the read opened, in the order FM sent them.</summary>
        public void Copy(object keys, object values, ChannelRead read)
        {
            var keyCount = (int)_spanLength.Invoke(keys, null)!;
            var keyData = SpanData(keys, keyCount);
            if (keyData == IntPtr.Zero)
            {
                return;
            }

            var count = Math.Min((int)_listCount.Invoke(values, null)!, keyCount);
            for (var i = 0; i < count; i++)
            {
                var key = ReadKey(keyData, i);
                if (read.Expects(key))
                {
                    read.Receive(key, CopyValue(_listItem.Invoke(values, new object[] { i }), 0));
                }
            }
        }

        // Il2CppInterop's span indexer returns one constant address, so read the boxed span's own
        // fields: after the 16-byte object header, the element pointer and the int length. The
        // pointer is trusted only when that length is the one the span reports.
        private unsafe IntPtr SpanData(object span, int expectedLength)
        {
            if (expectedLength <= 0 || _pointer.GetValue(span) is not IntPtr boxed || boxed == IntPtr.Zero)
            {
                return IntPtr.Zero;
            }

            var body = (byte*)boxed + 16;
            var pointer = *(IntPtr*)body;
            var length = *(int*)(body + 8);
            var address = (ulong)pointer;
            return length == expectedLength && address > 0x10000 && address < 0x7FFF_FFFF_FFFFUL
                ? pointer
                : IntPtr.Zero;
        }

        private static unsafe ulong ReadKey(IntPtr data, int index) => *((ulong*)data + index);

        private FmValue? CopyValue(object? typedValue, int depth)
        {
            if (typedValue is null || depth > MaxDepth || (bool)_isNull.Invoke(typedValue, null)!)
            {
                return null;
            }

            var typeName = _dataType.Invoke(typedValue, null) is { } type
                ? _typeFullName.Invoke(type, null) as string
                : null;
            if (typeName == StringType)
            {
                return _asString.Invoke(typedValue, null) is string text ? new FmText(text) : null;
            }

            if (_get.Invoke(typedValue, null) is not { } boxed)
            {
                return null;
            }

            if (typeName == NumberType)
            {
                return ReadNumber(boxed);
            }

            if (depth == MaxDepth)
            {
                return null;
            }

            if (_castRecord.Invoke(boxed, null) is { } record)
            {
                return CopyRecord(record, depth);
            }

            return _castList.Invoke(boxed, null) is { } list ? CopyList(list, depth) : null;
        }

        private FmNumber? ReadNumber(object boxed)
        {
            var number = _unboxNumber.Invoke(boxed, null)!;
            var union = _numberValue.GetValue(number)!;
            switch (Convert.ToInt32(_numberKind.GetValue(number)))
            {
                case 0:
                    return new FmNumber((long)_signed.GetValue(union)!);
                case 1:
                    var whole = (ulong)_unsigned.GetValue(union)!;
                    return whole <= long.MaxValue ? new FmNumber((long)whole) : null;
                case 2:
                    var floating = (double)_floating.GetValue(union)!;
                    return double.IsFinite(floating) && Math.Abs(floating) < 1e15
                        ? new FmNumber((long)Math.Round(floating))
                        : null;
                default:
                    return null;
            }
        }

        private FmRecord CopyRecord(object record, int depth)
        {
            var items = new Dictionary<uint, FmValue>();
            var enumerator = _recordEnumerator.Invoke(record, null)!;
            for (var seen = 0; seen < MaxItems && (bool)_moveNext.Invoke(enumerator, null)!; seen++)
            {
                var entry = _current.Invoke(enumerator, null)!;
                if (CopyValue(_entryValue.Invoke(entry, null), depth + 1) is { } value)
                {
                    items[(uint)_entryKey.Invoke(entry, null)!] = value;
                }
            }

            return new FmRecord(items);
        }

        private FmList CopyList(object list, int depth)
        {
            var items = new List<FmValue>();
            var count = Math.Min((int)_listCount.Invoke(list, null)!, MaxItems);
            for (var i = 0; i < count; i++)
            {
                if (CopyValue(_listItem.Invoke(list, new object[] { i }), depth + 1) is { } value)
                {
                    items.Add(value);
                }
            }

            return new FmList(items);
        }
    }
}
