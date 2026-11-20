using System.Text;
using System.Text.Json;
using FmDataBridge.Layouts;
using FmDataBridge.Scanning;
using FmDataBridge.Tests.Fakes;
using Xunit;

namespace FmDataBridge.Tests;

public sealed class TeamLayoutDiagnosticTests
{
    private const ulong Club = 0x100000;
    private const ulong ClubName = 0x101000;
    private const ulong TeamList = 0x102000;
    private const ulong FirstTeam = 0x110000;
    private const ulong YouthTeam = 0x120000;
    private const ulong Squad = 0x130000;
    private const ulong SquadHolder = 0x140000;
    private const ulong YouthSquad = 0x141000;
    private const ulong Wrapper = 0x150000;
    private const ulong PlayerBlock = 0x200000;
    private const ulong PlayerObject = PlayerBlock + 0x288;
    private const ulong PlayerContract = 0x210000;
    private const ulong StaffBlock = 0x300000;
    private const ulong StaffObject = StaffBlock + 0x100;
    private const ulong StaffContract = 0x310000;

    [Fact]
    public void Finds_squad_vectors_and_contract_teams_of_a_requested_club()
    {
        var layout = Fm27Layout.Instance;
        var reader = new FakeMemoryReader();
        reader.AddBytes(Club + (ulong)layout.ObjectUidOffset, BitConverter.GetBytes(680u));
        reader.AddBytes(Club + (ulong)layout.ClubNameOffset, BitConverter.GetBytes(ClubName));
        reader.AddBytes(ClubName + 4, Encoding.UTF8.GetBytes("Test FC\0"));
        reader.AddBytes(Club + (ulong)layout.ClubTeamsBeginOffset, BitConverter.GetBytes(TeamList));
        reader.AddBytes(Club + (ulong)layout.ClubTeamsEndOffset, BitConverter.GetBytes(TeamList + 16));
        reader.AddBytes(TeamList, BitConverter.GetBytes(FirstTeam));
        reader.AddBytes(TeamList + 8, BitConverter.GetBytes(YouthTeam));

        // First team: a direct vector of player objects at 0x38.
        reader.AddBytes(FirstTeam + 0x38, BitConverter.GetBytes(Squad));
        reader.AddBytes(FirstTeam + 0x40, BitConverter.GetBytes(Squad + 8));
        reader.AddBytes(Squad, BitConverter.GetBytes(PlayerObject));

        // Youth team: a holder object whose vector holds wrappers around the player block.
        reader.AddBytes(YouthTeam + 0x58, BitConverter.GetBytes(SquadHolder));
        reader.AddBytes(SquadHolder + 0x10, BitConverter.GetBytes(YouthSquad));
        reader.AddBytes(SquadHolder + 0x18, BitConverter.GetBytes(YouthSquad + 8));
        reader.AddBytes(YouthSquad, BitConverter.GetBytes(Wrapper));
        reader.AddBytes(Wrapper + 0x8, BitConverter.GetBytes(PlayerBlock));

        reader.AddBytes(PlayerObject + (ulong)layout.FullContractPtrOffset, BitConverter.GetBytes(PlayerContract));
        reader.AddBytes(PlayerContract + (ulong)layout.ContractTeamPtrOffset, BitConverter.GetBytes(FirstTeam));
        reader.AddBytes(StaffObject + (ulong)layout.FullContractPtrOffset, BitConverter.GetBytes(StaffContract));
        reader.AddBytes(StaffContract + (ulong)layout.ContractTeamPtrOffset, BitConverter.GetBytes(YouthTeam));

        var directory = Path.Combine(Path.GetTempPath(), "fm-valuescout-tests", Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(directory);
        File.WriteAllText(Path.Combine(directory, TeamLayoutDiagnostic.RequestFileName), "680\n");
        var persons = new[]
        {
            new TeamLayoutDiagnostic.Person(11, true, "Player", 2008, 100, 150, null, PlayerBlock, PlayerObject, FirstTeam, Club),
            new TeamLayoutDiagnostic.Person(22, false, "Coach", 1980, 90, 120, 2, StaffBlock, StaffObject, YouthTeam, Club),
        };

        TeamLayoutDiagnostic.TryWrite(reader, layout, directory, "27.1", new[] { Club }, persons, default);

        Assert.False(File.Exists(Path.Combine(directory, "team-diagnostic-error.txt")));
        var lines = File.ReadAllLines(Path.Combine(directory, TeamLayoutDiagnostic.OutputFileName))
            .Select(line => JsonDocument.Parse(line).RootElement)
            .ToList();
        var meta = lines.Single(line => line.GetProperty("type").GetString() == "meta");
        Assert.Equal(new[] { 680u }, meta.GetProperty("requestedClubUids").EnumerateArray().Select(uid => uid.GetUInt32()));

        var teams = lines.Where(line => line.GetProperty("type").GetString() == "team").ToList();
        var first = teams.Single(team => team.GetProperty("index").GetInt32() == 0);
        var squad = first.GetProperty("vectors").EnumerateArray().Single(vector => vector.GetProperty("off").GetString() == "0x38");
        Assert.Equal(1, squad.GetProperty("kinds").GetProperty("player").GetInt32());
        Assert.Equal(1, first.GetProperty("contractedPlayers").GetInt32());

        var youth = teams.Single(team => team.GetProperty("index").GetInt32() == 1);
        Assert.Contains(
            youth.GetProperty("wrapped").EnumerateArray(),
            hit => hit.GetProperty("vector").GetString() == "0x58>0x10"
                && hit.GetProperty("hits").GetProperty("0x8:player").GetInt32() == 1);
        Assert.Equal(1, youth.GetProperty("contractedStaff").GetInt32());

        var people = lines.Where(line => line.GetProperty("type").GetString() == "person").ToList();
        var coach = people.Single(person => person.GetProperty("uid").GetUInt32() == 22);
        Assert.Equal("team:680:1", coach.GetProperty("contractTeam").GetString());
        var player = people.Single(person => person.GetProperty("uid").GetUInt32() == 11);
        Assert.Contains(
            player.GetProperty("inVectors").EnumerateArray(),
            entry => entry[0].GetString() == "team:680:1" && entry[2].GetString() == "wrapper0x8");

        var staffJobs = lines.Single(line => line.GetProperty("type").GetString() == "histogram"
            && line.GetProperty("name").GetString() == "staffJobIdByContractTeamIndex");
        Assert.Equal(1, staffJobs.GetProperty("counts").GetProperty("2|1").GetInt32());
        Assert.Equal("done", lines.Last().GetProperty("type").GetString());
    }
}
