using System.Text.Json;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace Exa.Console.Infrastructure;

public static class YamlOutput
{
    private static readonly ISerializer Serializer = new SerializerBuilder()
        .WithNamingConvention(CamelCaseNamingConvention.Instance)
        .Build();

    public static void Write(JsonDocument doc)
    {
        var obj = JsonToObject(doc.RootElement);
        System.Console.WriteLine(Serializer.Serialize(obj).TrimEnd());
    }

    public static void WriteError(string message, int code)
    {
        var error = new Dictionary<string, object> { ["error"] = message, ["code"] = code };
        System.Console.Error.WriteLine(Serializer.Serialize(error).TrimEnd());
    }

    private static object? JsonToObject(JsonElement element) => element.ValueKind switch
    {
        JsonValueKind.Object => element.EnumerateObject()
            .ToDictionary(p => p.Name, p => JsonToObject(p.Value)),
        JsonValueKind.Array => element.EnumerateArray().Select(JsonToObject).ToList(),
        JsonValueKind.String => element.GetString(),
        JsonValueKind.Number => element.TryGetInt64(out var l) ? l : element.GetDouble(),
        JsonValueKind.True => true,
        JsonValueKind.False => false,
        _ => null
    };
}
