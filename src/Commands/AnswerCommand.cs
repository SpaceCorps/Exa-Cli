using System.ComponentModel;
using Exa.Console.Infrastructure;
using Spectre.Console.Cli;

namespace Exa.Console.Commands;

public sealed class AnswerCommand : AsyncCommand<AnswerCommand.Settings>
{
    public sealed class Settings : GlobalSettings
    {
        [CommandArgument(0, "<QUERY>")]
        [Description("The question to answer")]
        public required string Query { get; init; }

        [CommandOption("--text")]
        [Description("Include full text content in citations")]
        public bool Text { get; init; }
    }

    protected override async Task<int> ExecuteAsync(CommandContext context, Settings settings, CancellationToken cancellation)
    {
        using var client = settings.CreateClient();

        var body = new Dictionary<string, object> { ["query"] = settings.Query };

        if (settings.Text) body["text"] = true;

        var result = await client.PostAsync("answer", body);
        YamlOutput.Write(result);
        return 0;
    }
}
