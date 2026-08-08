namespace FmDataBridge.Models;

public enum PlayerGender
{
    Unknown,
    Male,
    Female,
}

public static class PlayerGenderValues
{
    public const string Unknown = "unknown";
    public const string Male = "male";
    public const string Female = "female";

    public static string ToWireValue(PlayerGender gender) => gender switch
    {
        PlayerGender.Unknown => Unknown,
        PlayerGender.Male => Male,
        PlayerGender.Female => Female,
        _ => throw new ArgumentOutOfRangeException(nameof(gender), gender, "Unsupported player gender."),
    };
}
