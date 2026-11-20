using System.Buffers.Binary;
using FmDataBridge.Layouts;
using FmDataBridge.Mutations;
using FmDataBridge.Protocol;
using FmDataBridge.Scanning;
using FmDataBridge.StaffLimits;
using FmDataBridge.Tests.Fakes;
using Xunit;

namespace FmDataBridge.Tests;

public sealed class StaffLimitsOperationServiceTests
{
    private const string GameVersion = "27.1";
    private const string SourceRequestId = "scan-1";
    private const uint ClubUid = 680;
    private const int FirstTeamIndex = 565;
    private const int ReserveTeamIndex = 50754;
    private const int YouthTeamIndex = 50755;
    private const int CandidatesTeamIndex = 50756;
    private const uint Coaching = 1400137539; // StgC
    private const uint Recruitment = 1400137554; // StgR
    private const uint Medical = 1400137549; // StgM
    private static readonly IFmMemoryLayout Layout = Fm27Layout.Instance;

    // Record keys of a staff group, as FM sends it.
    private const uint NameKey = 1684370275; // "desc"
    private const uint CountKey = 1668771444; // "cwnt"
    private const uint MaxKey = 1298233430; // "MaxV"
    private const uint JobsKey = 1400138340; // "Stkd"
    private const uint JobNameKey = 825696289; // "17 !"
    private const uint JobTypeKey = 1363900275; // "QKws"

    [Fact]
    public void Reads_the_first_reserve_and_youth_teams_groups_and_never_the_youth_candidates()
    {
        var fm = new FakeFm();
        fm.Reader.Values[(FirstTeamIndex, Coaching)] = Group("Coaching Team", 28, 28, Job("Manager", 16, 1, 1), Job("Coaches", 2, 13, 13));
        fm.Reader.Values[(FirstTeamIndex, Recruitment)] = Group("Recruitment Team", 34, 32, Job("Scouts", 14, 18, 16));
        fm.Reader.Values[(ReserveTeamIndex, Coaching)] = Group("Coaching Team", 6, 7, Job("Coaches", 2, 4, 5));

        var result = fm.Service().Execute(fm.Request(), GameVersion, fm.Memory);

        Assert.True(result.Succeeded);
        Assert.Equal(new[] { FirstTeamIndex, ReserveTeamIndex, YouthTeamIndex }, fm.Reader.TeamIndexes);
        Assert.Equal(new[] { Coaching, Recruitment, Medical }, fm.Reader.Properties);
        Assert.Equal(
            new[] { ("senior", 0), ("reserve", 10), ("youth", 12) },
            result.Result!.Teams.Select(team => (team.Level, team.TeamType)));

        var senior = result.Result.Teams[0].Groups;
        Assert.Equal(
            new[] { (StaffLimitsGroup.Coaching, "Coaching Team", (int?)28, 28), (StaffLimitsGroup.Recruitment, "Recruitment Team", 34, 32) },
            senior.Select(group => (group.Group, group.Name, group.Count, group.Max)));
        Assert.Equal(
            new[] { (16, "Manager", (int?)1, 1), (2, "Coaches", 13, 13) },
            senior[0].Jobs.Select(job => (job.JobType, job.Name, job.Count, job.Max)));
        Assert.Equal((2, 5), (result.Result.Teams[1].Groups.Single().Jobs.Single().JobType, result.Result.Teams[1].Groups.Single().Jobs.Single().Max));
        Assert.Empty(result.Result.Teams[2].Groups);
    }

    [Fact]
    public void Skips_job_rows_without_a_job_type_or_limit_and_values_that_are_not_groups()
    {
        var fm = new FakeFm();
        fm.Reader.Values[(FirstTeamIndex, Coaching)] = Group(
            "Coaching Team",
            28,
            28,
            Job("Coaches", 2, 13, 13),
            new FmRecord(new Dictionary<uint, FmValue> { [JobNameKey] = new FmText("No type"), [MaxKey] = new FmNumber(2) }),
            Job("Negative", 60, 1, -1),
            new FmText("not a row"));
        fm.Reader.Values[(FirstTeamIndex, Medical)] = new FmList(Array.Empty<FmValue>());

        var result = fm.Service().Execute(fm.Request(), GameVersion, fm.Memory);

        var group = Assert.Single(result.Result!.Teams[0].Groups);
        Assert.Equal(2, Assert.Single(group.Jobs).JobType);
    }

    [Fact]
    public void Fails_when_fm_sends_no_group_for_the_first_team()
    {
        var fm = new FakeFm();
        fm.Reader.Values[(ReserveTeamIndex, Coaching)] = Group("Coaching Team", 6, 7, Job("Coaches", 2, 4, 5));

        var result = fm.Service().Execute(fm.Request(), GameVersion, fm.Memory);

        Assert.False(result.Succeeded);
        Assert.Equal(StaffLimitsFailure.NoLimits, result.Failure);
    }

    [Theory]
    [InlineData("other-scan", nameof(StaffLimitsFailure.SourceRequestMismatch))]
    [InlineData("other-club", nameof(StaffLimitsFailure.ClubNotFound))]
    [InlineData("no-reader", nameof(StaffLimitsFailure.ReadUnavailable))]
    [InlineData("no-layout", nameof(StaffLimitsFailure.UnsupportedGameBuild))]
    [InlineData("read-fails", nameof(StaffLimitsFailure.ReadFailed))]
    public void Refuses_or_fails_without_limits_when_it_cannot_read_for_the_club(string defect, string expected)
    {
        var fm = new FakeFm();
        fm.Reader.StaffLimitsAvailable = defect != "no-reader";
        fm.Reader.Error = defect == "read-fails" ? "FM did not run the staff limit read" : null;
        var request = new BridgeRequest
        {
            ProtocolVersion = BridgeProtocol.ProtocolVersion,
            RequestId = "limits-1",
            Operation = BridgeProtocol.OperationReadStaffLimits,
            SourceRequestId = defect == "other-scan" ? "scan-2" : SourceRequestId,
            ClubUid = defect == "other-club" ? ClubUid + 7 : ClubUid,
        };

        var result = fm.Service().Execute(request, defect == "no-layout" ? "28.0" : GameVersion, fm.Memory);

        Assert.False(result.Succeeded);
        Assert.Equal(expected, result.Failure.ToString());
        Assert.Null(result.Result);
        Assert.Equal(defect == "read-fails" ? "FM did not run the staff limit read" : null, result.Detail);
        if (defect != "read-fails")
        {
            Assert.Empty(fm.Reader.TeamIndexes);
        }
    }

    private static FmRecord Group(string name, int count, int max, params FmValue[] jobs) =>
        new(new Dictionary<uint, FmValue>
        {
            [NameKey] = new FmText(name),
            [CountKey] = new FmNumber(count),
            [MaxKey] = new FmNumber(max),
            [JobsKey] = new FmList(jobs),
        });

    private static FmRecord Job(string name, int jobType, int count, int max) =>
        new(new Dictionary<uint, FmValue>
        {
            [JobNameKey] = new FmText(name),
            [JobTypeKey] = new FmNumber(jobType),
            [CountKey] = new FmNumber(count),
            [MaxKey] = new FmNumber(max),
        });

    /// <summary>Fake FM memory: the managed club with a first, reserve, youth and Youth Candidates team.</summary>
    private sealed class FakeFm
    {
        private const ulong ClubAddress = 0x1000_0000;
        private const ulong TeamVectorAddress = 0x1100_0000;
        private const ulong FirstTeamAddress = 0x1200_0000;

        public FakeFm()
        {
            var teams = new[]
            {
                (Type: 22, Index: CandidatesTeamIndex),
                (Type: 0, Index: FirstTeamIndex),
                (Type: 12, Index: YouthTeamIndex),
                (Type: 10, Index: ReserveTeamIndex),
            };
            var club = new byte[0x100];
            WriteUInt32(club, Layout.ObjectUidOffset, ClubUid);
            WriteUInt64(club, Layout.ClubTeamsBeginOffset, TeamVectorAddress);
            WriteUInt64(club, Layout.ClubTeamsEndOffset, TeamVectorAddress + (ulong)(teams.Length * 8));
            Memory.AddBytes(ClubAddress, club);
            var vector = new byte[teams.Length * 8];
            for (var i = 0; i < teams.Length; i++)
            {
                var address = FirstTeamAddress + (ulong)i * 0x1_0000;
                WriteUInt64(vector, i * 8, address);
                var team = new byte[0x100];
                team[Layout.TeamTypeOffset] = (byte)teams[i].Type;
                WriteUInt32(team, Layout.ObjectDatabaseIndexOffset, (uint)teams[i].Index);
                Memory.AddBytes(address, team);
            }

            Memory.AddBytes(TeamVectorAddress, vector);
        }

        public FakeMemoryReader Memory { get; } = new();

        public FakeReader Reader { get; } = new();

        public StaffLimitsOperationService Service()
        {
            var players = new PlayerMutationIndex();
            players.Replace(
                SourceRequestId,
                GameVersion,
                new[] { new PersonCandidate(0x9000_0000, 0x9000_0000, 7, 100, 120, 0x288, PersonFacet.Player) },
                new Dictionary<uint, ulong> { [ClubUid] = ClubAddress });
            return new StaffLimitsOperationService(new LayoutRegistry(new[] { Layout }), players, Reader);
        }

        public BridgeRequest Request() =>
            new()
            {
                ProtocolVersion = BridgeProtocol.ProtocolVersion,
                RequestId = "limits-1",
                Operation = BridgeProtocol.OperationReadStaffLimits,
                SourceRequestId = SourceRequestId,
                ClubUid = ClubUid,
            };
    }

    private sealed class FakeReader : IStaffLimitsReader
    {
        public bool StaffLimitsAvailable { get; set; } = true;

        public string? Error { get; set; }

        public Dictionary<(int TeamIndex, uint Property), FmValue> Values { get; } = new();

        public List<int> TeamIndexes { get; } = new();

        public List<uint> Properties { get; } = new();

        public bool TryReadStaffGroups(
            IReadOnlyList<int> teamIndexes,
            IReadOnlyList<uint> properties,
            out IReadOnlyDictionary<(int TeamIndex, uint Property), FmValue> values,
            out string? error)
        {
            TeamIndexes.AddRange(teamIndexes);
            Properties.AddRange(properties);
            values = Values;
            error = Error;
            return Error is null;
        }
    }

    private static void WriteUInt32(byte[] bytes, int offset, uint value) =>
        BinaryPrimitives.WriteUInt32LittleEndian(bytes.AsSpan(offset), value);

    private static void WriteUInt64(byte[] bytes, int offset, ulong value) =>
        BinaryPrimitives.WriteUInt64LittleEndian(bytes.AsSpan(offset), value);
}
