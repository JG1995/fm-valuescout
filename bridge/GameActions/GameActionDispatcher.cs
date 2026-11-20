using System.Collections.Concurrent;
using System.Diagnostics;
using System.Reflection;
using BepInEx;
using BepInEx.Logging;
using FmDataBridge.Mutations;
using HarmonyLib;

namespace FmDataBridge.GameActions;

/// <summary>
/// Sends FM27's own actions on the game's main thread: the squad change its "Move to" menu sends,
/// the contract termination its staff menu sends, and the in-game editor's "Move to my club" with a
/// chosen job. It also reads the board's staff limits from the channels FM's Staff overview opens
/// (in the StaffLimits part). FM's types are found at run time so the bridge builds without the
/// game's assemblies; see the FM27 squad move action, FM27 staff actions, and FM27 board staff
/// limits notes in the wiki.
/// </summary>
internal sealed partial class GameActionDispatcher : ISquadMoveDispatcher, IStaffChangeDispatcher
{
    // Four-character keys FM uses in references and action records.
    private const uint ReferenceTypeKey = 825307168; // "110 "
    private const uint ActionKey = 1633904996; // "acid"
    private const uint DrefKey = 1685218662; // "dref"
    private const uint DrefListKey = 1685218636; // "dreL"
    private const uint FromTeamTypeKey = 1668510579; // "csss"
    private const uint ClubKey = 1130589301; // "Cclu"
    private const uint HumanManagerKey = 1214805357; // "Hhum"
    private const uint ToTeamTypeKey = 1416919408; // "Ttyp"
    private const int ActionReferenceType = 1;
    private const int ChangeSquadAction = 1516726887; // ActionID.PersonChangePlayersClubSquad
    private const uint DoActionEvent = 44; // InteropEventIDs.EventFmxDoAction
    private const long PersonTable = 1;
    private const long ClubTable = 3;
    private const long TeamTable = 24;

    // Contract termination: ActionID.PersonTerminateContract and its record keys.
    private const int TerminateContractAction = 1701671793;
    private const uint OnFreeKey = 1349087086; // "Pion"
    private const uint CancelKey = 1348693603; // "Pcnc"
    private const uint ContractTeamKey = 1349804916; // "Ptct"
    private const uint ConfirmedKey = 2003977071; // "wrCo"

    // The in-game editor's move: an IGEMovePersonReference, its SelectedJob property, and the event
    // its dialog sends. The job travels through a data slot bound to the SelectedJob channel.
    private const int MovePersonReferenceType = 57;
    private const uint MoveHumanKey = 1348620377; // "PbPY"
    private const uint MovePersonKey = 1348825715; // "Pers"
    private const uint MoveTeamKey = 1769566810; // "iyrZ"
    private const uint SelectedJobProperty = 1500739907; // "YsyC"
    private const uint OptionValueKey = 1886680684; // "ptvl"
    private const uint JobTeamKey = 1416914273; // "Ttea"
    private const uint JobTypeKey = 1348692591; // "Pcjo"
    private const uint MoveToTeamEvent = 2004440433; // InteropEventIDs.EventFmxMoveToTeam

    // The editor's dialog waits for the game to answer each channel step; these gaps are what the
    // spike verified, with room to spare.
    private static readonly TimeSpan ChannelStepGap = TimeSpan.FromMilliseconds(500);
    private static readonly TimeSpan HireTime = TimeSpan.FromSeconds(2);

    // Channel keys only need to differ from the UI's hashed keys.
    private const ulong ChannelKeyPrefix = 0x5F56_5300_0000_0000UL;

    // ponytail: data slots use one fixed index range far above the UI's own (about 9,600 in a
    // long session, reused rather than grown), with version 1, and are released after each hire.
    // Upgrade to allocating through FM's binding system if FM's UI data indexes approach 50,000.
    private const int DataSlotBaseIndex = 50_000;

    private static readonly TimeSpan DispatchTimeout = TimeSpan.FromSeconds(10);
    private static GameActionDispatcher? s_active;

    private readonly ConcurrentQueue<Work> _queue = new();
    private FmTypes? _types;
    private StaffTypes? _staffTypes;
    private object? _subsystem;
    private Work? _running;
    private long _channelCounter;

    public bool IsAvailable => _types is not null;

    public bool StaffActionsAvailable => _types is not null && _staffTypes is not null;

    /// <summary>Finds FM's interop types and hooks its interop subsystem. Leaves the dispatcher unavailable on failure.</summary>
    public void Install(ManualLogSource log)
    {
        var harmony = new Harmony("com.fmvaluescout.databridge.gameactions");
        try
        {
            var types = FmTypes.Resolve();
            var capture = new HarmonyMethod(typeof(GameActionDispatcher).GetMethod(
                nameof(CaptureSubsystem),
                BindingFlags.Static | BindingFlags.NonPublic));
            harmony.Patch(types.OpenChannel, prefix: capture);
            harmony.Patch(types.SendEvent, prefix: capture);
            harmony.Patch(
                types.Tick,
                postfix: new HarmonyMethod(typeof(GameActionDispatcher).GetMethod(
                    nameof(RunQueuedWork),
                    BindingFlags.Static | BindingFlags.NonPublic)));
            s_active = this;
            _types = types;
            log.LogInfo("FM squad actions are available");
        }
        catch (Exception ex)
        {
            log.LogWarning($"FM squad actions are unavailable: {ex.Message}");
            return;
        }

        try
        {
            _staffTypes = StaffTypes.Resolve(_types);
            log.LogInfo("FM staff actions are available");
        }
        catch (Exception ex)
        {
            log.LogWarning($"FM staff actions are unavailable: {ex.Message}");
            return;
        }

        InstallStaffLimits(harmony, log);
    }

    public bool TryDispatch(IReadOnlyList<SquadMoveAction> actions, out string? error)
    {
        if (_types is null)
        {
            error = "FM's squad action is unavailable in this game build";
            return false;
        }

        var work = new Work();
        work.Then(TimeSpan.Zero, subsystem =>
        {
            foreach (var action in actions)
            {
                SendSquadMove(subsystem, action);
            }
        });
        return Run(work, "squad moves", out error);
    }

    public bool TryDispatch(IReadOnlyList<StaffAction> actions, out string? error)
    {
        if (_types is null || _staffTypes is null)
        {
            error = "FM's staff actions are unavailable in this game build";
            return false;
        }

        var work = new Work();
        var hireStarted = false;
        foreach (var action in actions)
        {
            if (action.Kind == StaffActionKind.Sack)
            {
                work.Then(TimeSpan.Zero, subsystem => SendTermination(subsystem, action));
                continue;
            }

            // A gap after the last sack lets FM end those contracts before the first hire.
            var gap = hireStarted ? TimeSpan.Zero : ChannelStepGap;
            hireStarted = true;
            AddHire(work, action, gap);
        }

        return Run(work, "staff changes", out error, HireTime * actions.Count);
    }

    private bool Run(Work work, string what, out string? error, TimeSpan? expected = null)
    {
        if (Volatile.Read(ref _subsystem) is null)
        {
            error = "FM has not opened a game screen yet";
            return false;
        }

        _queue.Enqueue(work);
        if (!work.Done.Wait(DispatchTimeout + (expected ?? TimeSpan.Zero)) && work.TryCancel())
        {
            error = $"FM did not run the {what}; open a game screen and retry";
            return false;
        }

        // Started on the main thread before the wait ended: it finishes there.
        work.Done.Wait();
        error = work.Error;
        return error is null;
    }

    // Harmony prefix on the subsystem's OpenChannel and SendEvent: both run as FM's screens show data.
    private static void CaptureSubsystem(object __instance)
    {
        if (s_active is { } dispatcher)
        {
            Volatile.Write(ref dispatcher._subsystem, __instance);
        }
    }

    // Harmony postfix on the subsystem's Tick, which runs on the game's main thread. Work runs one
    // item at a time; an item's steps may wait across ticks.
    private static void RunQueuedWork(object __instance)
    {
        if (s_active is not { } dispatcher)
        {
            return;
        }

        Volatile.Write(ref dispatcher._subsystem, __instance);
        while (true)
        {
            if (dispatcher._running is null)
            {
                if (!dispatcher._queue.TryDequeue(out var next))
                {
                    return;
                }

                if (!next.TryStart())
                {
                    continue;
                }

                dispatcher._running = next;
            }

            if (!dispatcher._running.Advance(__instance))
            {
                return;
            }

            dispatcher._running = null;
        }
    }

    private void SendSquadMove(object subsystem, SquadMoveAction action)
    {
        var types = _types!;
        var reference = types.NewActionReference(new[]
        {
            (ReferenceTypeKey, ActionReferenceType),
            (ActionKey, ChangeSquadAction),
        });
        var data = types.NewRecord(new (uint, object)[]
        {
            (FromTeamTypeKey, types.Number(action.FromTeamType)),
            (ClubKey, Dref(types, ClubTable, action.ClubIndex)),
            (HumanManagerKey, Dref(types, PersonTable, action.HumanManagerIndex)),
            (DrefKey, Dref(types, PersonTable, action.PersonIndex)),
            (DrefListKey, types.NewList(new[] { Dref(types, PersonTable, action.PersonIndex) })),
            (ToTeamTypeKey, types.Number(action.ToTeamType)),
        });
        types.SendEvent.Invoke(subsystem, new object[] { DoActionEvent, reference, data });
    }

    /// <summary>The action FM's staff menu sends once its "end contract" dialog is confirmed.</summary>
    private void SendTermination(object subsystem, StaffAction action)
    {
        var types = _types!;
        var reference = types.NewActionReference(new[]
        {
            (ReferenceTypeKey, ActionReferenceType),
            (ActionKey, TerminateContractAction),
        });
        var data = types.NewRecord(new (uint, object)[]
        {
            (OnFreeKey, types.Boolean(false)),
            (CancelKey, types.Boolean(false)),
            (ContractTeamKey, Dref(types, TeamTable, action.TeamIndex)),
            (HumanManagerKey, Dref(types, PersonTable, action.HumanManagerIndex)),
            (DrefKey, Dref(types, PersonTable, action.PersonIndex)),
            (DrefListKey, types.NewList(new[] { Dref(types, PersonTable, action.PersonIndex) })),
            (ConfirmedKey, types.Boolean(true)),
        });
        types.SendEvent.Invoke(subsystem, new object[] { DoActionEvent, reference, data });
    }

    /// <summary>
    /// The in-game editor's "Move to my club", as its dialog runs it: register a channel for the
    /// move's SelectedJob, bind it to a data slot, open it, write the job and team, send the move,
    /// then release the channel and slot.
    /// </summary>
    private void AddHire(Work work, StaffAction action, TimeSpan gap)
    {
        var types = _types!;
        var staff = _staffTypes!;
        var counter = Interlocked.Increment(ref _channelCounter);
        var key = staff.Key(ChannelKeyPrefix | (ulong)counter);
        var slot = staff.DataKey((1UL << 32) | (ulong)(DataSlotBaseIndex + (counter % 10_000)));
        object? reference = null;
        var opened = false;

        void Release(object subsystem)
        {
            if (!opened)
            {
                return;
            }

            opened = false;
            staff.CloseChannel.Invoke(subsystem, new[] { key });
            staff.ReleaseData.Invoke(subsystem, new[] { slot });
            staff.RemoveNode.Invoke(subsystem, new[] { key });
        }

        work.Then(gap, subsystem =>
        {
            reference = staff.NewMoveReference(types, new[]
            {
                (ReferenceTypeKey, MovePersonReferenceType),
                (MoveHumanKey, (int)((PersonTable << 24) | (uint)action.HumanManagerIndex)),
                (MovePersonKey, (int)((PersonTable << 24) | (uint)action.PersonIndex)),
                (MoveTeamKey, (int)((TeamTable << 24) | (uint)action.FirstTeamIndex)),
            });
            var property = staff.Property(SelectedJobProperty);
            opened = true;
            work.OnFailure(Release);
            staff.AddNode.Invoke(subsystem, new[] { key, staff.RootKey(), property });
            staff.AddData.Invoke(subsystem, new[] { slot });
            staff.SetTarget.Invoke(subsystem, new[] { key, slot });
            types.OpenChannel.Invoke(subsystem, new[] { reference, property, key });
        });
        work.Then(ChannelStepGap, subsystem =>
        {
            var job = types.NewRecord(new (uint, object)[]
            {
                (OptionValueKey, types.NewRecord(new (uint, object)[]
                {
                    (JobTeamKey, Dref(types, TeamTable, action.TeamIndex)),
                    (JobTypeKey, staff.Unsigned((ulong)action.JobType)),
                })),
            });
            staff.SetData.Invoke(subsystem, new[] { slot, job });
            staff.UpdateChannelData.Invoke(subsystem, new[] { key, job });
        });
        work.Then(ChannelStepGap, subsystem =>
            types.SendEvent.Invoke(subsystem, new[] { (object)MoveToTeamEvent, reference!, staff.Null() }));
        work.Then(TimeSpan.Zero, Release);
    }

    /// <summary>FM's database reference: a record holding the table in the top byte and the index below.</summary>
    private static object Dref(FmTypes types, long table, int index) =>
        types.NewRecord(new (uint, object)[] { (DrefKey, types.Number((table << 24) | (uint)index)) });

    /// <summary>Steps that run in order on the main thread, each once its gap since the last has passed.</summary>
    private sealed class Work
    {
        private readonly Queue<(TimeSpan Gap, Action<object> Run, Func<bool>? Ready)> _steps = new();
        private readonly List<Action<object>> _onFailure = new();
        private readonly Stopwatch _sinceLastStep = new();
        private int _state; // 0 queued, 1 started, 2 cancelled

        public ManualResetEventSlim Done { get; } = new();

        public string? Error { get; private set; }

        public void Then(TimeSpan gap, Action<object> run) => _steps.Enqueue((gap, run, null));

        /// <summary>Holds the later steps until <paramref name="ready"/> returns true, checked once a tick.</summary>
        public void Until(Func<bool> ready) => _steps.Enqueue((TimeSpan.Zero, _ => { }, ready));

        /// <summary>Runs if a later step fails, so channels and slots are not left behind.</summary>
        public void OnFailure(Action<object> cleanup) => _onFailure.Add(cleanup);

        public bool TryStart()
        {
            var started = Interlocked.CompareExchange(ref _state, 1, 0) == 0;
            if (started)
            {
                _sinceLastStep.Restart();
            }

            return started;
        }

        public bool TryCancel() => Interlocked.CompareExchange(ref _state, 2, 0) == 0;

        /// <summary>Runs every due step; returns true once the work has finished.</summary>
        public bool Advance(object subsystem)
        {
            try
            {
                while (_steps.TryPeek(out var step)
                       && _sinceLastStep.Elapsed >= step.Gap
                       && (step.Ready?.Invoke() ?? true))
                {
                    _steps.Dequeue();
                    step.Run(subsystem);
                    _sinceLastStep.Restart();
                }
            }
            catch (Exception ex)
            {
                Error = $"FM rejected an action: {(ex.InnerException ?? ex).Message}";
                _steps.Clear();
                foreach (var cleanup in _onFailure)
                {
                    try
                    {
                        cleanup(subsystem);
                    }
                    catch (Exception)
                    {
                        // Best effort: the channel or slot may already be gone.
                    }
                }
            }

            if (_steps.Count > 0)
            {
                return false;
            }

            Done.Set();
            return true;
        }
    }

    /// <summary>FM's interop types and members, found by name in the game's interop assemblies.</summary>
    private sealed class FmTypes
    {
        private Type _pair = null!;
        private FieldInfo _pairId = null!;
        private FieldInfo _pairValue = null!;
        private Type _pairList = null!;
        private MethodInfo _pairListAdd = null!;
        private ConstructorInfo _actionReference = null!;
        private Type _record = null!;
        private MethodInfo _recordAdd = null!;
        private Type _typedValueList = null!;
        private MethodInfo _typedValueListAdd = null!;
        private MethodInfo _createNumber = null!;
        private MethodInfo _createBoolean = null!;
        private MethodInfo _createRecord = null!;
        private MethodInfo _createList = null!;

        public MethodInfo OpenChannel { get; private set; } = null!;

        public MethodInfo SendEvent { get; private set; } = null!;

        public MethodInfo Tick { get; private set; } = null!;

        public static FmTypes Resolve()
        {
            var subsystem = Find("FM.GamePlugin", "FM.GamePlugin.GameInteropSubsystem");
            var reference = Find("SI.Interop", "SI.Interop.InteropReference");
            var pair = Find("SI.Interop", "SI.Interop.InteropReference+Pair");
            var actionReference = Find("FM.UI", "FM.UI.ActionReference");
            var record = Find("SI.Core", "SI.Core.Record");
            var typedValue = Find("SI.Core", "SI.Core.TypedValue");
            var list = Find("Il2Cppmscorlib", "Il2CppSystem.Collections.Generic.List`1");
            var pairList = list.MakeGenericType(pair);
            var typedValueList = list.MakeGenericType(typedValue);
            var create = typedValue
                .GetMethods(BindingFlags.Public | BindingFlags.Static)
                .Single(method => method.Name == "Create"
                    && method.IsGenericMethodDefinition
                    && method.GetParameters().Length == 1);

            return new FmTypes
            {
                OpenChannel = Member(subsystem.GetMethod("OpenChannel"), "OpenChannel"),
                SendEvent = Member(
                    subsystem.GetMethod("SendEvent", new[] { typeof(uint), reference, typedValue }),
                    "SendEvent"),
                Tick = Member(
                    subsystem.GetMethod("Tick", BindingFlags.Public | BindingFlags.Instance | BindingFlags.DeclaredOnly, null, Type.EmptyTypes, null),
                    "Tick"),
                _pair = pair,
                _pairId = Member(pair.GetField("m_id"), "Pair.m_id"),
                _pairValue = Member(pair.GetField("m_value"), "Pair.m_value"),
                _pairList = pairList,
                _pairListAdd = Member(pairList.GetMethod("Add"), "List<Pair>.Add"),
                _actionReference = Member(actionReference.GetConstructor(new[] { pairList }), "ActionReference(List<Pair>)"),
                _record = record,
                _recordAdd = Member(record.GetMethod("Add", new[] { typeof(uint), typedValue }), "Record.Add"),
                _typedValueList = typedValueList,
                _typedValueListAdd = Member(typedValueList.GetMethod("Add"), "List<TypedValue>.Add"),
                _createNumber = Member(typedValue.GetMethod("Create", new[] { typeof(long) }), "TypedValue.Create(long)"),
                _createBoolean = Member(typedValue.GetMethod("Create", new[] { typeof(bool) }), "TypedValue.Create(bool)"),
                _createRecord = create.MakeGenericMethod(record),
                _createList = create.MakeGenericMethod(typedValueList),
            };
        }

        public Type PairListType => _pairList;

        public object Number(long value) => _createNumber.Invoke(null, new object[] { value })!;

        public object Boolean(bool value) => _createBoolean.Invoke(null, new object[] { value })!;

        public object NewActionReference(IEnumerable<(uint Id, int Value)> pairs) =>
            _actionReference.Invoke(new[] { NewPairList(pairs) });

        public object NewPairList(IEnumerable<(uint Id, int Value)> pairs)
        {
            var list = Activator.CreateInstance(_pairList)!;
            foreach (var (id, value) in pairs)
            {
                var pair = Activator.CreateInstance(_pair)!;
                _pairId.SetValue(pair, id);
                _pairValue.SetValue(pair, value);
                _pairListAdd.Invoke(list, new[] { pair });
            }

            return list;
        }

        public object NewRecord(IEnumerable<(uint Key, object Value)> items)
        {
            var record = Activator.CreateInstance(_record)!;
            foreach (var (key, value) in items)
            {
                _recordAdd.Invoke(record, new object[] { key, value });
            }

            return _createRecord.Invoke(null, new[] { record })!;
        }

        public object NewList(IEnumerable<object> values)
        {
            var list = Activator.CreateInstance(_typedValueList)!;
            foreach (var value in values)
            {
                _typedValueListAdd.Invoke(list, new[] { value });
            }

            return _createList.Invoke(null, new[] { list })!;
        }

        internal static Type Find(string assemblyName, string typeName)
        {
            var assembly = AppDomain.CurrentDomain.GetAssemblies()
                .FirstOrDefault(candidate => candidate.GetName().Name == assemblyName)
                ?? LoadInterop(assemblyName);
            return assembly.GetType(typeName, throwOnError: false)
                ?? throw new InvalidOperationException($"{typeName} not found in {assemblyName}");
        }

        private static Assembly LoadInterop(string assemblyName)
        {
            try
            {
                return Assembly.Load(assemblyName);
            }
            catch (Exception) when (File.Exists(InteropPath(assemblyName)))
            {
                return Assembly.LoadFrom(InteropPath(assemblyName));
            }
        }

        private static string InteropPath(string assemblyName) =>
            Path.Combine(Paths.BepInExRootPath, "interop", assemblyName + ".dll");

        internal static T Member<T>(T? member, string name)
            where T : class =>
            member ?? throw new InvalidOperationException($"FM member {name} not found");
    }

    /// <summary>
    /// The binding and editor members the staff hire uses: channel keys, data slots, the editor's
    /// move reference, and the subsystem methods that register, write and release a channel.
    /// </summary>
    private sealed class StaffTypes
    {
        private ConstructorInfo _key = null!;
        private Type _dataKey = null!;
        private FieldInfo _dataKeyValue = null!;
        private ConstructorInfo _property = null!;
        private MethodInfo _rootKey = null!;
        private ConstructorInfo _moveReference = null!;
        private MethodInfo _createUnsigned = null!;
        private MethodInfo _null = null!;

        public MethodInfo AddNode { get; private set; } = null!;

        public MethodInfo AddData { get; private set; } = null!;

        public MethodInfo SetTarget { get; private set; } = null!;

        public MethodInfo SetData { get; private set; } = null!;

        public MethodInfo UpdateChannelData { get; private set; } = null!;

        public MethodInfo CloseChannel { get; private set; } = null!;

        public MethodInfo ReleaseData { get; private set; } = null!;

        public MethodInfo RemoveNode { get; private set; } = null!;

        public static StaffTypes Resolve(FmTypes types)
        {
            var subsystem = FmTypes.Find("FM.GamePlugin", "FM.GamePlugin.GameInteropSubsystem");
            var key = FmTypes.Find("SI.Bindable", "SI.Bindable.Bindings+Key");
            var dataKey = FmTypes.Find("SI.Bindable", "SI.Bindable.Bindings+DataKey");
            var bindings = FmTypes.Find("SI.Bindable", "SI.Bindable.Bindings");
            var property = FmTypes.Find("SI.Bindable.Reference.Core", "SI.Bindable.Reference.Core.PropertyID");
            var typedValue = FmTypes.Find("SI.Core", "SI.Core.TypedValue");
            var move = FmTypes.Find("FM.UI", "FM.UI.IGEMovePersonReference");
            const BindingFlags Static = BindingFlags.Public | BindingFlags.Static;

            return new StaffTypes
            {
                _key = FmTypes.Member(key.GetConstructor(new[] { typeof(ulong) }), "Bindings.Key(ulong)"),
                _dataKey = dataKey,
                _dataKeyValue = FmTypes.Member(dataKey.GetField("m_key"), "Bindings.DataKey.m_key"),
                _property = FmTypes.Member(property.GetConstructor(new[] { typeof(uint) }), "PropertyID(uint)"),
                _rootKey = FmTypes.Member(bindings.GetMethod("get_RootKey", Static), "Bindings.RootKey"),
                _moveReference = FmTypes.Member(
                    move.GetConstructor(new[] { types.PairListType }),
                    "IGEMovePersonReference(List<Pair>)"),
                _createUnsigned = FmTypes.Member(typedValue.GetMethod("Create", new[] { typeof(ulong) }), "TypedValue.Create(ulong)"),
                _null = FmTypes.Member(typedValue.GetMethod("get_Null", Static), "TypedValue.Null"),
                AddNode = FmTypes.Member(subsystem.GetMethod("AddNode", new[] { key, key, property }), "AddNode"),
                AddData = FmTypes.Member(subsystem.GetMethod("AddData", new[] { dataKey }), "AddData"),
                SetTarget = FmTypes.Member(subsystem.GetMethod("SetTarget", new[] { key, dataKey }), "SetTarget"),
                SetData = FmTypes.Member(subsystem.GetMethod("SetData", new[] { dataKey, typedValue }), "SetData"),
                UpdateChannelData = FmTypes.Member(
                    subsystem.GetMethod("UpdateChannelData", new[] { key, typedValue }),
                    "UpdateChannelData"),
                CloseChannel = FmTypes.Member(subsystem.GetMethod("CloseChannel", new[] { key }), "CloseChannel"),
                ReleaseData = FmTypes.Member(subsystem.GetMethod("ReleaseData", new[] { dataKey }), "ReleaseData"),
                RemoveNode = FmTypes.Member(subsystem.GetMethod("RemoveNode", new[] { key }), "RemoveNode"),
            };
        }

        public object Key(ulong value) => _key.Invoke(new object[] { value });

        public object DataKey(ulong value)
        {
            var key = Activator.CreateInstance(_dataKey)!;
            _dataKeyValue.SetValue(key, value);
            return key;
        }

        public object Property(uint id) => _property.Invoke(new object[] { id });

        public object RootKey() => _rootKey.Invoke(null, null)!;

        public object Unsigned(ulong value) => _createUnsigned.Invoke(null, new object[] { value })!;

        public object Null() => _null.Invoke(null, null)!;

        public object NewMoveReference(FmTypes types, IEnumerable<(uint Id, int Value)> pairs) =>
            _moveReference.Invoke(new[] { types.NewPairList(pairs) });
    }
}
