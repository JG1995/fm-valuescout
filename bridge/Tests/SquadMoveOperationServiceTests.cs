using System.Buffers.Binary;
using FmDataBridge.Layouts;
using FmDataBridge.Mutations;
using FmDataBridge.Protocol;
using FmDataBridge.Scanning;
using FmDataBridge.Tests.Fakes;
using Xunit;

namespace FmDataBridge.Tests;

public sealed class SquadMoveOperationServiceTests
{
    private const string GameVersion = "27.1";
    private const string SourceRequestId = "scan-1";
    private const uint ClubUid = 1234;
    private const int ClubIndex = 439;
    private const int ManagerIndex = 357747;
    private static readonly IFmMemoryLayout Layout = Fm27Layout.Instance;

    [Fact]
    public void Moves_a_player_through_fm_and_confirms_his_new_team()
    {
        var club = new FakeClub();
        club.AddTeam(0, 77);
        club.AddTeam(10, 78);
        club.AddTeam(12, 79);

        var result = club.Service().Execute(club.Request((77, "reserve")), GameVersion, club.Reader);

        Assert.True(result.Succeeded);
        var outcome = Assert.Single(result.Result!.Moves);
        Assert.Equal(SquadMoveOutcome.Moved, outcome.Outcome);
        Assert.Equal(10, outcome.TeamType);
        Assert.Equal("reserve", outcome.TeamLevel);
        Assert.Equal(
            new SquadMoveAction(club.PersonIndex(77), ClubIndex, ManagerIndex, 0, 10),
            Assert.Single(club.Dispatcher.Sent));
    }

    [Theory]
    [InlineData(ManagerPointer.ObjectStart)]
    [InlineData(ManagerPointer.PersonRecord)]
    public void Finds_the_managers_person_record_wherever_the_team_points_into_his_object(ManagerPointer pointer)
    {
        var club = new FakeClub(pointer);
        club.AddTeam(0, 77);
        club.AddTeam(10);

        var result = club.Service().Execute(club.Request((77, "reserve")), GameVersion, club.Reader);

        Assert.True(result.Succeeded);
        Assert.Equal(ManagerIndex, Assert.Single(club.Dispatcher.Sent).HumanManagerIndex);
    }

    [Fact]
    public void Sends_nothing_for_a_player_already_at_the_target_level()
    {
        var club = new FakeClub();
        club.AddTeam(0, 77);
        club.AddTeam(10, 78);

        var result = club.Service().Execute(club.Request((78, "reserve")), GameVersion, club.Reader);

        Assert.True(result.Succeeded);
        Assert.Equal(SquadMoveOutcome.AlreadyThere, Assert.Single(result.Result!.Moves).Outcome);
        Assert.Empty(club.Dispatcher.Sent);
    }

    [Fact]
    public void Targets_the_team_at_the_level_that_holds_the_most_players()
    {
        var club = new FakeClub();
        club.AddTeam(0, 77);
        club.AddTeam(12, 80, 81);
        club.AddTeam(18, 82, 83, 84);
        club.AddTeam(22);

        var result = club.Service().Execute(club.Request((77, "youth")), GameVersion, club.Reader);

        Assert.Equal(18, Assert.Single(result.Result!.Moves).TeamType);
        Assert.Equal(18, Assert.Single(club.Dispatcher.Sent).ToTeamType);
    }

    [Fact]
    public void A_player_listed_twice_counts_for_his_higher_team_until_fm_moves_him()
    {
        var club = new FakeClub();
        club.AddTeam(0, 77);
        club.AddTeam(10, 79);
        club.AddTeam(12, 79);
        club.Dispatcher.Applies = false;

        var result = club.Service().Execute(club.Request((79, "youth")), GameVersion, club.Reader);

        var outcome = Assert.Single(result.Result!.Moves);
        Assert.Equal(SquadMoveOutcome.NotConfirmed, outcome.Outcome);
        Assert.Equal(10, outcome.TeamType);
        Assert.Equal(10, Assert.Single(club.Dispatcher.Sent).FromTeamType);
    }

    [Fact]
    public void Reports_each_player_it_cannot_move_and_still_moves_the_rest()
    {
        var club = new FakeClub();
        club.AddTeam(0, 77);
        club.AddTeam(12, 78);
        club.AddPlayer(90);

        var result = club.Service().Execute(
            club.Request((91, "youth"), (90, "youth"), (78, "reserve"), (77, "youth")),
            GameVersion,
            club.Reader);

        Assert.True(result.Succeeded);
        Assert.Equal(
            new[]
            {
                SquadMoveOutcome.NotFound,
                SquadMoveOutcome.NotInClubTeams,
                SquadMoveOutcome.NoTargetTeam,
                SquadMoveOutcome.Moved,
            },
            result.Result!.Moves.Select(move => move.Outcome));
        Assert.Equal(club.PersonIndex(77), Assert.Single(club.Dispatcher.Sent).PersonIndex);
    }

    [Fact]
    public void A_failed_dispatch_still_reports_the_moves_fm_made()
    {
        var club = new FakeClub();
        club.AddTeam(0, 77, 78);
        club.AddTeam(10);
        club.Dispatcher.AppliesFirstOnly = true;

        var result = club.Service().Execute(club.Request((77, "reserve"), (78, "reserve")), GameVersion, club.Reader);

        Assert.False(result.Succeeded);
        Assert.Equal(SquadMoveFailure.DispatchFailed, result.Failure);
        Assert.Equal("FM stopped", result.Detail);
        Assert.Equal(
            new[] { SquadMoveOutcome.Moved, SquadMoveOutcome.NotConfirmed },
            result.Result!.Moves.Select(move => move.Outcome));
    }

    [Theory]
    [InlineData("other-scan", nameof(SquadMoveFailure.SourceRequestMismatch))]
    [InlineData("other-club", nameof(SquadMoveFailure.ClubNotFound))]
    [InlineData("no-manager", nameof(SquadMoveFailure.ManagerNotFound))]
    [InlineData("manager-not-a-person", nameof(SquadMoveFailure.ManagerNotFound))]
    [InlineData("no-game-actions", nameof(SquadMoveFailure.GameActionsUnavailable))]
    [InlineData("no-layout", nameof(SquadMoveFailure.UnsupportedGameBuild))]
    public void Refuses_before_sending_anything_when_it_cannot_act_for_the_club(
        string defect,
        string expected)
    {
        var club = new FakeClub(
            defect switch
            {
                "no-manager" => ManagerPointer.None,
                "manager-not-a-person" => ManagerPointer.NotAPerson,
                _ => ManagerPointer.ObjectStart,
            });
        club.AddTeam(0, 77);
        club.AddTeam(10);
        club.Dispatcher.IsAvailable = defect != "no-game-actions";
        var request = club.Request((77, "reserve"));
        request = defect switch
        {
            "other-scan" => Rebuild(request, source: "scan-2"),
            "other-club" => Rebuild(request, clubUid: ClubUid + 1),
            _ => request,
        };

        var result = club.Service().Execute(request, defect == "no-layout" ? "28.0" : GameVersion, club.Reader);

        Assert.False(result.Succeeded);
        Assert.Equal(expected, result.Failure.ToString());
        Assert.Empty(club.Dispatcher.Sent);
    }

    private static BridgeRequest Rebuild(BridgeRequest request, string? source = null, uint? clubUid = null) =>
        new()
        {
            ProtocolVersion = request.ProtocolVersion,
            RequestId = request.RequestId,
            Operation = request.Operation,
            SourceRequestId = source ?? request.SourceRequestId,
            ClubUid = clubUid ?? request.ClubUid,
            SquadMoves = request.SquadMoves,
        };

    /// <summary>
    /// A club in fake FM memory: its object, its teams with their squad lists, the first team's
    /// manager, and players whose squad lists hold their block addresses.
    /// </summary>
    private sealed class FakeClub
    {
        private const ulong ClubAddress = 0x1000_0000;
        private const ulong TeamVectorAddress = 0x1100_0000;
        private const ulong FirstTeamAddress = 0x1200_0000;
        private const ulong FirstSquadAddress = 0x1300_0000;
        private const ulong ManagerAddress = 0x1400_0000;
        private const ulong ManagerVtable = 0x1500_0008;
        private const ulong ManagerTypeInfo = 0x1600_0000;
        private const int HumanManagerRecordOffset = 0x440;
        private const ulong FirstPlayerBlock = 0x2000_0000;
        private const int PlayerClassOffset = 0x288;
        private const int SquadCapacity = 64;

        private readonly byte[] _teamVector = new byte[8 * 32];
        private readonly List<Team> _teams = new();
        private readonly Dictionary<uint, PersonCandidate> _players = new();
        private readonly ulong _managerPointer;

        public FakeClub(ManagerPointer manager = ManagerPointer.ObjectStart)
        {
            _managerPointer = manager switch
            {
                ManagerPointer.None => 0,
                ManagerPointer.PersonRecord => ManagerAddress + HumanManagerRecordOffset,
                _ => ManagerAddress,
            };
            var club = new byte[0x100];
            WriteInt32(club, Layout.ObjectDatabaseIndexOffset, ClubIndex);
            WriteUInt32(club, Layout.ObjectUidOffset, ClubUid);
            WriteUInt64(club, Layout.ClubTeamsBeginOffset, TeamVectorAddress);
            WriteUInt64(club, Layout.ClubTeamsEndOffset, TeamVectorAddress);
            Reader.AddBytes(ClubAddress, club);
            Reader.AddBytes(TeamVectorAddress, _teamVector);
            _clubBytes = club;

            // The manager's person record sits 0x440 into his object; its vtable's type information
            // (MSVC: the word before the vtable points to a record whose +4 is the offset) says so.
            var managerObject = new byte[HumanManagerRecordOffset + 0x20];
            if (manager != ManagerPointer.NotAPerson)
            {
                WriteUInt64(managerObject, HumanManagerRecordOffset, ManagerVtable);
            }

            WriteInt32(managerObject, HumanManagerRecordOffset + Layout.ObjectDatabaseIndexOffset, ManagerIndex);
            WriteUInt32(managerObject, HumanManagerRecordOffset + Layout.ObjectUidOffset, 99);
            Reader.AddBytes(ManagerAddress, managerObject);
            Reader.AddBytes(ManagerVtable - 8, BitConverter.GetBytes(ManagerTypeInfo));
            Reader.AddBytes(ManagerTypeInfo, new byte[] { 0, 0, 0, 0 }.Concat(BitConverter.GetBytes(HumanManagerRecordOffset)).ToArray());

            Dispatcher = new FakeDispatcher(this);
        }

        private readonly byte[] _clubBytes;

        public FakeMemoryReader Reader { get; } = new();

        public FakeDispatcher Dispatcher { get; }

        public void AddTeam(int type, params uint[] players)
        {
            var slot = _teams.Count;
            var address = FirstTeamAddress + (ulong)slot * 0x1000;
            var squadAddress = FirstSquadAddress + (ulong)slot * 0x1000;
            var bytes = new byte[0x100];
            bytes[Layout.TeamTypeOffset] = (byte)type;
            WriteUInt64(bytes, Layout.TeamSquadBeginOffset, squadAddress);
            if (type == 0 && _managerPointer != 0)
            {
                WriteUInt64(bytes, Layout.TeamManagerPtrOffset, _managerPointer);
            }

            var team = new Team(type, address, bytes, squadAddress, new byte[8 * SquadCapacity]);
            Reader.AddBytes(address, bytes);
            Reader.AddBytes(squadAddress, team.SquadBytes);
            _teams.Add(team);
            WriteUInt64(_teamVector, slot * 8, address);
            WriteUInt64(_clubBytes, Layout.ClubTeamsEndOffset, TeamVectorAddress + (ulong)(_teams.Count * 8));

            foreach (var uid in players)
            {
                team.Squad.Add(AddPlayer(uid).BlockAddress);
            }

            team.Write();
        }

        public PersonCandidate AddPlayer(uint uid)
        {
            if (_players.TryGetValue(uid, out var existing))
            {
                return existing;
            }

            var block = FirstPlayerBlock + (ulong)_players.Count * 0x1000;
            var bytes = new byte[PlayerClassOffset + 0x20];
            WriteInt32(bytes, PlayerClassOffset + Layout.ObjectDatabaseIndexOffset, PersonIndex(uid));
            WriteUInt32(bytes, PlayerClassOffset + Layout.ObjectUidOffset, uid);
            Reader.AddBytes(block, bytes);
            var candidate = new PersonCandidate(
                block + PlayerClassOffset,
                block,
                uid,
                120,
                150,
                PlayerClassOffset,
                PersonFacet.Player);
            _players[uid] = candidate;
            return candidate;
        }

        public int PersonIndex(uint uid) => 200_000 + (int)uid;

        public SquadMoveOperationService Service()
        {
            var index = new PlayerMutationIndex();
            index.Replace(
                SourceRequestId,
                GameVersion,
                _players.Values.ToArray(),
                new Dictionary<uint, ulong> { [ClubUid] = ClubAddress });
            return new SquadMoveOperationService(
                new LayoutRegistry(new[] { Layout }),
                index,
                Dispatcher,
                confirmTimeout: TimeSpan.Zero,
                pollInterval: TimeSpan.Zero);
        }

        public BridgeRequest Request(params (uint Uid, string Level)[] moves) =>
            new()
            {
                ProtocolVersion = BridgeProtocol.ProtocolVersion,
                RequestId = "move-1",
                Operation = BridgeProtocol.OperationMoveSquadPlayers,
                SourceRequestId = SourceRequestId,
                ClubUid = ClubUid,
                SquadMoves = moves
                    .Select(move => new SquadMoveItem { PlayerUid = move.Uid, TargetLevel = move.Level })
                    .ToArray(),
            };

        /// <summary>What FM does with the action: takes the player off his team's list and adds him to the target's.</summary>
        public void Apply(SquadMoveAction action)
        {
            var player = _players.Values.Single(candidate => PersonIndex(candidate.Uid) == action.PersonIndex);
            foreach (var team in _teams.Where(team => team.Type == action.FromTeamType))
            {
                team.Squad.Remove(player.BlockAddress);
                team.Write();
            }

            var target = _teams.First(team => team.Type == action.ToTeamType);
            target.Squad.Add(player.BlockAddress);
            target.Write();
        }

        private sealed record Team(int Type, ulong Address, byte[] Bytes, ulong SquadAddress, byte[] SquadBytes)
        {
            public List<ulong> Squad { get; } = new();

            public void Write()
            {
                Array.Clear(SquadBytes);
                for (var i = 0; i < Squad.Count; i++)
                {
                    WriteUInt64(SquadBytes, i * 8, Squad[i]);
                }

                WriteUInt64(Bytes, Layout.TeamSquadEndOffset, SquadAddress + (ulong)(Squad.Count * 8));
            }
        }
    }

    public enum ManagerPointer
    {
        ObjectStart,
        PersonRecord,
        NotAPerson,
        None,
    }

    private sealed class FakeDispatcher : ISquadMoveDispatcher
    {
        private readonly FakeClub _club;

        public FakeDispatcher(FakeClub club) => _club = club;

        public bool IsAvailable { get; set; } = true;

        public bool Applies { get; set; } = true;

        public bool AppliesFirstOnly { get; set; }

        public List<SquadMoveAction> Sent { get; } = new();

        public bool TryDispatch(IReadOnlyList<SquadMoveAction> actions, out string? error)
        {
            Sent.AddRange(actions);
            if (AppliesFirstOnly)
            {
                _club.Apply(actions[0]);
                error = "FM stopped";
                return false;
            }

            if (Applies)
            {
                foreach (var action in actions)
                {
                    _club.Apply(action);
                }
            }

            error = null;
            return true;
        }
    }

    private static void WriteInt32(byte[] bytes, int offset, int value) =>
        BinaryPrimitives.WriteInt32LittleEndian(bytes.AsSpan(offset), value);

    private static void WriteUInt32(byte[] bytes, int offset, uint value) =>
        BinaryPrimitives.WriteUInt32LittleEndian(bytes.AsSpan(offset), value);

    private static void WriteUInt64(byte[] bytes, int offset, ulong value) =>
        BinaryPrimitives.WriteUInt64LittleEndian(bytes.AsSpan(offset), value);
}
