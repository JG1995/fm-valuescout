using System.Buffers.Binary;
using System.Diagnostics;
using System.Text;
using System.Text.Json;
using FmDataBridge.Extraction;
using FmDataBridge.Layouts;
using FmDataBridge.Memory;

namespace FmDataBridge.Scanning;

/// <summary>
/// TEMPORARY FM27 team-layout diagnostic (BACKLOG: FM27 team detection). After a successful
/// scan it writes <see cref="OutputFileName"/> beside the dump: every walked club's team objects
/// as raw bytes with each pointer classified as a known person, contract, club, or team; the
/// contract team of every person; and, for focus clubs, vectors, sub-objects, strings, and the
/// pointer paths from each person to a team. One run should locate the team type, squad list,
/// staff team, loan, manager, reputation, and competition fields. Remove it once the layout is
/// fixed.
/// </summary>
internal sealed class TeamLayoutDiagnostic
{
    internal const string OutputFileName = "team-diagnostic.jsonl";
    internal const string RequestFileName = "team-diagnostic-clubs.txt";

    private const int TeamBytes = 0x200;
    private const int FocusObjectBytes = 0x400;
    private const int SubObjectBytes = 0x200;
    private const int ContractBytes = 0x100;
    private const int PersonTailBytes = 0x400;
    private const int MaxTeamsPerClub = 24;
    private const int MaxVectorEntries = 512;
    private const int MaxWrapperProbes = 64;
    private const int WrapperProbeBytes = 0x100;
    private const int MaxSubObjectRefs = 64;
    private const int TopClubs = 25;
    private const int DeepTopClubs = 5;
    private const int MaxPathsPerPerson = 200;
    private const ulong MaxPersonSpan = 0x800;
    private static readonly TimeSpan FocusBudget = TimeSpan.FromSeconds(35);
    private static readonly TimeSpan TotalBudget = TimeSpan.FromSeconds(50);
    private static readonly int[] PersonPartOffsets = { 0x100, 0x288, 0x380, 0x450 };

    private readonly IMemoryReader _reader;
    private readonly IFmMemoryLayout _layout;
    private readonly Stopwatch _clock = Stopwatch.StartNew();
    private readonly Dictionary<ulong, Ref> _refs = new();
    private readonly Dictionary<ulong, TeamInfo> _teams = new();
    private readonly Dictionary<ulong, ClubInfo> _clubs = new();
    private readonly Dictionary<uint, List<string[]>> _inVectors = new();
    private readonly List<string> _skipped = new();
    private ulong[] _personBlocks = Array.Empty<ulong>();
    private Ref[] _personBlockRefs = Array.Empty<Ref>();

    private TeamLayoutDiagnostic(IMemoryReader reader, IFmMemoryLayout layout)
    {
        _reader = reader;
        _layout = layout;
    }

    internal sealed record Person(
        uint Uid,
        bool IsPlayer,
        string? Name,
        int? BirthYear,
        int Ca,
        int Pa,
        int? JobId,
        ulong BlockAddress,
        ulong ObjectAddress,
        ulong ContractTeamAddress,
        ulong ContractClubAddress);

    /// <summary>Writes the diagnostic; any failure goes to an error file and never fails the scan.</summary>
    internal static void TryWrite(
        IMemoryReader reader,
        IFmMemoryLayout layout,
        string bridgeDirectory,
        string gameVersion,
        IEnumerable<ulong> clubAddresses,
        IReadOnlyList<Person> persons,
        CancellationToken cancellationToken)
    {
        var errorPath = Path.Combine(bridgeDirectory, "team-diagnostic-error.txt");
        try
        {
            File.Delete(errorPath);
            new TeamLayoutDiagnostic(reader, layout).Write(
                bridgeDirectory,
                gameVersion,
                clubAddresses,
                persons,
                cancellationToken);
        }
        catch (Exception exception)
        {
            try
            {
                File.WriteAllText(errorPath, exception.ToString(), Encoding.UTF8);
            }
            catch (IOException)
            {
            }
        }
    }

    private void Write(
        string bridgeDirectory,
        string gameVersion,
        IEnumerable<ulong> clubAddresses,
        IReadOnlyList<Person> persons,
        CancellationToken cancellationToken)
    {
        var requested = ReadRequestedClubs(bridgeDirectory);

        // Real objects first, so a guessed person part or contract never hides one.
        foreach (var person in persons)
        {
            _refs.TryAdd(person.ObjectAddress, new Ref(RefKind.Person, person.Uid, -1, person.IsPlayer));
            _refs.TryAdd(person.BlockAddress, new Ref(RefKind.Person, person.Uid, 0, person.IsPlayer));
        }

        foreach (var club in clubAddresses.Where(address => address != 0).Distinct().OrderBy(address => address))
        {
            IndexClub(club);
        }

        IndexContractsAndPersonParts(persons);
        var blocks = persons
            .GroupBy(person => person.BlockAddress)
            .Select(group => group.First())
            .OrderBy(person => person.BlockAddress)
            .ToList();
        _personBlocks = blocks.Select(person => person.BlockAddress).ToArray();
        _personBlockRefs = blocks.Select(person => new Ref(RefKind.Person, person.Uid, 0, person.IsPlayer)).ToArray();

        var contracted = new Dictionary<ulong, (int Players, int Staff)>();
        var staffJobs = new Dictionary<string, int>();
        var playerBirthYears = new Dictionary<string, int>();
        var personsByClub = new Dictionary<ulong, List<Person>>();
        foreach (var person in persons)
        {
            var index = TeamIndex(person.ContractTeamAddress);
            if (_teams.ContainsKey(person.ContractTeamAddress))
            {
                contracted.TryGetValue(person.ContractTeamAddress, out var counts);
                contracted[person.ContractTeamAddress] = person.IsPlayer
                    ? (counts.Players + 1, counts.Staff)
                    : (counts.Players, counts.Staff + 1);
            }

            var club = _clubs.ContainsKey(person.ContractClubAddress)
                ? person.ContractClubAddress
                : _teams.TryGetValue(person.ContractTeamAddress, out var team) ? team.Club : 0;
            if (club != 0)
            {
                if (!personsByClub.TryGetValue(club, out var members))
                {
                    members = new List<Person>();
                    personsByClub[club] = members;
                }

                members.Add(person);
            }

            var key = person.IsPlayer
                ? $"{person.BirthYear?.ToString() ?? "?"}|{index}"
                : $"{person.JobId?.ToString() ?? "?"}|{index}";
            var histogram = person.IsPlayer ? playerBirthYears : staffJobs;
            histogram.TryGetValue(key, out var n);
            histogram[key] = n + 1;
        }

        var bySize = personsByClub
            .OrderByDescending(entry => entry.Value.Count)
            .ThenBy(entry => entry.Key)
            .Select(entry => entry.Key)
            .ToList();
        var requestedClubs = _clubs
            .Where(entry => requested.Contains(entry.Value.Uid))
            .Select(entry => entry.Key)
            .OrderBy(address => address)
            .ToList();
        var focus = requestedClubs.Concat(bySize.Take(TopClubs)).Distinct().ToList();
        var deep = requestedClubs.Concat(bySize.Take(DeepTopClubs)).ToHashSet();
        var withHex = requestedClubs.ToHashSet();

        Directory.CreateDirectory(bridgeDirectory);
        var path = Path.Combine(bridgeDirectory, OutputFileName);
        var tempPath = path + ".tmp";
        using (var writer = new StreamWriter(tempPath, append: false, new UTF8Encoding(false)))
        {
            Emit(writer, new
            {
                type = "meta",
                generatedAtUtc = DateTimeOffset.UtcNow.ToString("O"),
                gameVersion,
                layout = new
                {
                    objectUid = Hex(_layout.ObjectUidOffset),
                    fullContractPtr = Hex(_layout.FullContractPtrOffset),
                    contractTeamPtr = Hex(_layout.ContractTeamPtrOffset),
                    contractJobId = Hex(_layout.ContractJobIdOffset),
                    teamClubPtr = Hex(_layout.TeamClubPtrOffset),
                    teamType = Hex(_layout.TeamTypeOffset),
                    teamReputation = Hex(_layout.TeamReputationOffset),
                    teamManagerPtr = Hex(_layout.TeamManagerPtrOffset),
                    teamCompPtr = Hex(_layout.TeamCompPtrOffset),
                    teamCompAltPtr = Hex(_layout.TeamCompAltPtrOffset),
                    teamSquadBegin = Hex(_layout.TeamSquadBeginOffset),
                    clubTeamsBegin = Hex(_layout.ClubTeamsBeginOffset),
                    compName = Hex(_layout.CompNameOffset),
                },
                players = persons.Count(person => person.IsPlayer),
                staff = persons.Count(person => !person.IsPlayer),
                clubs = _clubs.Count,
                teams = _teams.Count,
                requestedClubUids = requested.OrderBy(uid => uid).ToArray(),
                focusClubUids = focus.Select(club => _clubs[club].Uid).ToArray(),
                deepClubUids = deep.Select(club => _clubs[club].Uid).OrderBy(uid => uid).ToArray(),
            });
            Emit(writer, new { type = "histogram", name = "staffJobIdByContractTeamIndex", counts = staffJobs });
            Emit(writer, new { type = "histogram", name = "playerBirthYearByContractTeamIndex", counts = playerBirthYears });

            var written = new HashSet<ulong>();
            foreach (var club in focus)
            {
                if (cancellationToken.IsCancellationRequested || _clock.Elapsed > FocusBudget)
                {
                    _skipped.Add($"focus club {_clubs[club].Uid}");
                    continue;
                }

                EmitClub(writer, club, detailed: true, contracted);
                foreach (var team in _clubs[club].Teams)
                {
                    EmitTeam(writer, team, detailed: true, contracted);
                    written.Add(team);
                }
            }

            var emittedPersons = new HashSet<uint>();
            var personsByUid = persons.GroupBy(person => person.Uid).ToDictionary(group => group.Key, group => group.First());
            foreach (var club in focus)
            {
                if (!personsByClub.TryGetValue(club, out var members))
                {
                    continue;
                }

                foreach (var person in members.OrderBy(person => person.Uid))
                {
                    if (cancellationToken.IsCancellationRequested || _clock.Elapsed > FocusBudget)
                    {
                        _skipped.Add($"persons of club {_clubs[club].Uid}");
                        break;
                    }

                    if (emittedPersons.Add(person.Uid))
                    {
                        EmitPerson(writer, person, deep.Contains(club), withHex.Contains(club));
                    }
                }
            }

            // Persons who sit in a focus team's vectors without a contract there, such as loanees.
            foreach (var uid in _inVectors.Keys.OrderBy(uid => uid))
            {
                if (cancellationToken.IsCancellationRequested || _clock.Elapsed > FocusBudget)
                {
                    _skipped.Add("vector-only persons");
                    break;
                }

                if (personsByUid.TryGetValue(uid, out var person) && emittedPersons.Add(uid))
                {
                    EmitPerson(writer, person, deep: true, hex: false);
                }
            }

            foreach (var team in _teams.Keys.OrderBy(address => address))
            {
                if (written.Contains(team))
                {
                    continue;
                }

                if (cancellationToken.IsCancellationRequested || _clock.Elapsed > TotalBudget)
                {
                    _skipped.Add("remaining teams");
                    break;
                }

                EmitTeam(writer, team, detailed: false, contracted);
            }

            foreach (var club in _clubs.Keys.OrderBy(address => address))
            {
                if (!focus.Contains(club))
                {
                    EmitClub(writer, club, detailed: false, contracted);
                }
            }

            Emit(writer, new
            {
                type = "done",
                elapsedMs = _clock.ElapsedMilliseconds,
                cancelled = cancellationToken.IsCancellationRequested,
                skipped = _skipped,
            });
        }

        File.Move(tempPath, path, overwrite: true);
    }

    private static HashSet<uint> ReadRequestedClubs(string bridgeDirectory)
    {
        var path = Path.Combine(bridgeDirectory, RequestFileName);
        if (!File.Exists(path))
        {
            return new HashSet<uint>();
        }

        return File.ReadAllLines(path)
            .Select(line => uint.TryParse(line.Trim(), out var uid) ? uid : 0)
            .Where(uid => uid != 0)
            .ToHashSet();
    }

    private void IndexContractsAndPersonParts(IReadOnlyList<Person> persons)
    {
        foreach (var person in persons)
        {
            if (TryReadPointer(person.ObjectAddress, _layout.FullContractPtrOffset, out var contract)
                && IsPlausiblePointer(contract))
            {
                _refs.TryAdd(contract, new Ref(RefKind.Contract, person.Uid, 0, person.IsPlayer));
            }
        }

        foreach (var person in persons)
        {
            foreach (var part in PersonPartOffsets)
            {
                if (TryAdd(person.BlockAddress, part, out var address))
                {
                    _refs.TryAdd(address, new Ref(RefKind.Person, person.Uid, part, person.IsPlayer));
                }
            }
        }
    }

    private void IndexClub(ulong club)
    {
        if (_clubs.ContainsKey(club))
        {
            return;
        }

        var uid = _reader.TryReadUInt32(club + (ulong)_layout.ObjectUidOffset, out var value) ? value : 0;
        var teams = new List<ulong>();
        if (TryReadPointer(club, _layout.ClubTeamsBeginOffset, out var begin)
            && TryReadPointer(club, _layout.ClubTeamsEndOffset, out var end)
            && begin != 0
            && end > begin
            && begin % 8 == 0
            && (end - begin) % 8 == 0
            && (end - begin) / 8 <= MaxTeamsPerClub)
        {
            var count = (int)((end - begin) / 8);
            for (var index = 0; index < count; index++)
            {
                if (_reader.TryReadUInt64(begin + (ulong)(index * 8), out var team) && team != 0)
                {
                    teams.Add(team);
                    if (_teams.TryAdd(team, new TeamInfo(club, uid, index, count)))
                    {
                        _refs[team] = new Ref(RefKind.Team, uid, index, false);
                    }
                }
            }
        }

        _clubs[club] = new ClubInfo(uid, ClubNameReader.TryRead(_reader, club, _layout), teams);
        _refs[club] = new Ref(RefKind.Club, uid, 0, false);
    }

    private int TeamIndex(ulong team) =>
        team == 0 ? -2 : _teams.TryGetValue(team, out var info) ? info.Index : -1;

    private void EmitClub(
        StreamWriter writer,
        ulong club,
        bool detailed,
        Dictionary<ulong, (int Players, int Staff)> contracted)
    {
        var info = _clubs[club];
        var players = 0;
        var staff = 0;
        foreach (var team in info.Teams)
        {
            if (contracted.TryGetValue(team, out var counts))
            {
                players += counts.Players;
                staff += counts.Staff;
            }
        }

        if (!detailed)
        {
            Emit(writer, new
            {
                type = "club",
                addr = Hex(club),
                uid = info.Uid,
                name = info.Name,
                teams = info.Teams.Select(Hex).ToArray(),
                contractedPlayers = players,
                contractedStaff = staff,
            });
            return;
        }

        var survey = Survey(club, FocusObjectBytes, $"club:{info.Uid}", detailed: true);
        Emit(writer, new
        {
            type = "club",
            focus = true,
            addr = Hex(club),
            uid = info.Uid,
            name = info.Name,
            teams = info.Teams.Select(Hex).ToArray(),
            contractedPlayers = players,
            contractedStaff = staff,
            hex = survey.Hex,
            partial = survey.Partial,
            ptrs = survey.Ptrs,
            vectors = survey.Vectors,
            wrapped = survey.Wrapped,
            strings = survey.Strings,
            sub = survey.Sub,
        });
    }

    private void EmitTeam(
        StreamWriter writer,
        ulong team,
        bool detailed,
        Dictionary<ulong, (int Players, int Staff)> contracted)
    {
        var info = _teams[team];
        contracted.TryGetValue(team, out var counts);
        var survey = Survey(
            team,
            detailed ? FocusObjectBytes : TeamBytes,
            $"team:{info.ClubUid}:{info.Index}",
            detailed);
        Emit(writer, new
        {
            type = "team",
            focus = detailed,
            addr = Hex(team),
            clubUid = info.ClubUid,
            index = info.Index,
            count = info.Count,
            contractedPlayers = counts.Players,
            contractedStaff = counts.Staff,
            hex = survey.Hex,
            partial = survey.Partial,
            ptrs = survey.Ptrs,
            vectors = survey.Vectors,
            wrapped = survey.Wrapped,
            strings = survey.Strings,
            sub = survey.Sub,
        });
    }

    private void EmitPerson(StreamWriter writer, Person person, bool deep, bool hex)
    {
        var start = person.BlockAddress;
        var length = (int)Math.Min(
            0x1000UL,
            person.ObjectAddress - person.BlockAddress + PersonTailBytes);
        var bytes = new byte[length];
        var readable = _reader.TryReadBlock(start, bytes, 0, length, out var bytesRead) || bytesRead > 0;
        var paths = new List<string[]>();
        if (readable)
        {
            for (var offset = 0; offset + 8 <= length && paths.Count < MaxPathsPerPerson; offset += 8)
            {
                var value = BinaryPrimitives.ReadUInt64LittleEndian(bytes.AsSpan(offset, 8));
                if (_refs.TryGetValue(value, out var target))
                {
                    if (!(target.Kind == RefKind.Person && target.Uid == person.Uid))
                    {
                        paths.Add(new[] { $"b+{Hex(offset)}", target.Describe() });
                    }

                    continue;
                }

                if (!deep || !IsPlausiblePointer(value))
                {
                    continue;
                }

                var sub = new byte[SubObjectBytes];
                if (!_reader.TryReadBlock(value, sub, 0, sub.Length, out var subRead) && subRead == 0)
                {
                    continue;
                }

                for (var inner = 0; inner + 8 <= sub.Length && paths.Count < MaxPathsPerPerson; inner += 8)
                {
                    var innerValue = BinaryPrimitives.ReadUInt64LittleEndian(sub.AsSpan(inner, 8));
                    if (_refs.TryGetValue(innerValue, out var innerTarget)
                        && innerTarget.Kind != RefKind.Person)
                    {
                        paths.Add(new[] { $"b+{Hex(offset)}>+{Hex(inner)}", innerTarget.Describe() });
                    }
                }
            }
        }

        object? contract = null;
        if (TryReadPointer(person.ObjectAddress, _layout.FullContractPtrOffset, out var contractAddress)
            && IsPlausiblePointer(contractAddress))
        {
            var contractBytes = new byte[ContractBytes];
            var contractRead = _reader.TryReadBlock(contractAddress, contractBytes, 0, ContractBytes, out var count)
                || count > 0;
            contract = new
            {
                addr = Hex(contractAddress),
                hex = contractRead ? Convert.ToHexString(contractBytes) : null,
                ptrs = contractRead ? ClassifyPointers(contractBytes, ContractBytes) : new List<string[]>(),
            };
        }

        _inVectors.TryGetValue(person.Uid, out var inVectors);
        Emit(writer, new
        {
            type = "person",
            uid = person.Uid,
            kind = person.IsPlayer ? "p" : "s",
            name = person.Name,
            birthYear = person.BirthYear,
            ca = person.Ca,
            pa = person.Pa,
            jobId = person.JobId,
            block = Hex(person.BlockAddress),
            classOffset = Hex(person.ObjectAddress - person.BlockAddress),
            contractTeam = _refs.TryGetValue(person.ContractTeamAddress, out var contractTeam)
                ? contractTeam.Describe()
                : Hex(person.ContractTeamAddress),
            contract,
            paths,
            inVectors = inVectors ?? new List<string[]>(),
            hex = hex && readable ? Convert.ToHexString(bytes) : null,
        });
    }

    /// <summary>Raw bytes of one object plus every classified pointer, vector, and string in it.</summary>
    private Survey Survey(ulong address, int length, string owner, bool detailed)
    {
        var bytes = new byte[length];
        var complete = _reader.TryReadBlock(address, bytes, 0, length, out var bytesRead);
        if (!complete && bytesRead == 0)
        {
            return new Survey(null, true, new(), new(), new(), new(), new());
        }

        var ptrs = ClassifyPointers(bytes, length);
        var wrapped = new List<object>();
        var vectors = FindVectors(bytes, length, owner, "", detailed, wrapped);
        var strings = new List<string[]>();
        var sub = new List<object>();
        if (detailed)
        {
            for (var offset = 0; offset + 8 <= length; offset += 8)
            {
                var value = BinaryPrimitives.ReadUInt64LittleEndian(bytes.AsSpan(offset, 8));
                if (!IsPlausiblePointer(value) || _refs.ContainsKey(value))
                {
                    continue;
                }

                var text = FmStringReader.TryReadIndirect(_reader, address + (ulong)offset);
                if (IsPrintable(text))
                {
                    strings.Add(new[] { Hex(offset), text! });
                }

                var subObject = SurveySubObject(value, owner, Hex(offset), wrapped);
                if (subObject is not null)
                {
                    sub.Add(subObject);
                }
            }
        }

        return new Survey(Convert.ToHexString(bytes), !complete, ptrs, vectors, wrapped, strings, sub);
    }

    private object? SurveySubObject(ulong address, string owner, string offset, List<object> wrapped)
    {
        var bytes = new byte[SubObjectBytes];
        if (!_reader.TryReadBlock(address, bytes, 0, bytes.Length, out var bytesRead) && bytesRead == 0)
        {
            return null;
        }

        var ptrs = ClassifyPointers(bytes, bytes.Length);
        var vectors = FindVectors(bytes, bytes.Length, owner, offset + ">", detailed: true, wrapped);
        string? comp = null;
        foreach (var nameOffset in new[] { _layout.CompNameOffset, _layout.CompShortNameOffset })
        {
            var text = FmStringReader.TryReadIndirect(_reader, address + (ulong)nameOffset);
            if (IsPrintable(text))
            {
                comp = text;
                break;
            }
        }

        if (ptrs.Count == 0 && vectors.Count == 0 && comp is null)
        {
            return null;
        }

        return new
        {
            off = offset,
            addr = Hex(address),
            ptrs = ptrs.Take(MaxSubObjectRefs).ToList(),
            vectors,
            comp,
        };
    }

    private List<string[]> ClassifyPointers(byte[] bytes, int length)
    {
        var ptrs = new List<string[]>();
        for (var offset = 0; offset + 8 <= length; offset += 8)
        {
            var value = BinaryPrimitives.ReadUInt64LittleEndian(bytes.AsSpan(offset, 8));
            if (_refs.TryGetValue(value, out var target))
            {
                ptrs.Add(new[] { Hex(offset), target.Describe() });
            }
        }

        return ptrs;
    }

    /// <summary>
    /// Begin/end pointer pairs whose entries resolve to known objects, directly or through a
    /// wrapper object that holds one within its first bytes.
    /// </summary>
    private List<object> FindVectors(
        byte[] bytes,
        int length,
        string owner,
        string prefix,
        bool detailed,
        List<object> wrapped)
    {
        var vectors = new List<object>();
        for (var offset = 0; offset + 16 <= length; offset += 8)
        {
            var begin = BinaryPrimitives.ReadUInt64LittleEndian(bytes.AsSpan(offset, 8));
            var end = BinaryPrimitives.ReadUInt64LittleEndian(bytes.AsSpan(offset + 8, 8));
            if (!IsPlausiblePointer(begin) || end <= begin || (end - begin) % 8 != 0)
            {
                continue;
            }

            var count = (end - begin) / 8;
            if (count > MaxVectorEntries)
            {
                continue;
            }

            var entries = new byte[(int)count * 8];
            if (!_reader.TryReadBlock(begin, entries, 0, entries.Length, out var entriesRead) && entriesRead == 0)
            {
                continue;
            }

            var kinds = new Dictionary<string, int>();
            var refs = new List<string>();
            var unknown = new List<ulong>();
            for (var index = 0; index < (int)count; index++)
            {
                var entry = BinaryPrimitives.ReadUInt64LittleEndian(entries.AsSpan(index * 8, 8));
                if (_refs.TryGetValue(entry, out var target))
                {
                    var kind = target.KindName();
                    kinds.TryGetValue(kind, out var n);
                    kinds[kind] = n + 1;
                    if (detailed)
                    {
                        refs.Add(target.Describe());
                        RecordInVector(target, owner, $"{prefix}{Hex(offset)}", "direct");
                    }
                }
                else if (detailed && TryInsidePerson(entry, out var inside, out var insideOffset))
                {
                    var kind = $"inside{(inside.IsPlayer ? "Player" : "Staff")}+{Hex(insideOffset)}";
                    kinds.TryGetValue(kind, out var n);
                    kinds[kind] = n + 1;
                    refs.Add($"{inside.Describe()}+{Hex(insideOffset)}");
                    RecordInVector(inside, owner, $"{prefix}{Hex(offset)}", $"inside+{Hex(insideOffset)}");
                }
                else if (IsPlausiblePointer(entry))
                {
                    unknown.Add(entry);
                }
            }

            if (detailed && unknown.Count > 0)
            {
                ProbeWrappers(unknown, owner, $"{prefix}{Hex(offset)}", wrapped);
            }

            if (kinds.Count == 0 && !detailed)
            {
                continue;
            }

            vectors.Add(new
            {
                off = $"{prefix}{Hex(offset)}",
                count,
                unknownPointers = unknown.Count,
                kinds,
                refs = detailed ? refs : null,
            });
        }

        return vectors;
    }

    private void ProbeWrappers(List<ulong> entries, string owner, string vectorOffset, List<object> wrapped)
    {
        var hits = new Dictionary<string, int>();
        var buffer = new byte[WrapperProbeBytes];
        foreach (var entry in entries.Take(MaxWrapperProbes))
        {
            if (!_reader.TryReadBlock(entry, buffer, 0, buffer.Length, out var bytesRead) && bytesRead == 0)
            {
                continue;
            }

            for (var offset = 0; offset + 8 <= buffer.Length; offset += 8)
            {
                var value = BinaryPrimitives.ReadUInt64LittleEndian(buffer.AsSpan(offset, 8));
                if (!_refs.TryGetValue(value, out var target))
                {
                    continue;
                }

                var key = $"{Hex(offset)}:{target.KindName()}";
                hits.TryGetValue(key, out var n);
                hits[key] = n + 1;
                RecordInVector(target, owner, vectorOffset, $"wrapper{Hex(offset)}");
            }
        }

        if (hits.Count > 0)
        {
            wrapped.Add(new { vector = vectorOffset, probed = Math.Min(entries.Count, MaxWrapperProbes), hits });
        }
    }

    private void RecordInVector(Ref target, string owner, string vectorOffset, string via)
    {
        if (target.Kind is not (RefKind.Person or RefKind.Contract))
        {
            return;
        }

        if (!_inVectors.TryGetValue(target.Uid, out var list))
        {
            list = new List<string[]>();
            _inVectors[target.Uid] = list;
        }

        if (list.Count < 32)
        {
            list.Add(new[] { owner, vectorOffset, via, target.Describe() });
        }
    }

    /// <summary>A pointer into a known person object, short of an exact match.</summary>
    private bool TryInsidePerson(ulong value, out Ref person, out int offset)
    {
        person = default;
        offset = 0;
        var index = Array.BinarySearch(_personBlocks, value);
        if (index < 0)
        {
            index = ~index - 1;
        }

        if (index < 0 || value - _personBlocks[index] >= MaxPersonSpan)
        {
            return false;
        }

        person = _personBlockRefs[index];
        offset = (int)(value - _personBlocks[index]);
        return true;
    }

    private bool TryReadPointer(ulong address, int offset, out ulong value)
    {
        value = 0;
        return TryAdd(address, offset, out var field) && _reader.TryReadUInt64(field, out value);
    }

    private static bool TryAdd(ulong address, int offset, out ulong result)
    {
        result = 0;
        if (offset < 0 || (ulong)offset > ulong.MaxValue - address)
        {
            return false;
        }

        result = address + (ulong)offset;
        return true;
    }

    private static bool IsPlausiblePointer(ulong value) =>
        value >= 0x10000 && value <= 0x7FFF_FFFF_FFFFUL && value % 8 == 0;

    private static bool IsPrintable(string? text) =>
        !string.IsNullOrWhiteSpace(text)
        && text.Length >= 3
        && text.Count(char.IsLetter) >= 2
        && text.All(c => !char.IsControl(c) && c != '\uFFFD');

    private static string Hex(int value) => $"0x{value:X}";

    private static string Hex(ulong value) => $"0x{value:X}";

    private static void Emit(StreamWriter writer, object line) =>
        writer.WriteLine(JsonSerializer.Serialize(line));

    private enum RefKind : byte
    {
        Person,
        Contract,
        Club,
        Team,
    }

    /// <summary>A known object: Extra is a team's index, or a person part (-1 object, 0 block, else block offset).</summary>
    private readonly record struct Ref(RefKind Kind, uint Uid, int Extra, bool IsPlayer)
    {
        public string KindName() => Kind switch
        {
            RefKind.Person => IsPlayer ? "player" : "staff",
            RefKind.Contract => IsPlayer ? "playerContract" : "staffContract",
            RefKind.Club => "club",
            _ => "team",
        };

        public string Describe() => Kind switch
        {
            RefKind.Person => $"{KindName()}:{Uid}:{(Extra == -1 ? "obj" : Extra == 0 ? "block" : $"b+0x{Extra:X}")}",
            RefKind.Contract => $"{KindName()}:{Uid}",
            RefKind.Club => $"club:{Uid}",
            _ => $"team:{Uid}:{Extra}",
        };
    }

    private sealed record TeamInfo(ulong Club, uint ClubUid, int Index, int Count);

    private sealed record ClubInfo(uint Uid, string? Name, List<ulong> Teams);
}

internal sealed record Survey(
    string? Hex,
    bool Partial,
    List<string[]> Ptrs,
    List<object> Vectors,
    List<object> Wrapped,
    List<string[]> Strings,
    List<object> Sub);
