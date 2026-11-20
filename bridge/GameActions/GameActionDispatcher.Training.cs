using System.Reflection;
using BepInEx.Logging;
using FmDataBridge.StaffLimits;
using FmDataBridge.Training;

namespace FmDataBridge.GameActions;

/// <summary>
/// Individual training. FM's Individual Training screen sends one event per change on the player's
/// PlayerTrainingReference; FM needs only the confirmation flag and the new selection in its data.
/// The training is then read back through the same properties the screen reads. See the FM27
/// training actions note in the wiki.
/// </summary>
internal sealed partial class GameActionDispatcher : ITrainingDispatcher
{
    private const int PlayerTrainingReferenceType = 77;
    private const uint TrainingScheduleKey = 1416196963; // "Tisc"; FM ignores the events without it
    private const int TrainingSchedule = 1;
    private const uint ConfirmedChangeKey = 1215644527; // "HuCo"; FM ignores the events without it
    private const uint FocusChangedEvent = 1280659764; // EventFmxPlayerTrainingFocusChanged
    private const uint InPossessionChangedEvent = 2052018511; // EventFmxPlayerTrainingPositionChanged
    private const uint OutOfPossessionChangedEvent = 1176516205; // EventFmxPlayerOOPTrainingRoleChanged

    // FM applies the events on its own thread; the read-back opens once it has had time to.
    private static readonly TimeSpan TrainingSettle = TimeSpan.FromMilliseconds(500);

    private static readonly uint[] TrainingProperties =
    {
        TrainingParser.InPossessionProperty,
        TrainingParser.OutOfPossessionProperty,
        TrainingParser.FocusProperty,
    };

    private ConstructorInfo? _trainingReference;

    public bool TrainingAvailable => StaffLimitsAvailable && _trainingReference is not null;

    private void InstallTraining(ManualLogSource log)
    {
        try
        {
            var reference = FmTypes.Find("FM.UI", "FM.UI.PlayerTrainingReference");
            _trainingReference = FmTypes.Member(
                reference.GetConstructor(new[] { _types!.PairListType }),
                "PlayerTrainingReference(List<Pair>)");
            log.LogInfo("FM training changes are available");
        }
        catch (Exception ex)
        {
            _trainingReference = null;
            log.LogWarning($"FM training changes are unavailable: {ex.Message}");
        }
    }

    public bool TryApplyTraining(
        int humanManagerIndex,
        IReadOnlyList<TrainingAction> actions,
        out IReadOnlyDictionary<(int PersonIndex, uint Property), FmValue> values,
        out string? error)
    {
        values = new Dictionary<(int PersonIndex, uint Property), FmValue>();
        if (!TrainingAvailable)
        {
            error = "FM's training changes are unavailable in this game build";
            return false;
        }

        var types = _types!;
        var staff = _staffTypes!;
        object Reference(int personIndex) => _trainingReference!.Invoke(new[]
        {
            types.NewPairList(new[]
            {
                (ReferenceTypeKey, PlayerTrainingReferenceType),
                (MoveHumanKey, (int)((PersonTable << 24) | (uint)humanManagerIndex)),
                (MovePersonKey, (int)((PersonTable << 24) | (uint)personIndex)),
                (TrainingScheduleKey, TrainingSchedule),
            }),
        });

        void Send(object subsystem, uint eventId, int personIndex, uint selectionKey, object selection) =>
            types.SendEvent.Invoke(subsystem, new[]
            {
                (object)eventId,
                Reference(personIndex),
                types.NewRecord(new (uint, object)[]
                {
                    (ConfirmedChangeKey, types.Boolean(true)),
                    (selectionKey, types.NewRecord(new (uint, object)[] { (OptionValueKey, selection) })),
                }),
            });

        var work = new Work();
        work.Then(TimeSpan.Zero, subsystem =>
        {
            foreach (var action in actions)
            {
                if (action is { IpPosition: { } position, IpRole: { } role })
                {
                    var selection = types.NewRecord(new (uint, object)[]
                    {
                        (TrainingParser.PositionKey, staff.Unsigned((ulong)position)),
                        (OptionValueKey, staff.Unsigned(role)),
                    });
                    Send(subsystem, InPossessionChangedEvent, action.PersonIndex, TrainingParser.InPossessionKey, selection);
                }

                if (action.OopRole is { } oop)
                {
                    Send(subsystem, OutOfPossessionChangedEvent, action.PersonIndex, TrainingParser.OutOfPossessionKey, types.Number(oop));
                }

                if (action.Focus is { } focus)
                {
                    Send(subsystem, FocusChangedEvent, action.PersonIndex, TrainingParser.FocusKey, types.Number(focus));
                }
            }
        });
        var read = AddChannelRead(
            work,
            actions.SelectMany(action => TrainingProperties.Select(property => (action.PersonIndex, property))).ToArray(),
            Reference,
            TrainingSettle);

        if (!Run(work, "training changes", out error, TrainingSettle + ChannelReadLongest))
        {
            return false;
        }

        values = read.Values();
        return true;
    }
}
