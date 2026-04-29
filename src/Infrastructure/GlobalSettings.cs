using System.ComponentModel;
using Spectre.Console.Cli;

namespace Exa.Console.Infrastructure;

public class GlobalSettings : CommandSettings
{
    [CommandOption("--api-key <KEY>")]
    [Description("Exa API key (or set EXA_API_KEY env var)")]
    public string? ApiKey { get; init; }

    public ExaClient CreateClient()
    {
        var key = ApiKey ?? Environment.GetEnvironmentVariable("EXA_API_KEY")
            ?? throw new InvalidOperationException(
                "API key required. Use --api-key or set EXA_API_KEY.");
        return new ExaClient(key);
    }
}
