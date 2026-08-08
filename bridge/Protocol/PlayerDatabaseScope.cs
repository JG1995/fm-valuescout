using FmDataBridge.Models;

namespace FmDataBridge.Protocol;

public enum PlayerDatabaseScope
{
    Men,
    Women,
    Both,
}

public static class PlayerDatabaseScopes
{
    public const string Men = "men";
    public const string Women = "women";
    public const string Both = "both";

    public static bool TryParse(string? value, out PlayerDatabaseScope scope)
    {
        scope = value switch
        {
            Men => PlayerDatabaseScope.Men,
            Women => PlayerDatabaseScope.Women,
            Both => PlayerDatabaseScope.Both,
            _ => default,
        };

        return value is Men or Women or Both;
    }

    public static string ToWireValue(PlayerDatabaseScope scope) => scope switch
    {
        PlayerDatabaseScope.Men => Men,
        PlayerDatabaseScope.Women => Women,
        PlayerDatabaseScope.Both => Both,
        _ => throw new ArgumentOutOfRangeException(nameof(scope), scope, "Unsupported player database scope."),
    };

    public static bool Includes(PlayerDatabaseScope scope, PlayerGender gender) => scope switch
    {
        PlayerDatabaseScope.Men => gender is PlayerGender.Male or PlayerGender.Unknown,
        PlayerDatabaseScope.Women => gender == PlayerGender.Female,
        PlayerDatabaseScope.Both => true,
        _ => false,
    };
}
