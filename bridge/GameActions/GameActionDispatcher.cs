using System.Collections.Concurrent;
using System.Reflection;
using BepInEx;
using BepInEx.Logging;
using FmDataBridge.Mutations;
using HarmonyLib;

namespace FmDataBridge.GameActions;

/// <summary>
/// Sends FM27's own squad-change action, the one its "Move to" menu sends, on the game's main
/// thread. FM's types are found at run time so the bridge builds without the game's assemblies;
/// see the FM27 squad move action note in the wiki for the action's format.
/// </summary>
internal sealed class GameActionDispatcher : ISquadMoveDispatcher
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

    private static readonly TimeSpan DispatchTimeout = TimeSpan.FromSeconds(10);
    private static GameActionDispatcher? s_active;

    private readonly ConcurrentQueue<Work> _queue = new();
    private FmTypes? _types;
    private object? _subsystem;

    public bool IsAvailable => _types is not null;

    /// <summary>Finds FM's interop types and hooks its interop subsystem. Leaves the dispatcher unavailable on failure.</summary>
    public void Install(ManualLogSource log)
    {
        try
        {
            var types = FmTypes.Resolve();
            var harmony = new Harmony("com.fmvaluescout.databridge.gameactions");
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
        }
    }

    public bool TryDispatch(IReadOnlyList<SquadMoveAction> actions, out string? error)
    {
        if (_types is null)
        {
            error = "FM's squad action is unavailable in this game build";
            return false;
        }

        if (Volatile.Read(ref _subsystem) is null)
        {
            error = "FM has not opened a game screen yet";
            return false;
        }

        var work = new Work(actions);
        _queue.Enqueue(work);
        if (!work.Done.Wait(DispatchTimeout) && work.TryCancel())
        {
            error = "FM did not run the squad moves; open a game screen and retry";
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

    // Harmony postfix on the subsystem's Tick, which runs on the game's main thread.
    private static void RunQueuedWork(object __instance)
    {
        if (s_active is not { } dispatcher)
        {
            return;
        }

        Volatile.Write(ref dispatcher._subsystem, __instance);
        while (dispatcher._queue.TryDequeue(out var work))
        {
            if (!work.TryStart())
            {
                continue;
            }

            try
            {
                foreach (var action in work.Actions)
                {
                    dispatcher.Send(__instance, action);
                }
            }
            catch (Exception ex)
            {
                work.Error = $"FM rejected a squad move: {(ex.InnerException ?? ex).Message}";
            }
            finally
            {
                work.Done.Set();
            }
        }
    }

    private void Send(object subsystem, SquadMoveAction action)
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

    /// <summary>FM's database reference: a record holding the table in the top byte and the index below.</summary>
    private static object Dref(FmTypes types, long table, int index) =>
        types.NewRecord(new (uint, object)[] { (DrefKey, types.Number((table << 24) | (uint)index)) });

    private sealed class Work
    {
        private int _state; // 0 queued, 1 started, 2 cancelled

        public Work(IReadOnlyList<SquadMoveAction> actions) => Actions = actions;

        public IReadOnlyList<SquadMoveAction> Actions { get; }

        public ManualResetEventSlim Done { get; } = new();

        public string? Error { get; set; }

        public bool TryStart() => Interlocked.CompareExchange(ref _state, 1, 0) == 0;

        public bool TryCancel() => Interlocked.CompareExchange(ref _state, 2, 0) == 0;
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
                _createRecord = create.MakeGenericMethod(record),
                _createList = create.MakeGenericMethod(typedValueList),
            };
        }

        public object Number(long value) => _createNumber.Invoke(null, new object[] { value })!;

        public object NewActionReference(IEnumerable<(uint Id, int Value)> pairs)
        {
            var list = Activator.CreateInstance(_pairList)!;
            foreach (var (id, value) in pairs)
            {
                var pair = Activator.CreateInstance(_pair)!;
                _pairId.SetValue(pair, id);
                _pairValue.SetValue(pair, value);
                _pairListAdd.Invoke(list, new[] { pair });
            }

            return _actionReference.Invoke(new[] { list });
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

        private static Type Find(string assemblyName, string typeName)
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

        private static T Member<T>(T? member, string name)
            where T : class =>
            member ?? throw new InvalidOperationException($"FM member {name} not found");
    }
}
