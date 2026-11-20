using System.Buffers.Binary;
using FmDataBridge.Layouts;
using FmDataBridge.Mutations;
using FmDataBridge.Protocol;
using FmDataBridge.Scanning;
using FmDataBridge.Tests.Fakes;
using Xunit;

namespace FmDataBridge.Tests;

public sealed class StaffChangeOperationServiceTests
{
    private const string GameVersion = "27.1";
    private const string SourceRequestId = "scan-1";
    private const uint ClubUid = 680;
    private const uint OtherClubUid = 681;
    private const int ManagerIndex = 357747;
    private const int FirstTeamIndex = 565;
    private const int YouthTeamIndex = 50755;
    private static readonly IFmMemoryLayout Layout = Fm27Layout.Instance;

    [Fact]
    public void Sends_every_sack_before_any_hire_and_reports_each_new_contract()
    {
        var fm = new FakeFm();
        fm.AddStaff(41, ClubUid, team: 0, job: 2, wage: 1549);
        fm.AddStaff(42, ClubUid, team: 0, job: 20, wage: 4906);
        fm.AddStaff(43, OtherClubUid, team: 0, job: 14, wage: 3043);

        var result = fm.Service().Execute(
            fm.Request(Hire(43, 20, "senior"), Change(42, 2, "youth"), Sack(41)),
            GameVersion,
            fm.Reader);

        Assert.True(result.Succeeded);
        Assert.Equal(
            new[]
            {
                new StaffAction(StaffActionKind.Sack, fm.PersonIndex(42), ManagerIndex, FirstTeamIndex),
                new StaffAction(StaffActionKind.Sack, fm.PersonIndex(41), ManagerIndex, FirstTeamIndex),
                new StaffAction(StaffActionKind.Hire, fm.PersonIndex(43), ManagerIndex, FirstTeamIndex, FirstTeamIndex, 20),
                new StaffAction(StaffActionKind.Hire, fm.PersonIndex(42), ManagerIndex, YouthTeamIndex, FirstTeamIndex, 2),
            },
            fm.Dispatcher.Sent);

        var outcomes = result.Result!.Changes;
        Assert.Equal(StaffChangeOutcome.Hired, outcomes[0].Outcome);
        Assert.Equal((ClubUid, 20, 0, "senior"), (outcomes[0].ClubUid, outcomes[0].JobId, outcomes[0].TeamType, outcomes[0].TeamLevel));
        Assert.Equal(FakeFm.NewWage(20), outcomes[0].WeeklyWageGbp);
        Assert.Equal((2031, 181), (outcomes[0].ContractExpiryYear, outcomes[0].ContractExpiryDayOfYear));

        Assert.Equal(StaffChangeOutcome.Changed, outcomes[1].Outcome);
        Assert.Equal((ClubUid, 2, 12, "youth"), (outcomes[1].ClubUid, outcomes[1].JobId, outcomes[1].TeamType, outcomes[1].TeamLevel));

        Assert.Equal(StaffChangeOutcome.Sacked, outcomes[2].Outcome);
        Assert.Null(outcomes[2].ClubUid);
        Assert.Null(outcomes[2].JobId);
    }

    [Fact]
    public void A_change_whose_rehire_fm_ignores_leaves_the_person_unemployed()
    {
        var fm = new FakeFm();
        fm.AddStaff(42, ClubUid, team: 0, job: 20, wage: 4906);
        fm.AddStaff(43, OtherClubUid, team: 0, job: 14, wage: 3043);
        fm.Dispatcher.AppliesHires = false;

        var result = fm.Service().Execute(
            fm.Request(Change(42, 2, "youth"), Hire(43, 14, "senior")),
            GameVersion,
            fm.Reader);

        Assert.True(result.Succeeded);
        Assert.Equal(StaffChangeOutcome.LeftUnemployed, result.Result!.Changes[0].Outcome);
        Assert.Equal(StaffChangeOutcome.NotConfirmed, result.Result.Changes[1].Outcome);
        Assert.Equal(OtherClubUid, result.Result.Changes[1].ClubUid);
    }

    [Fact]
    public void Hires_nobody_into_a_wrong_job()
    {
        var fm = new FakeFm();
        fm.AddStaff(43, OtherClubUid, team: 0, job: 14, wage: 3043);
        fm.Dispatcher.HiredJob = 44;

        var result = fm.Service().Execute(fm.Request(Hire(43, 2, "senior")), GameVersion, fm.Reader);

        var outcome = Assert.Single(result.Result!.Changes);
        Assert.Equal(StaffChangeOutcome.NotConfirmed, outcome.Outcome);
        Assert.Equal(44, outcome.JobId);
    }

    [Fact]
    public void Reports_each_change_it_cannot_make_and_sends_no_sack_for_a_change_without_a_team()
    {
        var fm = new FakeFm();
        fm.AddStaff(41, ClubUid, team: 0, job: 2, wage: 1549);
        fm.AddStaff(43, OtherClubUid, team: 0, job: 14, wage: 3043);

        var result = fm.Service().Execute(
            fm.Request(Sack(43), Change(41, 2, "reserve"), Sack(99)),
            GameVersion,
            fm.Reader);

        Assert.True(result.Succeeded);
        Assert.Equal(
            new[] { StaffChangeOutcome.NotAtClub, StaffChangeOutcome.NoTargetTeam, StaffChangeOutcome.NotFound },
            result.Result!.Changes.Select(change => change.Outcome));
        Assert.Equal(ClubUid, result.Result.Changes[1].ClubUid);
        Assert.Empty(fm.Dispatcher.Sent);
    }

    [Theory]
    [InlineData("other-scan", nameof(StaffChangeFailure.SourceRequestMismatch))]
    [InlineData("other-club", nameof(StaffChangeFailure.ClubNotFound))]
    [InlineData("no-manager", nameof(StaffChangeFailure.ManagerNotFound))]
    [InlineData("no-game-actions", nameof(StaffChangeFailure.GameActionsUnavailable))]
    [InlineData("no-layout", nameof(StaffChangeFailure.UnsupportedGameBuild))]
    public void Refuses_before_sending_anything_when_it_cannot_act_for_the_club(string defect, string expected)
    {
        var fm = new FakeFm(withManager: defect != "no-manager");
        fm.AddStaff(41, ClubUid, team: 0, job: 2, wage: 1549);
        fm.Dispatcher.StaffActionsAvailable = defect != "no-game-actions";
        var request = fm.Request(Sack(41));
        request = new BridgeRequest
        {
            ProtocolVersion = request.ProtocolVersion,
            RequestId = request.RequestId,
            Operation = request.Operation,
            SourceRequestId = defect == "other-scan" ? "scan-2" : request.SourceRequestId,
            ClubUid = defect == "other-club" ? ClubUid + 7 : request.ClubUid,
            StaffChanges = request.StaffChanges,
        };

        var result = fm.Service().Execute(request, defect == "no-layout" ? "28.0" : GameVersion, fm.Reader);

        Assert.False(result.Succeeded);
        Assert.Equal(expected, result.Failure.ToString());
        Assert.Empty(fm.Dispatcher.Sent);
    }

    private static StaffChangeItem Sack(uint uid) => new() { StaffUid = uid, Kind = StaffChangeItem.Sack };

    private static StaffChangeItem Hire(uint uid, int job, string level) =>
        new() { StaffUid = uid, Kind = StaffChangeItem.Hire, JobType = job, TargetLevel = level };

    private static StaffChangeItem Change(uint uid, int job, string level) =>
        new() { StaffUid = uid, Kind = StaffChangeItem.Change, JobType = job, TargetLevel = level };

    /// <summary>
    /// Fake FM memory: the managed club with a first team, its manager and a youth team; another
    /// club; and staff whose person objects point at contracts that name a team.
    /// </summary>
    private sealed class FakeFm
    {
        private const ulong ClubAddress = 0x1000_0000;
        private const ulong OtherClubAddress = 0x1010_0000;
        private const ulong TeamVectorAddress = 0x1100_0000;
        private const ulong FirstTeamAddress = 0x1200_0000;
        private const ulong YouthTeamAddress = 0x1201_0000;
        private const ulong OtherTeamAddress = 0x1210_0000;
        private const ulong ManagerAddress = 0x1400_0000;
        private const ulong ManagerVtable = 0x1500_0008;
        private const ulong ManagerTypeInfo = 0x1600_0000;
        private const int HumanManagerRecordOffset = 0x440;
        private const ulong FirstPerson = 0x2000_0000;
        private const ulong FirstContract = 0x3000_0000;
        private const uint ExpiryJune2031 = (2031u << 16) | 181u;

        private readonly Dictionary<uint, (ulong Address, byte[] Bytes)> _staff = new();
        private readonly Dictionary<int, ulong> _teamsByIndex = new();
        private int _contracts;

        public FakeFm(bool withManager = true)
        {
            Reader.AddBytes(ClubAddress, Club(ClubUid, 439, teams: 2));
            Reader.AddBytes(OtherClubAddress, Club(OtherClubUid, 440, teams: 0));
            var vector = new byte[16];
            WriteUInt64(vector, 0, FirstTeamAddress);
            WriteUInt64(vector, 8, YouthTeamAddress);
            Reader.AddBytes(TeamVectorAddress, vector);
            AddTeam(FirstTeamAddress, 0, FirstTeamIndex, ClubAddress, withManager ? ManagerAddress : 0);
            AddTeam(YouthTeamAddress, 12, YouthTeamIndex, ClubAddress, 0);
            AddTeam(OtherTeamAddress, 0, 777, OtherClubAddress, 0);

            // The manager's person record sits 0x440 into his object; its vtable's type information
            // (MSVC: the word before the vtable points to a record whose +4 is the offset) says so.
            var manager = new byte[HumanManagerRecordOffset + 0x20];
            WriteUInt64(manager, HumanManagerRecordOffset, ManagerVtable);
            WriteInt32(manager, HumanManagerRecordOffset + Layout.ObjectDatabaseIndexOffset, ManagerIndex);
            WriteUInt32(manager, HumanManagerRecordOffset + Layout.ObjectUidOffset, 99);
            Reader.AddBytes(ManagerAddress, manager);
            Reader.AddBytes(ManagerVtable - 8, BitConverter.GetBytes(ManagerTypeInfo));
            Reader.AddBytes(ManagerTypeInfo, new byte[] { 0, 0, 0, 0 }.Concat(BitConverter.GetBytes(HumanManagerRecordOffset)).ToArray());

            Dispatcher = new FakeDispatcher(this);
        }

        public FakeMemoryReader Reader { get; } = new();

        public FakeDispatcher Dispatcher { get; }

        public static long NewWage(int job) => 1000 + job;

        public int PersonIndex(uint uid) => 100_000 + (int)uid;

        public void AddStaff(uint uid, uint clubUid, int team, int job, long wage)
        {
            var address = FirstPerson + (ulong)_staff.Count * 0x1000;
            var bytes = new byte[0x200];
            WriteInt32(bytes, Layout.ObjectDatabaseIndexOffset, PersonIndex(uid));
            WriteUInt32(bytes, Layout.ObjectUidOffset, uid);
            Reader.AddBytes(address, bytes);
            _staff[uid] = (address, bytes);
            var teamAddress = clubUid == ClubUid
                ? (team == 0 ? FirstTeamAddress : YouthTeamAddress)
                : OtherTeamAddress;
            SetContract(uid, teamAddress, job, wage, (2030u << 16) | 181u);
        }

        public StaffChangeOperationService Service()
        {
            var players = new PlayerMutationIndex();
            players.Replace(
                SourceRequestId,
                GameVersion,
                new[] { new PersonCandidate(0x9000_0000, 0x9000_0000, 7, 100, 120, 0x288, PersonFacet.Player) },
                new Dictionary<uint, ulong> { [ClubUid] = ClubAddress, [OtherClubUid] = OtherClubAddress });
            var staff = new StaffMutationIndex();
            staff.Replace(
                SourceRequestId,
                GameVersion,
                _staff.Select(entry => new PersonCandidate(
                    entry.Value.Address,
                    entry.Value.Address,
                    entry.Key,
                    120,
                    150,
                    0x100,
                    PersonFacet.Staff)).ToArray());
            return new StaffChangeOperationService(
                new LayoutRegistry(new[] { Layout }),
                players,
                staff,
                Dispatcher,
                confirmTimeout: TimeSpan.Zero,
                pollInterval: TimeSpan.Zero);
        }

        public BridgeRequest Request(params StaffChangeItem[] changes) =>
            new()
            {
                ProtocolVersion = BridgeProtocol.ProtocolVersion,
                RequestId = "staff-1",
                Operation = BridgeProtocol.OperationApplyStaffChanges,
                SourceRequestId = SourceRequestId,
                ClubUid = ClubUid,
                StaffChanges = changes,
            };

        /// <summary>What FM does: a sack ends the contract; a hire gives a new one for the job at the team.</summary>
        public void Apply(StaffAction action, int? hiredJob)
        {
            var uid = _staff.Keys.Single(candidate => PersonIndex(candidate) == action.PersonIndex);
            if (action.Kind == StaffActionKind.Sack)
            {
                WriteUInt64(_staff[uid].Bytes, Layout.FullContractPtrOffset, 0);
                return;
            }

            var job = hiredJob ?? action.JobType;
            SetContract(uid, _teamsByIndex[action.TeamIndex], job, NewWage(job), ExpiryJune2031);
        }

        private void SetContract(uint uid, ulong teamAddress, int job, long wage, uint expiry)
        {
            var address = FirstContract + (ulong)_contracts++ * 0x100;
            var contract = new byte[0x80];
            WriteUInt64(contract, Layout.ContractTeamPtrOffset, teamAddress);
            WriteUInt32(contract, Layout.ContractWeeklyWageOffset, (uint)wage);
            contract[Layout.ContractJobIdOffset] = (byte)job;
            WriteUInt32(contract, Layout.ContractExpiryOffset, expiry);
            Reader.AddBytes(address, contract);
            WriteUInt64(_staff[uid].Bytes, Layout.FullContractPtrOffset, address);
        }

        private byte[] Club(uint uid, int index, int teams)
        {
            var club = new byte[0x100];
            WriteInt32(club, Layout.ObjectDatabaseIndexOffset, index);
            WriteUInt32(club, Layout.ObjectUidOffset, uid);
            WriteUInt64(club, Layout.ClubTeamsBeginOffset, TeamVectorAddress);
            WriteUInt64(club, Layout.ClubTeamsEndOffset, TeamVectorAddress + (ulong)(teams * 8));
            return club;
        }

        private void AddTeam(ulong address, int type, int index, ulong club, ulong manager)
        {
            var team = new byte[0x100];
            team[Layout.TeamTypeOffset] = (byte)type;
            WriteInt32(team, Layout.ObjectDatabaseIndexOffset, index);
            WriteUInt64(team, Layout.TeamClubPtrOffset, club);
            WriteUInt64(team, Layout.TeamManagerPtrOffset, manager);
            Reader.AddBytes(address, team);
            _teamsByIndex[index] = address;
        }
    }

    private sealed class FakeDispatcher : IStaffChangeDispatcher
    {
        private readonly FakeFm _fm;

        public FakeDispatcher(FakeFm fm) => _fm = fm;

        public bool StaffActionsAvailable { get; set; } = true;

        public bool AppliesHires { get; set; } = true;

        /// <summary>A job FM gives instead of the requested one.</summary>
        public int? HiredJob { get; set; }

        public List<StaffAction> Sent { get; } = new();

        public bool TryDispatch(IReadOnlyList<StaffAction> actions, out string? error)
        {
            Sent.AddRange(actions);
            foreach (var action in actions.Where(action => action.Kind == StaffActionKind.Sack || AppliesHires))
            {
                _fm.Apply(action, HiredJob);
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
