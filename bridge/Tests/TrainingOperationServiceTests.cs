using System.Buffers.Binary;
using FmDataBridge.Layouts;
using FmDataBridge.Mutations;
using FmDataBridge.Protocol;
using FmDataBridge.Scanning;
using FmDataBridge.StaffLimits;
using FmDataBridge.Training;
using FmDataBridge.Tests.Fakes;
using Xunit;

namespace FmDataBridge.Tests;

public sealed class TrainingOperationServiceTests
{
    private const string GameVersion = "27.1";
    private const string SourceRequestId = "scan-1";
    private const uint ClubUid = 1234;
    private const int ManagerIndex = 357747;
    private const ulong HalfBack = 34359738368;
    private static readonly IFmMemoryLayout Layout = Fm27Layout.Instance;

    [Fact]
    public void Sends_each_players_training_and_confirms_it_from_what_fm_holds_afterwards()
    {
        var fm = new FakeFm();

        var result = fm.Service().Execute(
            fm.Request(Change(77, ipPosition: 128, ipRole: HalfBack, oopRole: -14, focus: 19), Change(78, focus: -1)),
            GameVersion,
            fm.Reader);

        Assert.True(result.Succeeded);
        Assert.Equal(ManagerIndex, fm.Dispatcher.ManagerIndex);
        Assert.Equal(
            new[]
            {
                new TrainingAction(fm.PersonIndex(77), 128, HalfBack, -14, 19),
                new TrainingAction(fm.PersonIndex(78), null, null, null, -1),
            },
            fm.Dispatcher.Sent);
        var (halfBack, cleared) = (result.Result!.Players[0], result.Result.Players[1]);
        Assert.Equal(
            (TrainingOutcome.Applied, (int?)128, (ulong?)HalfBack, "Role 34359738368", (int?)-14, "OOP -14", (int?)19, "Focus 19"),
            (halfBack.Outcome, halfBack.IpPosition, halfBack.IpRole, halfBack.IpRoleName, halfBack.OopRole, halfBack.OopRoleName, halfBack.Focus, halfBack.FocusName));
        Assert.Equal((TrainingOutcome.Applied, (int?)-1), (cleared.Outcome, cleared.Focus));
    }

    [Fact]
    public void A_part_fm_did_not_take_leaves_the_player_unconfirmed_with_what_fm_holds()
    {
        var fm = new FakeFm();
        fm.Dispatcher.IgnoresOutOfPossession = true;

        var result = fm.Service().Execute(fm.Request(Change(77, ipPosition: 16, ipRole: 2, oopRole: -31)), GameVersion, fm.Reader);

        var outcome = Assert.Single(result.Result!.Players);
        Assert.Equal(
            (TrainingOutcome.NotConfirmed, (ulong?)2, (int?)-28),
            (outcome.Outcome, outcome.IpRole, outcome.OopRole));
    }

    [Fact]
    public void Never_sends_training_for_a_player_who_is_not_one_of_the_clubs_own()
    {
        var fm = new FakeFm();

        var result = fm.Service().Execute(
            fm.Request(Change(90, focus: 19), Change(555, focus: 19), Change(78, focus: 20)),
            GameVersion,
            fm.Reader);

        Assert.True(result.Succeeded);
        Assert.Equal(
            new[] { TrainingOutcome.NotAtClub, TrainingOutcome.NotFound, TrainingOutcome.Applied },
            result.Result!.Players.Select(player => player.Outcome));
        Assert.Equal(fm.PersonIndex(78), Assert.Single(fm.Dispatcher.Sent).PersonIndex);
    }

    [Fact]
    public void A_failed_dispatch_reports_every_sent_player_as_unconfirmed()
    {
        var fm = new FakeFm();
        fm.Dispatcher.Error = "FM did not run the training changes";

        var result = fm.Service().Execute(fm.Request(Change(77, focus: 19), Change(555, focus: 19)), GameVersion, fm.Reader);

        Assert.False(result.Succeeded);
        Assert.Equal(TrainingFailure.DispatchFailed, result.Failure);
        Assert.Equal("FM did not run the training changes", result.Detail);
        Assert.Equal(
            new[] { TrainingOutcome.NotConfirmed, TrainingOutcome.NotFound },
            result.Result!.Players.Select(player => player.Outcome));
    }

    [Theory]
    [InlineData("other-scan", nameof(TrainingFailure.SourceRequestMismatch))]
    [InlineData("other-club", nameof(TrainingFailure.ClubNotFound))]
    [InlineData("no-training", nameof(TrainingFailure.TrainingUnavailable))]
    [InlineData("no-manager", nameof(TrainingFailure.ManagerNotFound))]
    [InlineData("no-layout", nameof(TrainingFailure.UnsupportedGameBuild))]
    public void Refuses_before_sending_anything_when_it_cannot_act_for_the_club(string defect, string expected)
    {
        var fm = new FakeFm(withManager: defect != "no-manager");
        fm.Dispatcher.TrainingAvailable = defect != "no-training";
        var request = fm.Request(Change(77, focus: 19));
        request = new BridgeRequest
        {
            ProtocolVersion = request.ProtocolVersion,
            RequestId = request.RequestId,
            Operation = request.Operation,
            SourceRequestId = defect == "other-scan" ? "scan-2" : SourceRequestId,
            ClubUid = defect == "other-club" ? ClubUid + 1 : ClubUid,
            TrainingChanges = request.TrainingChanges,
        };

        var result = fm.Service().Execute(request, defect == "no-layout" ? "28.0" : GameVersion, fm.Reader);

        Assert.False(result.Succeeded);
        Assert.Equal(expected, result.Failure.ToString());
        Assert.Null(result.Result);
        Assert.Empty(fm.Dispatcher.Sent);
    }

    private static TrainingChangeItem Change(
        uint uid,
        int? ipPosition = null,
        ulong? ipRole = null,
        int? oopRole = null,
        int? focus = null) =>
        new() { PlayerUid = uid, IpPosition = ipPosition, IpRole = ipRole, OopRole = oopRole, Focus = focus };

    /// <summary>
    /// Fake FM memory: the managed club with a first team (players 77 and 78) and its manager, and
    /// player 90, who plays for no team of the club.
    /// </summary>
    private sealed class FakeFm
    {
        private const ulong ClubAddress = 0x1000_0000;
        private const ulong TeamVectorAddress = 0x1100_0000;
        private const ulong FirstTeamAddress = 0x1200_0000;
        private const ulong SquadAddress = 0x1300_0000;
        private const ulong ManagerAddress = 0x1400_0000;
        private const ulong ManagerVtable = 0x1500_0008;
        private const ulong ManagerTypeInfo = 0x1600_0000;
        private const int HumanManagerRecordOffset = 0x440;
        private const ulong FirstPlayerBlock = 0x2000_0000;
        private const int PlayerClassOffset = 0x288;

        private readonly List<PersonCandidate> _players = new();

        public FakeFm(bool withManager = true)
        {
            var club = new byte[0x100];
            WriteUInt32(club, Layout.ObjectUidOffset, ClubUid);
            WriteUInt64(club, Layout.ClubTeamsBeginOffset, TeamVectorAddress);
            WriteUInt64(club, Layout.ClubTeamsEndOffset, TeamVectorAddress + 8);
            Reader.AddBytes(ClubAddress, club);
            Reader.AddBytes(TeamVectorAddress, BitConverter.GetBytes(FirstTeamAddress));

            var squad = new[] { AddPlayer(77), AddPlayer(78) };
            AddPlayer(90);
            var team = new byte[0x100];
            WriteUInt64(team, Layout.TeamSquadBeginOffset, SquadAddress);
            WriteUInt64(team, Layout.TeamSquadEndOffset, SquadAddress + (ulong)(squad.Length * 8));
            WriteUInt64(team, Layout.TeamManagerPtrOffset, withManager ? ManagerAddress : 0);
            Reader.AddBytes(FirstTeamAddress, team);
            Reader.AddBytes(SquadAddress, squad.SelectMany(player => BitConverter.GetBytes(player.BlockAddress)).ToArray());

            // The manager's person record sits 0x440 into his object; its vtable's type information
            // (MSVC: the word before the vtable points to a record whose +4 is the offset) says so.
            var manager = new byte[HumanManagerRecordOffset + 0x20];
            WriteUInt64(manager, HumanManagerRecordOffset, ManagerVtable);
            WriteUInt32(manager, HumanManagerRecordOffset + Layout.ObjectDatabaseIndexOffset, ManagerIndex);
            WriteUInt32(manager, HumanManagerRecordOffset + Layout.ObjectUidOffset, 99);
            Reader.AddBytes(ManagerAddress, manager);
            Reader.AddBytes(ManagerVtable - 8, BitConverter.GetBytes(ManagerTypeInfo));
            Reader.AddBytes(ManagerTypeInfo, new byte[] { 0, 0, 0, 0 }.Concat(BitConverter.GetBytes(HumanManagerRecordOffset)).ToArray());
        }

        public FakeMemoryReader Reader { get; } = new();

        public FakeDispatcher Dispatcher { get; } = new();

        public int PersonIndex(uint uid) => 200_000 + (int)uid;

        public TrainingOperationService Service()
        {
            var index = new PlayerMutationIndex();
            index.Replace(SourceRequestId, GameVersion, _players.ToArray(), new Dictionary<uint, ulong> { [ClubUid] = ClubAddress });
            return new TrainingOperationService(new LayoutRegistry(new[] { Layout }), index, Dispatcher);
        }

        public BridgeRequest Request(params TrainingChangeItem[] changes) =>
            new()
            {
                ProtocolVersion = BridgeProtocol.ProtocolVersion,
                RequestId = "training-1",
                Operation = BridgeProtocol.OperationApplyTraining,
                SourceRequestId = SourceRequestId,
                ClubUid = ClubUid,
                TrainingChanges = changes,
            };

        private PersonCandidate AddPlayer(uint uid)
        {
            var block = FirstPlayerBlock + (ulong)_players.Count * 0x1000;
            var bytes = new byte[PlayerClassOffset + 0x20];
            WriteUInt32(bytes, PlayerClassOffset + Layout.ObjectDatabaseIndexOffset, (uint)PersonIndex(uid));
            WriteUInt32(bytes, PlayerClassOffset + Layout.ObjectUidOffset, uid);
            Reader.AddBytes(block, bytes);
            var candidate = new PersonCandidate(block + PlayerClassOffset, block, uid, 120, 150, PlayerClassOffset, PersonFacet.Player);
            _players.Add(candidate);
            return candidate;
        }
    }

    /// <summary>
    /// Plays FM: each player's training starts at D (C) Centre-Back, Centre-Back out of possession and
    /// no focus; the events change it, and the read-back reports it as FM's screens receive it.
    /// </summary>
    private sealed class FakeDispatcher : ITrainingDispatcher
    {
        public bool TrainingAvailable { get; set; } = true;

        public bool IgnoresOutOfPossession { get; set; }

        public string? Error { get; set; }

        public int? ManagerIndex { get; private set; }

        public List<TrainingAction> Sent { get; } = new();

        public bool TryApplyTraining(
            int humanManagerIndex,
            IReadOnlyList<TrainingAction> actions,
            out IReadOnlyDictionary<(int PersonIndex, uint Property), FmValue> values,
            out string? error)
        {
            ManagerIndex = humanManagerIndex;
            Sent.AddRange(actions);
            var read = new Dictionary<(int PersonIndex, uint Property), FmValue>();
            foreach (var action in actions)
            {
                var oop = IgnoresOutOfPossession ? -28 : action.OopRole ?? -28;
                // FM sends the held in-possession option flat, with its position beside the role.
                read[(action.PersonIndex, TrainingParser.InPossessionProperty)] = Property(
                    TrainingParser.InPossessionKey,
                    new FmRecord(new Dictionary<uint, FmValue>
                    {
                        [TrainingParser.PositionKey] = new FmNumber(action.IpPosition ?? 16),
                        [TrainingParser.OptionValueKey] = new FmNumber((long)(action.IpRole ?? 2)),
                        [TrainingParser.OptionNameKey] = new FmText($"Role {action.IpRole ?? 2}"),
                    }));
                read[(action.PersonIndex, TrainingParser.OutOfPossessionProperty)] =
                    Property(TrainingParser.OutOfPossessionKey, Option(new FmNumber(oop), $"OOP {oop}"));
                read[(action.PersonIndex, TrainingParser.FocusProperty)] =
                    Property(TrainingParser.FocusKey, Option(new FmNumber(action.Focus ?? -1), $"Focus {action.Focus ?? -1}"));
            }

            values = read;
            error = Error;
            return Error is null;
        }

        private static FmRecord Option(FmValue value, string name) =>
            new(new Dictionary<uint, FmValue>
            {
                [TrainingParser.OptionValueKey] = value,
                [TrainingParser.OptionNameKey] = new FmText(name),
            });

        // FM's property value carries more than the selection, such as its option list.
        private static FmRecord Property(uint key, FmRecord selection) =>
            new(new Dictionary<uint, FmValue>
            {
                [key] = selection,
                [1215644527] = new FmNumber(1), // "HuCo"
                [1145138499] = new FmList(Array.Empty<FmValue>()), // "DamC"
            });
    }

    private static void WriteUInt32(byte[] bytes, int offset, uint value) =>
        BinaryPrimitives.WriteUInt32LittleEndian(bytes.AsSpan(offset), value);

    private static void WriteUInt64(byte[] bytes, int offset, ulong value) =>
        BinaryPrimitives.WriteUInt64LittleEndian(bytes.AsSpan(offset), value);
}
