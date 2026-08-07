using System.Text.Json;
using FmDataBridge.Protocol;

namespace FmDataBridge.MemoryProbe;

public static class MemoryProbeCli
{
    private const int DefaultTimeoutSeconds = 120;
    private const int MaximumTimeoutSeconds = 300;

    private static readonly JsonSerializerOptions OutputSerializerOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        WriteIndented = true,
    };

    private static readonly JsonSerializerOptions ProtocolSerializerOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        PropertyNameCaseInsensitive = true,
        WriteIndented = true,
    };

    public static int Run(string[] args, TextWriter output, TextWriter error)
    {
        ArgumentNullException.ThrowIfNull(args);
        ArgumentNullException.ThrowIfNull(output);
        ArgumentNullException.ThrowIfNull(error);

        try
        {
            if (args.Length == 0 || args[0] is "--help" or "help")
            {
                output.WriteLine(HelpText);
                return 0;
            }

            var command = args[0];
            var options = CommandArguments.Parse(args[1..]);
            if (options.Has("help"))
            {
                output.WriteLine(HelpText);
                return 0;
            }

            object report = command switch
            {
                "capture" => Capture(options),
                "correlate" => Correlate(options),
                "diff" => Diff(options),
                _ => throw new MemoryProbeException($"unknown command '{command}'; use capture, correlate, diff, or --help"),
            };
            output.WriteLine(JsonSerializer.Serialize(report, OutputSerializerOptions));
            return 0;
        }
        catch (MemoryProbeException ex)
        {
            error.WriteLine($"memory-probe: {ex.Message}");
            return 2;
        }
        catch (Exception ex)
        {
            error.WriteLine($"memory-probe: unexpected failure: {ex.Message}");
            return 1;
        }
    }

    private static CaptureReport Capture(CommandArguments options)
    {
        options.RequireOnly("csv", "uid-column", "bridge-dir", "request-id", "timeout-seconds", "delimiter");
        var table = CsvPlayerTable.Load(
            options.RequireOne("csv"),
            options.RequireOne("uid-column"),
            Array.Empty<FieldMapping>(),
            options.OptionalOne("delimiter"));
        var bridgeDirectory = Path.GetFullPath(options.OptionalOne("bridge-dir") ?? BridgePaths.EnsureBridgeDirectory());
        var requestId = options.OptionalOne("request-id") ?? $"memory-probe-{Guid.NewGuid():N}";
        var timeout = TimeSpan.FromSeconds(ParseTimeout(options.OptionalOne("timeout-seconds")));
        var request = new ProbeRequest
        {
            ProtocolVersion = ProbeProtocol.ProtocolVersion,
            RequestId = requestId,
            CreatedAtUtc = DateTimeOffset.UtcNow,
            Uids = table.Players.Select(player => player.Uid).ToArray(),
        };
        if (!ProbeRequestAcceptance.TryValidateForCapture(request, out var invalidReason))
        {
            throw new MemoryProbeException(invalidReason ?? "probe request is invalid");
        }

        WriteRequest(bridgeDirectory, request);
        var capture = AwaitCapture(bridgeDirectory, request, timeout);
        return new CaptureReport(
            request.RequestId,
            capture.Document.PlayerCount,
            BridgePaths.GetProbePath(bridgeDirectory),
            request.Uids,
            table.DelimiterName);
    }

    private static CorrelationReport Correlate(CommandArguments options)
    {
        options.RequireOnly("csv", "capture", "uid-column", "field", "delimiter");
        var mappings = FieldMapping.Parse(options.Many("field"));
        var table = CsvPlayerTable.Load(
            options.RequireOne("csv"),
            options.RequireOne("uid-column"),
            mappings,
            options.OptionalOne("delimiter"));
        var capturePath = Path.GetFullPath(options.RequireOne("capture"));
        var capture = ProbeCapture.Load(capturePath);
        capture.RequireUids(table.Players.Select(player => player.Uid), "CSV");
        return ProbeAnalysis.Correlate(capture, table, capturePath, mappings);
    }

    private static DiffReport Diff(CommandArguments options)
    {
        options.RequireOnly(
            "before-csv",
            "after-csv",
            "before-capture",
            "after-capture",
            "uid-column",
            "field",
            "delimiter");
        var mappings = FieldMapping.Parse(options.Many("field"));
        var uidColumn = options.RequireOne("uid-column");
        var delimiter = options.OptionalOne("delimiter");
        var beforeTable = CsvPlayerTable.Load(options.RequireOne("before-csv"), uidColumn, mappings, delimiter);
        var afterTable = CsvPlayerTable.Load(options.RequireOne("after-csv"), uidColumn, mappings, delimiter);
        RequireSameUids(beforeTable.Players, afterTable.Players, "before CSV", "after CSV");

        var beforePath = Path.GetFullPath(options.RequireOne("before-capture"));
        var afterPath = Path.GetFullPath(options.RequireOne("after-capture"));
        var beforeCapture = ProbeCapture.Load(beforePath);
        var afterCapture = ProbeCapture.Load(afterPath);
        ProbeCapture.RequireCompatible(beforeCapture, afterCapture);
        beforeCapture.RequireUids(beforeTable.Players.Select(player => player.Uid), "before CSV");
        afterCapture.RequireUids(afterTable.Players.Select(player => player.Uid), "after CSV");
        return ProbeAnalysis.Diff(
            beforeCapture,
            afterCapture,
            beforeTable,
            afterTable,
            beforePath,
            afterPath,
            mappings);
    }

    private static void WriteRequest(string bridgeDirectory, ProbeRequest request)
    {
        Directory.CreateDirectory(bridgeDirectory);
        var requestPath = BridgePaths.GetProbeRequestPath(bridgeDirectory);
        if (File.Exists(requestPath))
        {
            throw new MemoryProbeException($"a probe request is already pending at {requestPath}");
        }

        EnsureRequestIdIsUnused(bridgeDirectory, request.RequestId);
        var temporaryPath = requestPath + ".tmp";
        try
        {
            File.WriteAllText(temporaryPath, JsonSerializer.Serialize(request, ProtocolSerializerOptions));
            File.Move(temporaryPath, requestPath);
        }
        catch (Exception ex)
        {
            throw new MemoryProbeException($"could not write probe request: {ex.Message}");
        }
        finally
        {
            TryDelete(temporaryPath);
        }
    }

    private static void EnsureRequestIdIsUnused(string bridgeDirectory, string requestId)
    {
        var status = TryReadStatus(BridgePaths.GetProbeStatusPath(bridgeDirectory));
        if (string.Equals(status?.RequestId, requestId, StringComparison.Ordinal))
        {
            throw new MemoryProbeException(
                $"probe request ID '{requestId}' must not be reused while its prior probe status remains; use a new --request-id");
        }

        var captureRequestId = TryReadCaptureRequestId(BridgePaths.GetProbePath(bridgeDirectory));
        if (string.Equals(captureRequestId, requestId, StringComparison.Ordinal))
        {
            throw new MemoryProbeException(
                $"probe request ID '{requestId}' must not be reused while its prior probe capture remains; use a new --request-id");
        }
    }

    private static ProbeCapture AwaitCapture(string bridgeDirectory, ProbeRequest request, TimeSpan timeout)
    {
        var deadline = DateTimeOffset.UtcNow + timeout;
        string? lastOtherRequestId = null;
        string? lastStaleResult = null;
        while (DateTimeOffset.UtcNow <= deadline)
        {
            var status = TryReadStatus(BridgePaths.GetProbeStatusPath(bridgeDirectory));
            if (status is not null)
            {
                if (!string.Equals(status.RequestId, request.RequestId, StringComparison.Ordinal))
                {
                    lastOtherRequestId = status.RequestId;
                }
                else if (status.ProtocolVersion != ProbeProtocol.ProtocolVersion)
                {
                    throw new MemoryProbeException($"probe status has unsupported protocol version {status.ProtocolVersion}");
                }
                else if (status.UpdatedAtUtc <= request.CreatedAtUtc)
                {
                    lastStaleResult = "latest matching status predates this request";
                }
                else if (status.State == ProbeProtocol.StateFailed)
                {
                    throw new MemoryProbeException($"probe request {request.RequestId} failed: {status.Error ?? "no error message"}");
                }
                else if (status.State == ProbeProtocol.StateReady)
                {
                    var capturePath = BridgePaths.GetProbePath(bridgeDirectory);
                    var capture = ProbeCapture.Load(capturePath);
                    if (!string.Equals(capture.Document.RequestId, request.RequestId, StringComparison.Ordinal))
                    {
                        throw new MemoryProbeException(
                            $"ready status for {request.RequestId} points to stale capture {capture.Document.RequestId}");
                    }

                    if (capture.GeneratedAtUtc <= request.CreatedAtUtc)
                    {
                        lastStaleResult = "latest matching capture predates this request";
                    }
                    else
                    {
                        capture.RequireUids(request.Uids, "probe request");
                        if (status.PlayersCaptured is { } playersCaptured && playersCaptured != capture.Document.PlayerCount)
                        {
                            throw new MemoryProbeException(
                                $"ready status reports {playersCaptured} players but capture contains {capture.Document.PlayerCount}");
                        }

                        return capture;
                    }
                }
            }

            Thread.Sleep(100);
        }

        var suffix = lastStaleResult is not null
            ? $"; {lastStaleResult}"
            : lastOtherRequestId is not null
                ? $"; latest status belonged to {lastOtherRequestId}"
                : string.Empty;
        throw new MemoryProbeException($"timed out waiting for probe request {request.RequestId}{suffix}");
    }

    private static ProbeStatus? TryReadStatus(string statusPath)
    {
        if (!File.Exists(statusPath))
        {
            return null;
        }

        try
        {
            return JsonSerializer.Deserialize<ProbeStatus>(File.ReadAllText(statusPath), ProtocolSerializerOptions);
        }
        catch (JsonException)
        {
            return null;
        }
        catch (IOException)
        {
            return null;
        }
    }

    private static string? TryReadCaptureRequestId(string capturePath)
    {
        if (!File.Exists(capturePath))
        {
            return null;
        }

        try
        {
            using var document = JsonDocument.Parse(File.ReadAllText(capturePath));
            return document.RootElement.TryGetProperty("requestId", out var requestId)
                && requestId.ValueKind == JsonValueKind.String
                ? requestId.GetString()
                : null;
        }
        catch (JsonException)
        {
            return null;
        }
        catch (IOException)
        {
            return null;
        }
    }

    private static int ParseTimeout(string? value)
    {
        if (value is null)
        {
            return DefaultTimeoutSeconds;
        }

        if (!int.TryParse(value, out var seconds) || seconds is < 1 or > MaximumTimeoutSeconds)
        {
            throw new MemoryProbeException($"--timeout-seconds must be between 1 and {MaximumTimeoutSeconds}");
        }

        return seconds;
    }

    private static void RequireSameUids(
        IReadOnlyList<CsvPlayer> first,
        IReadOnlyList<CsvPlayer> second,
        string firstName,
        string secondName)
    {
        var firstUids = first.Select(player => player.Uid).ToHashSet();
        var secondUids = second.Select(player => player.Uid).ToHashSet();
        if (firstUids.SetEquals(secondUids))
        {
            return;
        }

        throw new MemoryProbeException(DescribeUidDifference(firstUids, secondUids, firstName, secondName));
    }

    internal static string DescribeUidDifference(
        IReadOnlySet<uint> expected,
        IReadOnlySet<uint> actual,
        string expectedName,
        string actualName)
    {
        var missing = expected.Except(actual).OrderBy(uid => uid).ToArray();
        var extra = actual.Except(expected).OrderBy(uid => uid).ToArray();
        return $"UIDs differ between {expectedName} and {actualName}; missing: {FormatUids(missing)}; extra: {FormatUids(extra)}";
    }

    private static string FormatUids(IReadOnlyList<uint> uids) =>
        uids.Count == 0 ? "none" : string.Join(", ", uids);

    private static void TryDelete(string path)
    {
        try
        {
            if (File.Exists(path))
            {
                File.Delete(path);
            }
        }
        catch
        {
            // A failed temporary cleanup cannot replace a request file.
        }
    }

    private const string HelpText = "Usage: ./scripts/dev memory-probe {capture|correlate|diff} [options]\n"
        + "\n"
        + "capture --csv <path> --uid-column <header> [--bridge-dir <path>] [--request-id <unique-id>] [--timeout-seconds <1-300>] [--delimiter comma|semicolon|tab]\n"
        + "correlate --csv <path> --capture <probe.json> --uid-column <header> --field <metric=CSV header> [--field ...] [--delimiter comma|semicolon|tab]\n"
        + "diff --before-csv <path> --after-csv <path> --before-capture <probe.json> --after-capture <probe.json> --uid-column <header> --field <metric=CSV header> [--field ...] [--delimiter comma|semicolon|tab]\n"
        + "\n"
        + "The tool reports hypotheses only. It never verifies a production memory offset.";
}

internal sealed class CommandArguments
{
    private readonly Dictionary<string, List<string>> _values;

    private CommandArguments(Dictionary<string, List<string>> values)
    {
        _values = values;
    }

    public static CommandArguments Parse(IReadOnlyList<string> args)
    {
        var values = new Dictionary<string, List<string>>(StringComparer.Ordinal);
        for (var index = 0; index < args.Count; index++)
        {
            var option = args[index];
            if (!option.StartsWith("--", StringComparison.Ordinal))
            {
                throw new MemoryProbeException($"unexpected argument '{option}'");
            }

            var name = option[2..];
            if (name == "help")
            {
                values[name] = new List<string>();
                continue;
            }

            if (name.Length == 0 || index + 1 >= args.Count || args[index + 1].StartsWith("--", StringComparison.Ordinal))
            {
                throw new MemoryProbeException($"option {option} requires a value");
            }

            if (!values.TryGetValue(name, out var valuesForOption))
            {
                valuesForOption = new List<string>();
                values.Add(name, valuesForOption);
            }

            valuesForOption.Add(args[++index]);
        }

        return new CommandArguments(values);
    }

    public bool Has(string name) => _values.ContainsKey(name);

    public string RequireOne(string name)
    {
        var value = OptionalOne(name);
        return value ?? throw new MemoryProbeException($"--{name} is required");
    }

    public string? OptionalOne(string name)
    {
        if (!_values.TryGetValue(name, out var values))
        {
            return null;
        }

        if (values.Count != 1)
        {
            throw new MemoryProbeException($"--{name} may be supplied once");
        }

        return values[0];
    }

    public IReadOnlyList<string> Many(string name) =>
        _values.TryGetValue(name, out var values) ? values : Array.Empty<string>();

    public void RequireOnly(params string[] allowed)
    {
        var allowedSet = new HashSet<string>(allowed, StringComparer.Ordinal) { "help" };
        var unknown = _values.Keys.Where(name => !allowedSet.Contains(name)).OrderBy(name => name).ToArray();
        if (unknown.Length != 0)
        {
            throw new MemoryProbeException($"unsupported option(s): {string.Join(", ", unknown.Select(name => $"--{name}"))}");
        }
    }
}

internal sealed class MemoryProbeException : Exception
{
    public MemoryProbeException(string message)
        : base(message)
    {
    }
}

internal sealed record CaptureReport(
    string RequestId,
    int PlayersCaptured,
    string CapturePath,
    IReadOnlyList<uint> Uids,
    string Delimiter);
