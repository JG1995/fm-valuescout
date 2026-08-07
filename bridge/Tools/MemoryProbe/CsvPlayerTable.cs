using System.Globalization;
using System.Text;

namespace FmDataBridge.MemoryProbe;

internal sealed record FieldMapping(string Name, string CsvColumn)
{
    public static IReadOnlyList<FieldMapping> Parse(IReadOnlyList<string> values)
    {
        if (values.Count == 0)
        {
            throw new MemoryProbeException("at least one --field metric=CSV header mapping is required");
        }

        var mappings = new List<FieldMapping>();
        var names = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
        foreach (var value in values)
        {
            var separator = value.IndexOf('=');
            if (separator <= 0 || separator == value.Length - 1)
            {
                throw new MemoryProbeException($"invalid --field mapping '{value}'; use metric=CSV header");
            }

            var name = value[..separator].Trim();
            var column = value[(separator + 1)..].Trim();
            if (name.Length == 0 || column.Length == 0 || !names.Add(name))
            {
                throw new MemoryProbeException($"field mapping '{value}' has an empty or duplicate metric name");
            }

            mappings.Add(new FieldMapping(name, column));
        }

        return mappings;
    }
}

internal sealed record CsvPlayer(uint Uid, IReadOnlyDictionary<string, long> Values, int RowNumber);

internal sealed class CsvPlayerTable
{
    private CsvPlayerTable(string delimiterName, IReadOnlyList<CsvPlayer> players)
    {
        DelimiterName = delimiterName;
        Players = players;
    }

    public string DelimiterName { get; }

    public IReadOnlyList<CsvPlayer> Players { get; }

    public static CsvPlayerTable Load(
        string path,
        string uidColumn,
        IReadOnlyList<FieldMapping> mappings,
        string? delimiterOption)
    {
        if (string.IsNullOrWhiteSpace(path))
        {
            throw new MemoryProbeException("CSV path is required");
        }

        if (!File.Exists(path))
        {
            throw new MemoryProbeException($"CSV file does not exist: {path}");
        }

        string contents;
        try
        {
            contents = File.ReadAllText(path);
        }
        catch (Exception ex)
        {
            throw new MemoryProbeException($"could not read CSV file '{path}': {ex.Message}");
        }

        var delimiter = CsvParser.ResolveDelimiter(contents, delimiterOption, out var delimiterName);
        var rows = CsvParser.Parse(contents, delimiter);
        if (rows.Count < 2)
        {
            throw new MemoryProbeException("CSV must contain a header and at least one player row");
        }

        var headerIndex = BuildHeaderIndex(rows[0]);
        var uidIndex = FindHeader(headerIndex, uidColumn, "UID");
        var fieldIndexes = mappings.ToDictionary(
            mapping => mapping.Name,
            mapping => FindHeader(headerIndex, mapping.CsvColumn, $"field '{mapping.Name}'"),
            StringComparer.OrdinalIgnoreCase);
        var players = new List<CsvPlayer>();
        var seenUids = new HashSet<uint>();
        for (var rowIndex = 1; rowIndex < rows.Count; rowIndex++)
        {
            var row = rows[rowIndex];
            var displayRow = rowIndex + 1;
            if (row.Length != rows[0].Length)
            {
                throw new MemoryProbeException($"CSV row {displayRow} has {row.Length} columns; expected {rows[0].Length}");
            }

            var uidText = row[uidIndex].Trim();
            if (!uint.TryParse(uidText, NumberStyles.None, CultureInfo.InvariantCulture, out var uid)
                || uid == 0
                || uid == uint.MaxValue)
            {
                throw new MemoryProbeException($"CSV row {displayRow} has an invalid UID '{row[uidIndex]}'");
            }

            if (!seenUids.Add(uid))
            {
                throw new MemoryProbeException($"CSV row {displayRow} has duplicate UID {uid}");
            }

            var values = new Dictionary<string, long>(StringComparer.OrdinalIgnoreCase);
            foreach (var mapping in mappings)
            {
                var raw = row[fieldIndexes[mapping.Name]];
                if (!TryParseNumeric(raw, out var value))
                {
                    throw new MemoryProbeException(
                        $"CSV row {displayRow} field '{mapping.CsvColumn}' has unsupported numeric value '{raw}'");
                }

                if (value < int.MinValue || value > uint.MaxValue)
                {
                    throw new MemoryProbeException(
                        $"CSV row {displayRow} field '{mapping.CsvColumn}' is outside supported 8/16/32-bit scalar values");
                }

                values.Add(mapping.Name, value);
            }

            players.Add(new CsvPlayer(uid, values, displayRow));
        }

        return new CsvPlayerTable(delimiterName, players);
    }

    private static Dictionary<string, int> BuildHeaderIndex(IReadOnlyList<string> header)
    {
        var index = new Dictionary<string, int>(StringComparer.OrdinalIgnoreCase);
        for (var column = 0; column < header.Count; column++)
        {
            var value = header[column].Trim().TrimStart('\uFEFF');
            if (value.Length == 0)
            {
                throw new MemoryProbeException($"CSV header column {column + 1} is empty");
            }

            if (!index.TryAdd(value, column))
            {
                throw new MemoryProbeException($"CSV has duplicate header '{value}'");
            }
        }

        return index;
    }

    private static int FindHeader(IReadOnlyDictionary<string, int> headerIndex, string name, string purpose)
    {
        if (string.IsNullOrWhiteSpace(name) || !headerIndex.TryGetValue(name.Trim(), out var index))
        {
            throw new MemoryProbeException($"CSV does not contain the {purpose} column '{name}'");
        }

        return index;
    }

    private static bool TryParseNumeric(string raw, out long value)
    {
        var normalized = raw.Trim().Replace(" ", string.Empty).Replace("\u00A0", string.Empty).Replace("'", string.Empty);
        if (long.TryParse(
                normalized,
                NumberStyles.AllowLeadingSign | NumberStyles.AllowThousands,
                CultureInfo.InvariantCulture,
                out value))
        {
            return true;
        }

        if (HasThreeDigitGroups(normalized, '.'))
        {
            return long.TryParse(
                normalized.Replace(".", string.Empty),
                NumberStyles.AllowLeadingSign,
                CultureInfo.InvariantCulture,
                out value);
        }

        value = default;
        return false;
    }

    private static bool HasThreeDigitGroups(string value, char separator)
    {
        var unsigned = value.StartsWith('+') || value.StartsWith('-') ? value[1..] : value;
        var groups = unsigned.Split(separator);
        return groups.Length > 1
            && groups[0].Length is > 0 and <= 3
            && groups[0].All(char.IsDigit)
            && groups.Skip(1).All(group => group.Length == 3 && group.All(char.IsDigit));
    }
}

internal static class CsvParser
{
    public static char ResolveDelimiter(string contents, string? option, out string name)
    {
        if (!string.IsNullOrWhiteSpace(option))
        {
            return option.Trim().ToLowerInvariant() switch
            {
                "comma" => Named(',', "comma", out name),
                "semicolon" => Named(';', "semicolon", out name),
                "tab" => Named('\t', "tab", out name),
                _ => throw new MemoryProbeException("--delimiter must be comma, semicolon, or tab"),
            };
        }

        var counts = new Dictionary<char, int>
        {
            [','] = 0,
            [';'] = 0,
            ['\t'] = 0,
        };
        var inQuotes = false;
        for (var index = 0; index < contents.Length; index++)
        {
            var character = contents[index];
            if (character == '"')
            {
                if (inQuotes && index + 1 < contents.Length && contents[index + 1] == '"')
                {
                    index++;
                    continue;
                }

                inQuotes = !inQuotes;
                continue;
            }

            if (!inQuotes && (character == '\r' || character == '\n'))
            {
                break;
            }

            if (!inQuotes && counts.ContainsKey(character))
            {
                counts[character]++;
            }
        }

        if (inQuotes)
        {
            throw new MemoryProbeException("CSV has an unclosed quoted header field");
        }

        var highest = counts.Values.Max();
        var winners = counts.Where(pair => pair.Value == highest).Select(pair => pair.Key).ToArray();
        if (highest == 0 || winners.Length != 1)
        {
            throw new MemoryProbeException("could not detect CSV delimiter; pass --delimiter comma, semicolon, or tab");
        }

        return winners[0] switch
        {
            ',' => Named(',', "comma", out name),
            ';' => Named(';', "semicolon", out name),
            _ => Named('\t', "tab", out name),
        };
    }

    public static IReadOnlyList<string[]> Parse(string contents, char delimiter)
    {
        var rows = new List<string[]>();
        var row = new List<string>();
        var field = new StringBuilder();
        var inQuotes = false;
        var closedQuote = false;

        void EndRow()
        {
            row.Add(field.ToString());
            rows.Add(row.ToArray());
            row.Clear();
            field.Clear();
            closedQuote = false;
        }

        for (var index = 0; index < contents.Length; index++)
        {
            var character = contents[index];
            if (inQuotes)
            {
                if (character == '"')
                {
                    if (index + 1 < contents.Length && contents[index + 1] == '"')
                    {
                        field.Append(character);
                        index++;
                    }
                    else
                    {
                        inQuotes = false;
                        closedQuote = true;
                    }
                }
                else
                {
                    field.Append(character);
                }

                continue;
            }

            if (closedQuote && character != delimiter && character != '\r' && character != '\n')
            {
                throw new MemoryProbeException("CSV has text after a closing quote");
            }

            if (character == '"')
            {
                if (field.Length != 0)
                {
                    throw new MemoryProbeException("CSV has a quote inside an unquoted field");
                }

                inQuotes = true;
            }
            else if (character == delimiter)
            {
                row.Add(field.ToString());
                field.Clear();
                closedQuote = false;
            }
            else if (character == '\r' || character == '\n')
            {
                if (character == '\r' && index + 1 < contents.Length && contents[index + 1] == '\n')
                {
                    index++;
                }

                EndRow();
            }
            else
            {
                field.Append(character);
            }
        }

        if (inQuotes)
        {
            throw new MemoryProbeException("CSV has an unclosed quoted field");
        }

        if (field.Length != 0 || row.Count != 0 || closedQuote)
        {
            EndRow();
        }

        return rows;
    }

    private static char Named(char delimiter, string delimiterName, out string name)
    {
        name = delimiterName;
        return delimiter;
    }
}
