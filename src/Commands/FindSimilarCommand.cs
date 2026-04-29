using System.ComponentModel;
using Exa.Console.Infrastructure;
using Spectre.Console.Cli;

namespace Exa.Console.Commands;

public sealed class FindSimilarCommand : AsyncCommand<FindSimilarCommand.Settings>
{
    public sealed class Settings : GlobalSettings
    {
        [CommandArgument(0, "<URL>")]
        [Description("Reference URL to find similar pages for")]
        public required string Url { get; init; }

        [CommandOption("--num <COUNT>")]
        [Description("Number of results (1-100, default 10)")]
        public int? NumResults { get; init; }

        [CommandOption("--include-domains <DOMAINS>")]
        [Description("Comma-separated domain whitelist")]
        public string? IncludeDomains { get; init; }

        [CommandOption("--exclude-domains <DOMAINS>")]
        [Description("Comma-separated domain blacklist")]
        public string? ExcludeDomains { get; init; }

        [CommandOption("--start-date <DATE>")]
        [Description("Minimum published date (ISO 8601)")]
        public string? StartPublishedDate { get; init; }

        [CommandOption("--end-date <DATE>")]
        [Description("Maximum published date (ISO 8601)")]
        public string? EndPublishedDate { get; init; }

        [CommandOption("--highlights")]
        [Description("Return query-relevant excerpts")]
        public bool Highlights { get; init; }

        [CommandOption("--highlights-chars <COUNT>")]
        [Description("Max characters for highlights")]
        public int? HighlightsMaxChars { get; init; }

        [CommandOption("--text")]
        [Description("Return full page text as markdown")]
        public bool Text { get; init; }

        [CommandOption("--text-chars <COUNT>")]
        [Description("Max characters for text")]
        public int? TextMaxChars { get; init; }

        [CommandOption("--summary")]
        [Description("Return LLM-generated summary")]
        public bool Summary { get; init; }
    }

    protected override async Task<int> ExecuteAsync(CommandContext context, Settings settings, CancellationToken cancellation)
    {
        using var client = settings.CreateClient();

        var body = new Dictionary<string, object> { ["url"] = settings.Url };

        if (settings.NumResults is not null) body["numResults"] = settings.NumResults;
        if (settings.StartPublishedDate is not null) body["startPublishedDate"] = settings.StartPublishedDate;
        if (settings.EndPublishedDate is not null) body["endPublishedDate"] = settings.EndPublishedDate;

        if (settings.IncludeDomains is not null)
            body["includeDomains"] = settings.IncludeDomains.Split(',',
                StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);

        if (settings.ExcludeDomains is not null)
            body["excludeDomains"] = settings.ExcludeDomains.Split(',',
                StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);

        var contents = new Dictionary<string, object>();

        if (settings.Highlights || settings.HighlightsMaxChars is not null)
        {
            var highlights = new Dictionary<string, object>();
            if (settings.HighlightsMaxChars is not null) highlights["maxCharacters"] = settings.HighlightsMaxChars;
            contents["highlights"] = highlights.Count > 0 ? highlights : true;
        }

        if (settings.Text || settings.TextMaxChars is not null)
        {
            if (settings.TextMaxChars is not null)
                contents["text"] = new Dictionary<string, object> { ["maxCharacters"] = settings.TextMaxChars };
            else
                contents["text"] = true;
        }

        if (settings.Summary)
            contents["summary"] = true;

        if (contents.Count > 0) body["contents"] = contents;

        var result = await client.PostAsync("findSimilar", body);
        YamlOutput.Write(result);
        return 0;
    }
}
