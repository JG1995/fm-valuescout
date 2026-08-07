using System.Buffers.Binary;
using System.Text.Json;
using FmDataBridge.MemoryProbe;
using FmDataBridge.Models;
using FmDataBridge.Protocol;
using Xunit;

namespace FmDataBridge.Tests;

public sealed class MemoryProbeCliTests
{
    [Fact]
    public void Correlation_keeps_a_single_player_duplicate_ambiguous_but_ranks_the_shared_path()
    {
        var directory = CreateTempDirectory();
        try
        {
            var singleCapturePath = Path.Combine(directory, "single-probe.json");
            var singleCsvPath = Path.Combine(directory, "single.csv");
            File.WriteAllText(
                singleCapturePath,
                Serialize(
                    CreateCapture(
                        "single-capture",
                        Player(1001, 120, 160, 15, 10_000_000, 120),
                        schemaVersion: 1)));
            File.WriteAllText(singleCsvPath, "UID,CA\n1001,120\n");

            var single = Run(
                "correlate",
                "--csv",
                singleCsvPath,
                "--capture",
                singleCapturePath,
                "--uid-column",
                "UID",
                "--field",
                "ca=CA");

            Assert.Equal(0, single.ExitCode);
            using var singleDocument = JsonDocument.Parse(single.Output);
            var singleField = GetField(singleDocument.RootElement, "ca");
            Assert.Equal("ambiguous", singleField.GetProperty("outcome").GetString());
            Assert.Equal(2, singleField.GetProperty("topCandidateCount").GetInt32());

            var multiCapturePath = Path.Combine(directory, "multi-probe.json");
            var multiCsvPath = Path.Combine(directory, "multi.csv");
            File.WriteAllText(
                multiCapturePath,
                Serialize(
                    CreateCapture(
                        "multi-capture",
                        Player(1001, 120, 160, 15, 10_000_000, 120),
                        Player(1002, 145, 170, 11, 3_000_000, 119))));
            File.WriteAllText(multiCsvPath, "UID,CA\n1001,120\n1002,145\n");

            var multi = Run(
                "correlate",
                "--csv",
                multiCsvPath,
                "--capture",
                multiCapturePath,
                "--uid-column",
                "UID",
                "--field",
                "ca=CA");

            Assert.Equal(0, multi.ExitCode);
            using var multiDocument = JsonDocument.Parse(multi.Output);
            var multiField = GetField(multiDocument.RootElement, "ca");
            Assert.Equal("candidate", multiField.GetProperty("outcome").GetString());
            Assert.Equal("player-block+0x264", multiField.GetProperty("candidates")[0].GetProperty("path").GetString());
            Assert.Equal("uint16-le", multiField.GetProperty("candidates")[0].GetProperty("encoding").GetString());
        }
        finally
        {
            Directory.Delete(directory, recursive: true);
        }
    }

    [Fact]
    public void Correlation_requires_varied_multi_player_evidence_before_reporting_a_candidate()
    {
        var directory = CreateTempDirectory();
        try
        {
            var singleCapturePath = Path.Combine(directory, "single-probe.json");
            var singleCsvPath = Path.Combine(directory, "single.csv");
            File.WriteAllText(singleCapturePath, Serialize(CreateCapture("single-capture", Player(1001, 120, 160, 15, 10_000_000, 119))));
            File.WriteAllText(singleCsvPath, "UID,CA\n1001,120\n");

            var single = Run(
                "correlate",
                "--csv",
                singleCsvPath,
                "--capture",
                singleCapturePath,
                "--uid-column",
                "UID",
                "--field",
                "ca=CA");

            Assert.Equal(0, single.ExitCode);
            using var singleDocument = JsonDocument.Parse(single.Output);
            var singleField = GetField(singleDocument.RootElement, "ca");
            Assert.Equal("ambiguous", singleField.GetProperty("outcome").GetString());
            Assert.False(singleField.GetProperty("evidenceSufficient").GetBoolean());

            var repeatedCapturePath = Path.Combine(directory, "repeated-probe.json");
            var repeatedCsvPath = Path.Combine(directory, "repeated.csv");
            File.WriteAllText(
                repeatedCapturePath,
                Serialize(
                    CreateCapture(
                        "repeated-capture",
                        Player(1001, 120, 160, 15, 10_000_000, 119),
                        Player(1002, 120, 170, 11, 3_000_000, 118))));
            File.WriteAllText(repeatedCsvPath, "UID,CA\n1001,120\n1002,120\n");

            var repeated = Run(
                "correlate",
                "--csv",
                repeatedCsvPath,
                "--capture",
                repeatedCapturePath,
                "--uid-column",
                "UID",
                "--field",
                "ca=CA");

            Assert.Equal(0, repeated.ExitCode);
            using var repeatedDocument = JsonDocument.Parse(repeated.Output);
            var repeatedField = GetField(repeatedDocument.RootElement, "ca");
            Assert.Equal("ambiguous", repeatedField.GetProperty("outcome").GetString());
            Assert.False(repeatedField.GetProperty("evidenceSufficient").GetBoolean());
        }
        finally
        {
            Directory.Delete(directory, recursive: true);
        }
    }

    [Fact]
    public void Correlation_detects_quoted_delimiters_and_known_scalar_encodings()
    {
        var directory = CreateTempDirectory();
        try
        {
            var capturePath = Path.Combine(directory, "probe.json");
            var csvPath = Path.Combine(directory, "players.csv");
            File.WriteAllText(
                capturePath,
                Serialize(
                    CreateCapture(
                        "encoding-capture",
                        Player(1001, 120, 160, 15, 10_000_000, 1, signedValue: -12, rawByteValue: 201, signedByteValue: -70, signedInt32Value: -100_000),
                        Player(1002, 145, 170, 11, 3_000_000, 2, signedValue: -40, rawByteValue: 202, signedByteValue: -80, signedInt32Value: -200_000))));
            File.WriteAllText(
                csvPath,
                "UID;Name;CA;PA;Determination;Market Value;Signed;Raw Byte;Signed Byte;Signed Int32\n"
                + "1001;\"Alpha; One\";120;160;15;10,000,000;-12;201;-70;-100000\n"
                + "1002;\"Beta; Two\";145;170;11;3,000,000;-40;202;-80;-200000\n");

            var result = Run(
                "correlate",
                "--csv",
                csvPath,
                "--capture",
                capturePath,
                "--uid-column",
                "UID",
                "--field",
                "uid=UID",
                "--field",
                "ca=CA",
                "--field",
                "pa=PA",
                "--field",
                "determination=Determination",
                "--field",
                "market=Market Value",
                "--field",
                "signed=Signed",
                "--field",
                "raw-byte=Raw Byte",
                "--field",
                "signed-byte=Signed Byte",
                "--field",
                "signed-int32=Signed Int32");

            Assert.Equal(0, result.ExitCode);
            using var document = JsonDocument.Parse(result.Output);
            Assert.Equal("semicolon", document.RootElement.GetProperty("delimiter").GetString());
            AssertCandidate(document.RootElement, "uid", "person-object+0xC", "uint32-le");
            AssertCandidate(document.RootElement, "ca", "player-block+0x264", "uint16-le");
            AssertCandidate(document.RootElement, "pa", "player-block+0x266", "uint16-le");
            AssertCandidate(document.RootElement, "determination", "player-block+0x192", "uint8-times-five");
            AssertCandidate(document.RootElement, "market", "player-block+0x234", "uint32-le");
            AssertCandidate(document.RootElement, "signed", "player-block+0x30", "int16-le");
            AssertCandidate(document.RootElement, "raw-byte", "player-block+0x38", "uint8");
            AssertCandidate(document.RootElement, "signed-byte", "player-block+0x39", "int8");
            AssertCandidate(document.RootElement, "signed-int32", "player-block+0x3C", "int32-le");
            Assert.True(GetField(document.RootElement, "ca").GetProperty("candidates")[0].GetProperty("encodingAmbiguous").GetBoolean());
        }
        finally
        {
            Directory.Delete(directory, recursive: true);
        }
    }

    [Theory]
    [InlineData(',', "comma")]
    [InlineData('\t', "tab")]
    public void Correlation_detects_supported_delimiters(char delimiter, string expectedDelimiter)
    {
        var directory = CreateTempDirectory();
        try
        {
            var capturePath = Path.Combine(directory, "probe.json");
            var csvPath = Path.Combine(directory, "players.csv");
            File.WriteAllText(capturePath, Serialize(CreateCapture("delimiter-capture", Player(1001, 120, 160, 15, 10_000_000, 1))));
            File.WriteAllText(
                csvPath,
                $"UID{delimiter}Name{delimiter}CA\n1001{delimiter}\"Alpha{delimiter} One\"{delimiter}120\n");

            var result = Run(
                "correlate",
                "--csv",
                csvPath,
                "--capture",
                capturePath,
                "--uid-column",
                "UID",
                "--field",
                "ca=CA");

            Assert.Equal(0, result.ExitCode);
            using var document = JsonDocument.Parse(result.Output);
            Assert.Equal(expectedDelimiter, document.RootElement.GetProperty("delimiter").GetString());
        }
        finally
        {
            Directory.Delete(directory, recursive: true);
        }
    }

    [Fact]
    public void Correlation_parses_period_grouped_numeric_values()
    {
        var directory = CreateTempDirectory();
        try
        {
            var capturePath = Path.Combine(directory, "probe.json");
            var csvPath = Path.Combine(directory, "players.csv");
            File.WriteAllText(
                capturePath,
                Serialize(
                    CreateCapture(
                        "period-grouped-capture",
                        Player(1001, 120, 160, 15, 1_234, 1),
                        Player(1002, 145, 170, 11, 5_678, 2))));
            File.WriteAllText(csvPath, "UID;Market Value\n1001;1.234\n1002;5.678\n");

            var result = Run(
                "correlate",
                "--csv",
                csvPath,
                "--capture",
                capturePath,
                "--uid-column",
                "UID",
                "--field",
                "market=Market Value");

            Assert.Equal(0, result.ExitCode);
            using var document = JsonDocument.Parse(result.Output);
            AssertCandidate(document.RootElement, "market", "player-block+0x234", "uint32-le");
        }
        finally
        {
            Directory.Delete(directory, recursive: true);
        }
    }

    [Fact]
    public void Correlation_normalizes_real_export_numeric_shapes_and_reports_field_eligibility()
    {
        var directory = CreateTempDirectory();
        try
        {
            var capturePath = Path.Combine(directory, "probe.json");
            var csvPath = Path.Combine(directory, "players.csv");
            File.WriteAllText(
                capturePath,
                Serialize(
                    CreateCapture(
                        "normalized-capture",
                        Player(1001, 120, 160, 15, 10_000_000, 1, starts: 32, substitutes: 5, rating: 7.25f, distanceTenths: 2051, sparseValue: 0),
                        Player(1002, 145, 170, 11, 3_000_000, 2, starts: 12, substitutes: 1, rating: 6.80f, distanceTenths: 1558, sparseValue: 0),
                        Player(1003, 130, 165, 13, 5_000_000, 3, starts: 4, substitutes: 0, rating: 0f, distanceTenths: 0, sparseValue: 0))));
            File.WriteAllText(
                csvPath,
                "UID;Appearances;Rating;Distance;Sparse\n"
                + "1001;32 (5);7.25;205.1km;0\n"
                + "1002;12 (1);6.80;155.8km;-\n"
                + "1003;4 (0);-;;-\n");

            var result = Run(
                "correlate",
                "--csv",
                csvPath,
                "--capture",
                capturePath,
                "--uid-column",
                "UID",
                "--field",
                "starts=Appearances",
                "--field",
                "substitutes=Appearances",
                "--field",
                "rating=Rating",
                "--field",
                "distance=Distance",
                "--field",
                "sparse=Sparse",
                "--transform",
                "starts=appearances-starts",
                "--transform",
                "substitutes=appearances-subs",
                "--transform",
                "rating=decimal:2",
                "--transform",
                "distance=unit-decimal:km:1");

            Assert.Equal(0, result.ExitCode);
            using var document = JsonDocument.Parse(result.Output);
            AssertNormalizedCandidate(
                document.RootElement,
                "starts",
                "appearances-starts",
                "player-block+0x60",
                "uint16-le",
                "exact",
                new[] { 1001u, 1002u, 1003u },
                Array.Empty<uint>());
            AssertNormalizedCandidate(
                document.RootElement,
                "substitutes",
                "appearances-subs",
                "player-block+0x62",
                "uint16-le",
                "exact",
                new[] { 1001u, 1002u, 1003u },
                Array.Empty<uint>());
            AssertNormalizedCandidate(
                document.RootElement,
                "rating",
                "decimal:2",
                "player-block+0x64",
                "float32-le-rounded-2",
                "rounded",
                new[] { 1001u, 1002u },
                new[] { 1003u });
            AssertNormalizedCandidate(
                document.RootElement,
                "distance",
                "unit-decimal:km:1",
                "player-block+0x68",
                "uint32-le-fixed-scale-10",
                "fixed-scale",
                new[] { 1001u, 1002u },
                new[] { 1003u });

            var sparse = GetField(document.RootElement, "sparse");
            Assert.NotEqual("candidate", sparse.GetProperty("outcome").GetString());
            Assert.False(sparse.GetProperty("evidenceSufficient").GetBoolean());
            Assert.Equal(new[] { 1001u }, sparse.GetProperty("eligibleUids").EnumerateArray().Select(value => value.GetUInt32()).ToArray());
            Assert.Equal(new[] { 1002u, 1003u }, sparse.GetProperty("excludedUids").EnumerateArray().Select(value => value.GetUInt32()).ToArray());
        }
        finally
        {
            Directory.Delete(directory, recursive: true);
        }
    }

    [Fact]
    public void Correlation_rejects_unsupported_display_text_instead_of_guessing_a_scalar()
    {
        var directory = CreateTempDirectory();
        try
        {
            var capturePath = Path.Combine(directory, "probe.json");
            var csvPath = Path.Combine(directory, "players.csv");
            File.WriteAllText(
                capturePath,
                Serialize(
                    CreateCapture(
                        "display-text-capture",
                        Player(1001, 120, 160, 15, 10_000_000, 1),
                        Player(1002, 145, 170, 11, 3_000_000, 2))));
            File.WriteAllText(csvPath, "UID;Wage\n1001;£1K - £2K\n1002;£3K - £4K\n");

            var result = Run(
                "correlate",
                "--csv",
                csvPath,
                "--capture",
                capturePath,
                "--uid-column",
                "UID",
                "--field",
                "wage=Wage");

            Assert.NotEqual(0, result.ExitCode);
            Assert.Contains("field 'Wage' has unsupported numeric value", result.Error, StringComparison.Ordinal);
        }
        finally
        {
            Directory.Delete(directory, recursive: true);
        }
    }

    [Fact]
    public async Task Capture_writes_a_uid_scoped_request_and_awaits_a_new_ready_capture()
    {
        var directory = CreateTempDirectory();
        try
        {
            var csvPath = Path.Combine(directory, "players.csv");
            File.WriteAllText(csvPath, "UID,Name\n1001,Alpha\n1002,Beta\n");
            File.WriteAllText(
                BridgePaths.GetProbePath(directory),
                Serialize(CreateCapture("old-capture", Player(1001, 120, 160, 15, 10_000_000, 1))));
            File.WriteAllText(
                BridgePaths.GetProbeStatusPath(directory),
                Serialize(
                    new ProbeStatus
                    {
                        ProtocolVersion = ProbeProtocol.ProtocolVersion,
                        PluginVersion = "0.1.0",
                        State = ProbeProtocol.StateReady,
                        UpdatedAtUtc = DateTimeOffset.UtcNow,
                        GamePluginModulePresent = true,
                        GameAssemblyModulePresent = true,
                        RequestId = "old-capture",
                        PlayersCaptured = 1,
                    }));

            var writeFreshCapture = Task.Run(
                () =>
                {
                    Thread.Sleep(150);
                    File.WriteAllText(
                        BridgePaths.GetProbePath(directory),
                        Serialize(
                            CreateCapture(
                                "capture-ready",
                                Player(1001, 120, 160, 15, 10_000_000, 1),
                                Player(1002, 145, 170, 11, 3_000_000, 2),
                                generatedAtUtc: DateTimeOffset.UtcNow.ToString("O"))));
                    File.WriteAllText(
                        BridgePaths.GetProbeStatusPath(directory),
                        Serialize(
                            new ProbeStatus
                            {
                                ProtocolVersion = ProbeProtocol.ProtocolVersion,
                                PluginVersion = "0.1.0",
                                State = ProbeProtocol.StateReady,
                                UpdatedAtUtc = DateTimeOffset.UtcNow,
                                GamePluginModulePresent = true,
                                GameAssemblyModulePresent = true,
                                RequestId = "capture-ready",
                                PlayersCaptured = 2,
                            }));
                });

            var result = Run(
                "capture",
                "--csv",
                csvPath,
                "--uid-column",
                "UID",
                "--bridge-dir",
                directory,
                "--request-id",
                "capture-ready",
                "--timeout-seconds",
                "1");

            await writeFreshCapture;

            Assert.Equal(0, result.ExitCode);
            using var request = JsonDocument.Parse(File.ReadAllText(BridgePaths.GetProbeRequestPath(directory)));
            Assert.Equal("capture-ready", request.RootElement.GetProperty("requestId").GetString());
            Assert.Equal(new[] { 1001u, 1002u }, request.RootElement.GetProperty("uids").EnumerateArray().Select(value => value.GetUInt32()).ToArray());
            using var output = JsonDocument.Parse(result.Output);
            Assert.Equal("capture-ready", output.RootElement.GetProperty("requestId").GetString());
            Assert.Equal(2, output.RootElement.GetProperty("playersCaptured").GetInt32());
        }
        finally
        {
            Directory.Delete(directory, recursive: true);
        }
    }

    [Fact]
    public void Capture_rejects_a_request_id_already_used_by_a_probe_capture()
    {
        var directory = CreateTempDirectory();
        try
        {
            var csvPath = Path.Combine(directory, "players.csv");
            File.WriteAllText(csvPath, "UID,Name\n1001,Alpha\n");
            File.WriteAllText(
                BridgePaths.GetProbePath(directory),
                Serialize(
                    CreateCapture(
                        "wanted-capture",
                        Player(1001, 120, 160, 15, 10_000_000, 1),
                        generatedAtUtc: DateTimeOffset.UtcNow.AddMinutes(-1).ToString("O"))));

            var result = Run(
                "capture",
                "--csv",
                csvPath,
                "--uid-column",
                "UID",
                "--bridge-dir",
                directory,
                "--request-id",
                "wanted-capture",
                "--timeout-seconds",
                "1");

            Assert.NotEqual(0, result.ExitCode);
            Assert.Contains("must not be reused", result.Error, StringComparison.OrdinalIgnoreCase);
            Assert.False(File.Exists(BridgePaths.GetProbeRequestPath(directory)));
        }
        finally
        {
            Directory.Delete(directory, recursive: true);
        }
    }

    [Fact]
    public void Capture_rejects_a_request_id_while_an_earlier_same_id_scan_is_in_progress()
    {
        var directory = CreateTempDirectory();
        try
        {
            var csvPath = Path.Combine(directory, "players.csv");
            File.WriteAllText(csvPath, "UID,Name\n1001,Alpha\n");
            File.WriteAllText(
                BridgePaths.GetProbeStatusPath(directory),
                Serialize(
                    new ProbeStatus
                    {
                        ProtocolVersion = ProbeProtocol.ProtocolVersion,
                        PluginVersion = "0.1.0",
                        State = ProbeProtocol.StateScanning,
                        UpdatedAtUtc = DateTimeOffset.UtcNow,
                        GamePluginModulePresent = true,
                        GameAssemblyModulePresent = true,
                        RequestId = "retry-capture",
                    }));

            var result = Run(
                "capture",
                "--csv",
                csvPath,
                "--uid-column",
                "UID",
                "--bridge-dir",
                directory,
                "--request-id",
                "retry-capture",
                "--timeout-seconds",
                "1");

            Assert.NotEqual(0, result.ExitCode);
            Assert.Contains("must not be reused", result.Error, StringComparison.OrdinalIgnoreCase);
            Assert.False(File.Exists(BridgePaths.GetProbeRequestPath(directory)));
        }
        finally
        {
            Directory.Delete(directory, recursive: true);
        }
    }

    [Fact]
    public void Diff_finds_a_controlled_times_five_scalar_delta_for_matching_capture_pairs()
    {
        var directory = CreateTempDirectory();
        try
        {
            var beforeCapturePath = Path.Combine(directory, "before-probe.json");
            var afterCapturePath = Path.Combine(directory, "after-probe.json");
            var beforeCsvPath = Path.Combine(directory, "before.csv");
            var afterCsvPath = Path.Combine(directory, "after.csv");
            File.WriteAllText(
                beforeCapturePath,
                Serialize(
                    CreateCapture(
                        "before-capture",
                        Player(1001, 120, 160, 15, 10_000_000, 1),
                        Player(1002, 145, 170, 11, 3_000_000, 2))));
            File.WriteAllText(
                afterCapturePath,
                Serialize(
                    CreateCapture(
                        "after-capture",
                        Player(1001, 120, 160, 16, 10_000_000, 1),
                        Player(1002, 145, 170, 11, 3_000_000, 2))));
            File.WriteAllText(beforeCsvPath, "UID,Determination\n1001,15\n1002,11\n");
            File.WriteAllText(afterCsvPath, "UID,Determination\n1001,16\n1002,11\n");

            var result = Run(
                "diff",
                "--before-csv",
                beforeCsvPath,
                "--after-csv",
                afterCsvPath,
                "--before-capture",
                beforeCapturePath,
                "--after-capture",
                afterCapturePath,
                "--uid-column",
                "UID",
                "--field",
                "determination=Determination");

            Assert.Equal(0, result.ExitCode);
            using var document = JsonDocument.Parse(result.Output);
            Assert.Equal("before-capture", document.RootElement.GetProperty("beforeRequestId").GetString());
            Assert.Equal("after-capture", document.RootElement.GetProperty("afterRequestId").GetString());
            Assert.Equal(1, document.RootElement.GetProperty("changedByteCount").GetInt32());
            var field = GetField(document.RootElement, "determination");
            Assert.Equal("candidate", field.GetProperty("outcome").GetString());
            Assert.Equal(new[] { 1001u }, field.GetProperty("changedUids").EnumerateArray().Select(value => value.GetUInt32()).ToArray());
            Assert.Equal("player-block+0x192", field.GetProperty("candidates")[0].GetProperty("path").GetString());
            Assert.Equal("uint8-times-five", field.GetProperty("candidates")[0].GetProperty("encoding").GetString());
        }
        finally
        {
            Directory.Delete(directory, recursive: true);
        }
    }

    [Fact]
    public void Diff_excludes_missing_decimal_values_and_reports_rounded_evidence()
    {
        var directory = CreateTempDirectory();
        try
        {
            var beforeCapturePath = Path.Combine(directory, "before-probe.json");
            var afterCapturePath = Path.Combine(directory, "after-probe.json");
            var beforeCsvPath = Path.Combine(directory, "before.csv");
            var afterCsvPath = Path.Combine(directory, "after.csv");
            File.WriteAllText(
                beforeCapturePath,
                Serialize(
                    CreateCapture(
                        "before-capture",
                        Player(1001, 120, 160, 15, 10_000_000, 1, rating: 7.25f),
                        Player(1002, 145, 170, 11, 3_000_000, 2, rating: 6.80f),
                        Player(1003, 130, 165, 13, 5_000_000, 3, rating: 0f))));
            File.WriteAllText(
                afterCapturePath,
                Serialize(
                    CreateCapture(
                        "after-capture",
                        Player(1001, 120, 160, 15, 10_000_000, 1, rating: 7.30f),
                        Player(1002, 145, 170, 11, 3_000_000, 2, rating: 6.90f),
                        Player(1003, 130, 165, 13, 5_000_000, 3, rating: 0f))));
            File.WriteAllText(beforeCsvPath, "UID;Rating\n1001;7.25\n1002;6.80\n1003;-\n");
            File.WriteAllText(afterCsvPath, "UID;Rating\n1001;7.30\n1002;6.90\n1003;-\n");

            var result = Run(
                "diff",
                "--before-csv",
                beforeCsvPath,
                "--after-csv",
                afterCsvPath,
                "--before-capture",
                beforeCapturePath,
                "--after-capture",
                afterCapturePath,
                "--uid-column",
                "UID",
                "--field",
                "rating=Rating",
                "--transform",
                "rating=decimal:2");

            Assert.Equal(0, result.ExitCode);
            using var document = JsonDocument.Parse(result.Output);
            var field = GetField(document.RootElement, "rating");
            Assert.Equal("candidate", field.GetProperty("outcome").GetString());
            Assert.Equal("decimal:2", field.GetProperty("normalization").GetString());
            Assert.Equal(new[] { 1001u, 1002u }, field.GetProperty("changedUids").EnumerateArray().Select(value => value.GetUInt32()).ToArray());
            Assert.Equal(new[] { 1001u, 1002u }, field.GetProperty("eligibleUids").EnumerateArray().Select(value => value.GetUInt32()).ToArray());
            Assert.Equal(new[] { 1003u }, field.GetProperty("excludedUids").EnumerateArray().Select(value => value.GetUInt32()).ToArray());
            Assert.Equal("float32-le-rounded-2", field.GetProperty("candidates")[0].GetProperty("encoding").GetString());
            Assert.Equal("rounded", field.GetProperty("candidates")[0].GetProperty("evidenceKind").GetString());
        }
        finally
        {
            Directory.Delete(directory, recursive: true);
        }
    }

    [Fact]
    public void Diff_requires_cross_player_variation_before_reporting_a_candidate()
    {
        var directory = CreateTempDirectory();
        try
        {
            var beforeCapturePath = Path.Combine(directory, "before-probe.json");
            var afterCapturePath = Path.Combine(directory, "after-probe.json");
            var beforeCsvPath = Path.Combine(directory, "before.csv");
            var afterCsvPath = Path.Combine(directory, "after.csv");
            File.WriteAllText(
                beforeCapturePath,
                Serialize(
                    CreateCapture(
                        "before-capture",
                        Player(1001, 120, 160, 0, 10_000_000, 1),
                        Player(1002, 145, 170, 0, 3_000_000, 2))));
            File.WriteAllText(
                afterCapturePath,
                Serialize(
                    CreateCapture(
                        "after-capture",
                        Player(1001, 120, 160, 1, 10_000_000, 1),
                        Player(1002, 145, 170, 1, 3_000_000, 2))));
            File.WriteAllText(beforeCsvPath, "UID,Determination\n1001,0\n1002,0\n");
            File.WriteAllText(afterCsvPath, "UID,Determination\n1001,1\n1002,1\n");

            var result = Run(
                "diff",
                "--before-csv",
                beforeCsvPath,
                "--after-csv",
                afterCsvPath,
                "--before-capture",
                beforeCapturePath,
                "--after-capture",
                afterCapturePath,
                "--uid-column",
                "UID",
                "--field",
                "determination=Determination");

            Assert.Equal(0, result.ExitCode);
            using var document = JsonDocument.Parse(result.Output);
            var field = GetField(document.RootElement, "determination");
            Assert.False(field.GetProperty("evidenceSufficient").GetBoolean());
            Assert.NotEqual("candidate", field.GetProperty("outcome").GetString());
        }
        finally
        {
            Directory.Delete(directory, recursive: true);
        }
    }

    [Fact]
    public void Diff_requires_varied_multi_player_evidence_before_reporting_a_candidate()
    {
        var directory = CreateTempDirectory();
        try
        {
            var beforeCapturePath = Path.Combine(directory, "before-probe.json");
            var afterCapturePath = Path.Combine(directory, "after-probe.json");
            var beforeCsvPath = Path.Combine(directory, "before.csv");
            var afterCsvPath = Path.Combine(directory, "after.csv");
            File.WriteAllText(beforeCapturePath, Serialize(CreateCapture("before-capture", Player(1001, 120, 160, 15, 10_000_000, 1))));
            File.WriteAllText(afterCapturePath, Serialize(CreateCapture("after-capture", Player(1001, 120, 160, 16, 10_000_000, 1))));
            File.WriteAllText(beforeCsvPath, "UID,Determination\n1001,15\n");
            File.WriteAllText(afterCsvPath, "UID,Determination\n1001,16\n");

            var result = Run(
                "diff",
                "--before-csv",
                beforeCsvPath,
                "--after-csv",
                afterCsvPath,
                "--before-capture",
                beforeCapturePath,
                "--after-capture",
                afterCapturePath,
                "--uid-column",
                "UID",
                "--field",
                "determination=Determination");

            Assert.Equal(0, result.ExitCode);
            using var document = JsonDocument.Parse(result.Output);
            var field = GetField(document.RootElement, "determination");
            Assert.Equal("ambiguous", field.GetProperty("outcome").GetString());
            Assert.False(field.GetProperty("evidenceSufficient").GetBoolean());
        }
        finally
        {
            Directory.Delete(directory, recursive: true);
        }
    }

    [Fact]
    public void Correlation_rejects_duplicate_uids_and_reports_unreadable_ranges_without_false_zeroes()
    {
        var directory = CreateTempDirectory();
        try
        {
            var capturePath = Path.Combine(directory, "probe.json");
            var duplicateCsvPath = Path.Combine(directory, "duplicate.csv");
            var mismatchedCsvPath = Path.Combine(directory, "mismatched.csv");
            var unreadableCsvPath = Path.Combine(directory, "unreadable.csv");
            File.WriteAllText(capturePath, Serialize(CreateUnreadableOnlyCapture("validation-capture")));
            File.WriteAllText(duplicateCsvPath, "UID,CA\n1001,120\n1001,120\n");
            File.WriteAllText(mismatchedCsvPath, "UID,CA\n1002,120\n");
            File.WriteAllText(unreadableCsvPath, "UID,Zero\n1001,0\n");

            var duplicate = Run(
                "correlate",
                "--csv",
                duplicateCsvPath,
                "--capture",
                capturePath,
                "--uid-column",
                "UID",
                "--field",
                "ca=CA");
            Assert.NotEqual(0, duplicate.ExitCode);
            Assert.Contains("duplicate UID", duplicate.Error, StringComparison.OrdinalIgnoreCase);

            var mismatched = Run(
                "correlate",
                "--csv",
                mismatchedCsvPath,
                "--capture",
                capturePath,
                "--uid-column",
                "UID",
                "--field",
                "ca=CA");
            Assert.NotEqual(0, mismatched.ExitCode);
            Assert.Contains("UIDs differ", mismatched.Error, StringComparison.OrdinalIgnoreCase);

            var unreadable = Run(
                "correlate",
                "--csv",
                unreadableCsvPath,
                "--capture",
                capturePath,
                "--uid-column",
                "UID",
                "--field",
                "zero=Zero");
            Assert.Equal(0, unreadable.ExitCode);
            using var document = JsonDocument.Parse(unreadable.Output);
            Assert.Equal(1, document.RootElement.GetProperty("unreadableRangeCount").GetInt32());
            Assert.Equal("no-evidence", GetField(document.RootElement, "zero").GetProperty("outcome").GetString());
        }
        finally
        {
            Directory.Delete(directory, recursive: true);
        }
    }

    [Fact]
    public void Diff_rejects_incompatible_capture_metadata()
    {
        var directory = CreateTempDirectory();
        try
        {
            var beforeCapturePath = Path.Combine(directory, "before-probe.json");
            var afterCapturePath = Path.Combine(directory, "after-probe.json");
            var beforeCsvPath = Path.Combine(directory, "before.csv");
            var afterCsvPath = Path.Combine(directory, "after.csv");
            File.WriteAllText(beforeCapturePath, Serialize(CreateCapture("before-capture", Player(1001, 120, 160, 15, 10_000_000, 1), supportedGameVersion: "26.3")));
            File.WriteAllText(afterCapturePath, Serialize(CreateCapture("after-capture", Player(1001, 120, 160, 16, 10_000_000, 1), supportedGameVersion: "26.4")));
            File.WriteAllText(beforeCsvPath, "UID,Determination\n1001,15\n");
            File.WriteAllText(afterCsvPath, "UID,Determination\n1001,16\n");

            var result = Run(
                "diff",
                "--before-csv",
                beforeCsvPath,
                "--after-csv",
                afterCsvPath,
                "--before-capture",
                beforeCapturePath,
                "--after-capture",
                afterCapturePath,
                "--uid-column",
                "UID",
                "--field",
                "determination=Determination");

            Assert.NotEqual(0, result.ExitCode);
            Assert.Contains("incompatible", result.Error, StringComparison.OrdinalIgnoreCase);
        }
        finally
        {
            Directory.Delete(directory, recursive: true);
        }
    }

    [Fact]
    public void Diff_rejects_different_probe_schema_versions()
    {
        var directory = CreateTempDirectory();
        try
        {
            var beforeCapturePath = Path.Combine(directory, "before-probe.json");
            var afterCapturePath = Path.Combine(directory, "after-probe.json");
            var beforeCsvPath = Path.Combine(directory, "before.csv");
            var afterCsvPath = Path.Combine(directory, "after.csv");
            File.WriteAllText(
                beforeCapturePath,
                Serialize(
                    CreateCapture(
                        "before-capture",
                        Player(1001, 120, 160, 15, 10_000_000, 1),
                        schemaVersion: 1)));
            File.WriteAllText(
                afterCapturePath,
                Serialize(
                    CreateCapture(
                        "after-capture",
                        Player(1001, 120, 160, 16, 10_000_000, 1))));
            File.WriteAllText(beforeCsvPath, "UID,Determination\n1001,15\n");
            File.WriteAllText(afterCsvPath, "UID,Determination\n1001,16\n");

            var result = Run(
                "diff",
                "--before-csv",
                beforeCsvPath,
                "--after-csv",
                afterCsvPath,
                "--before-capture",
                beforeCapturePath,
                "--after-capture",
                afterCapturePath,
                "--uid-column",
                "UID",
                "--field",
                "determination=Determination");

            Assert.NotEqual(0, result.ExitCode);
            Assert.Contains("schemaVersion", result.Error, StringComparison.Ordinal);
        }
        finally
        {
            Directory.Delete(directory, recursive: true);
        }
    }

    private static (int ExitCode, string Output, string Error) Run(params string[] args)
    {
        using var output = new StringWriter();
        using var error = new StringWriter();
        var exitCode = MemoryProbeCli.Run(args, output, error);
        return (exitCode, output.ToString(), error.ToString());
    }

    private static void AssertCandidate(JsonElement document, string name, string expectedPath, string expectedEncoding)
    {
        var field = GetField(document, name);
        Assert.Equal("candidate", field.GetProperty("outcome").GetString());
        Assert.Equal(expectedPath, field.GetProperty("candidates")[0].GetProperty("path").GetString());
        Assert.Equal(expectedEncoding, field.GetProperty("candidates")[0].GetProperty("encoding").GetString());
    }

    private static void AssertNormalizedCandidate(
        JsonElement document,
        string name,
        string expectedNormalization,
        string expectedPath,
        string expectedEncoding,
        string expectedEvidenceKind,
        IReadOnlyList<uint> expectedEligibleUids,
        IReadOnlyList<uint> expectedExcludedUids)
    {
        var field = GetField(document, name);
        Assert.Equal("candidate", field.GetProperty("outcome").GetString());
        Assert.Equal(expectedNormalization, field.GetProperty("normalization").GetString());
        Assert.Equal(expectedEligibleUids, field.GetProperty("eligibleUids").EnumerateArray().Select(value => value.GetUInt32()).ToArray());
        Assert.Equal(expectedExcludedUids, field.GetProperty("excludedUids").EnumerateArray().Select(value => value.GetUInt32()).ToArray());
        Assert.Equal(expectedPath, field.GetProperty("candidates")[0].GetProperty("path").GetString());
        Assert.Equal(expectedEncoding, field.GetProperty("candidates")[0].GetProperty("encoding").GetString());
        Assert.Equal(expectedEvidenceKind, field.GetProperty("candidates")[0].GetProperty("evidenceKind").GetString());
    }

    private static JsonElement GetField(JsonElement document, string name) =>
        document.GetProperty("fields").EnumerateArray().Single(field => field.GetProperty("name").GetString() == name);

    private static ProbeDocument CreateCapture(
        string requestId,
        PlayerFixture first,
        PlayerFixture? second = null,
        PlayerFixture? third = null,
        bool includeUnreadableRange = false,
        string supportedGameVersion = "26.3",
        string? generatedAtUtc = null,
        int schemaVersion = ProbeProtocol.SchemaVersion)
    {
        var players = new[] { first, second, third }
            .Where(player => player is not null)
            .Cast<PlayerFixture>()
            .Select(player => CreateProbePlayer(player, includeUnreadableRange))
            .ToArray();
        return new ProbeDocument
        {
            SchemaVersion = schemaVersion,
            GeneratedAtUtc = generatedAtUtc ?? "2026-08-07T12:00:00.0000000+00:00",
            GameVersion = "26.3.2.2329565",
            SupportedGameVersion = supportedGameVersion,
            BridgeVersion = "0.1.0",
            ProtocolVersion = ProbeProtocol.ProtocolVersion,
            RequestId = requestId,
            RequestedUids = players.Select(player => player.Uid).ToArray(),
            PlayerCount = players.Length,
            CapturePolicy = schemaVersion == ProbeProtocol.SchemaVersion ? CreateCapturePolicy() : null,
            Players = players,
        };
    }

    private static ProbeDocument CreateUnreadableOnlyCapture(string requestId) =>
        new()
        {
            SchemaVersion = ProbeProtocol.SchemaVersion,
            GeneratedAtUtc = "2026-08-07T12:00:00.0000000+00:00",
            GameVersion = "26.3.2.2329565",
            SupportedGameVersion = "26.3",
            BridgeVersion = "0.1.0",
            ProtocolVersion = ProbeProtocol.ProtocolVersion,
            RequestId = requestId,
            RequestedUids = new[] { 1001u },
            PlayerCount = 1,
            CapturePolicy = CreateCapturePolicy(),
            Players = new[]
            {
                new ProbePlayer
                {
                    Uid = 1001,
                    CandidateAddress = 0x100000,
                    ClassOffset = 0x288,
                    PlayerBlockAddress = 0x0FFD78,
                    RequestedBytes = 2,
                    ReadableBytes = 1,
                    Ranges = new[]
                    {
                        new ProbeMemoryRange
                        {
                            AddressBasis = "pointer-target",
                            RelativePath = "player-block+0x20->target+0x0",
                            SourcePointerPath = "player-block+0x20",
                            Address = 0x200000,
                            RequestedLength = 2,
                            PointerDepth = 1,
                            ReadableSpans = new[]
                            {
                                new ProbeReadableSpan { Offset = 0, BytesBase64 = Convert.ToBase64String(new byte[] { 1 }) },
                            },
                        },
                    },
                },
            },
        };

    private static ProbeCapturePolicy CreateCapturePolicy() =>
        new()
        {
            MaxPointerDepth = 1,
            TargetWindowBytes = 128,
            MaxBytesPerPlayer = 2_944,
            MaxBytesPerRequest = 376_832,
            PathQuotas = new[]
            {
                new ProbePointerPathQuota
                {
                    AddressBasis = "player-block",
                    PointerDepth = 1,
                    MaxPaths = 8,
                },
                new ProbePointerPathQuota
                {
                    AddressBasis = "person-object",
                    PointerDepth = 1,
                    MaxPaths = 8,
                },
            },
        };

    private static ProbePlayer CreateProbePlayer(PlayerFixture fixture, bool includeUnreadableRange)
    {
        var playerBytes = new byte[0x280];
        BinaryPrimitives.WriteUInt16LittleEndian(playerBytes.AsSpan(0x264), fixture.Ca);
        BinaryPrimitives.WriteUInt16LittleEndian(playerBytes.AsSpan(0x266), fixture.Pa);
        BinaryPrimitives.WriteUInt32LittleEndian(playerBytes.AsSpan(0x234), fixture.MarketValue);
        playerBytes[0x192] = checked((byte)(fixture.Determination * 5));
        BinaryPrimitives.WriteUInt16LittleEndian(playerBytes.AsSpan(0x10), fixture.DecoyCa);
        BinaryPrimitives.WriteInt16LittleEndian(playerBytes.AsSpan(0x30), fixture.SignedValue);
        playerBytes[0x38] = fixture.RawByteValue;
        playerBytes[0x39] = unchecked((byte)fixture.SignedByteValue);
        BinaryPrimitives.WriteInt32LittleEndian(playerBytes.AsSpan(0x3C), fixture.SignedInt32Value);
        BinaryPrimitives.WriteUInt16LittleEndian(playerBytes.AsSpan(0x60), fixture.Starts);
        BinaryPrimitives.WriteUInt16LittleEndian(playerBytes.AsSpan(0x62), fixture.Substitutes);
        BinaryPrimitives.WriteInt32LittleEndian(playerBytes.AsSpan(0x64), BitConverter.SingleToInt32Bits(fixture.Rating));
        BinaryPrimitives.WriteUInt16LittleEndian(playerBytes.AsSpan(0x68), fixture.DistanceTenths);
        playerBytes[0x6A] = fixture.SparseValue;

        var personBytes = new byte[0x100];
        BinaryPrimitives.WriteUInt32LittleEndian(personBytes.AsSpan(0x0C), fixture.Uid);
        var ranges = new List<ProbeMemoryRange>
        {
            Range("player-block", "player-block+0x0", playerBytes),
            Range("person-object", "person-object+0x0", personBytes),
        };
        if (includeUnreadableRange)
        {
            ranges.Add(
                new ProbeMemoryRange
                {
                    AddressBasis = "pointer-target",
                    RelativePath = "player-block+0x20->target+0x0",
                    SourcePointerPath = "player-block+0x20",
                    Address = 0x200000,
                    RequestedLength = 2,
                    PointerDepth = 1,
                    ReadableSpans = new[]
                    {
                        new ProbeReadableSpan { Offset = 0, BytesBase64 = Convert.ToBase64String(new byte[] { 1 }) },
                    },
                });
        }

        return new ProbePlayer
        {
            Uid = fixture.Uid,
            CandidateAddress = 0x100000 + fixture.Uid,
            ClassOffset = 0x288,
            PlayerBlockAddress = 0x100000 + fixture.Uid - 0x288,
            RequestedBytes = ranges.Sum(range => range.RequestedLength),
            ReadableBytes = ranges.Sum(range => range.ReadableSpans.Sum(span => Convert.FromBase64String(span.BytesBase64).Length)),
            Ranges = ranges,
        };
    }

    private static ProbeMemoryRange Range(string addressBasis, string relativePath, byte[] bytes) =>
        new()
        {
            AddressBasis = addressBasis,
            RelativePath = relativePath,
            Address = 0x100000,
            RequestedLength = bytes.Length,
            PointerDepth = 0,
            ReadableSpans = new[]
            {
                new ProbeReadableSpan { Offset = 0, BytesBase64 = Convert.ToBase64String(bytes) },
            },
        };

    private static PlayerFixture Player(
        uint uid,
        ushort ca,
        ushort pa,
        int determination,
        uint marketValue,
        ushort decoyCa,
        short signedValue = -1,
        byte rawByteValue = 0,
        sbyte signedByteValue = 0,
        int signedInt32Value = 0,
        ushort starts = 0,
        ushort substitutes = 0,
        float rating = 0f,
        ushort distanceTenths = 0,
        byte sparseValue = 0) =>
        new(
            uid,
            ca,
            pa,
            determination,
            marketValue,
            decoyCa,
            signedValue,
            rawByteValue,
            signedByteValue,
            signedInt32Value,
            starts,
            substitutes,
            rating,
            distanceTenths,
            sparseValue);

    private static string Serialize<T>(T value) =>
        JsonSerializer.Serialize(value, new JsonSerializerOptions { PropertyNamingPolicy = JsonNamingPolicy.CamelCase });

    private static string CreateTempDirectory()
    {
        var path = Path.Combine(Path.GetTempPath(), "fm-memory-probe-cli-tests", Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(path);
        return path;
    }

    private sealed record PlayerFixture(
        uint Uid,
        ushort Ca,
        ushort Pa,
        int Determination,
        uint MarketValue,
        ushort DecoyCa,
        short SignedValue,
        byte RawByteValue,
        sbyte SignedByteValue,
        int SignedInt32Value,
        ushort Starts,
        ushort Substitutes,
        float Rating,
        ushort DistanceTenths,
        byte SparseValue);
}
