using System.ComponentModel;
using Exa.Console.Infrastructure;
using Spectre.Console;
using Spectre.Console.Cli;

namespace Exa.Console.Commands;

public sealed class SearchCommand : AsyncCommand<SearchCommand.Settings>
{
    public sealed class Settings : GlobalSettings
    {
        [CommandArgument(0, "<QUERY>")]
        [Description("Natural language search query")]
        public required string Query { get; init; }

        [CommandOption("--num <COUNT>")]
        [Description("Number of results (1-100, default 10)")]
        public int? NumResults { get; init; }

        [CommandOption("--type <TYPE>")]
        [Description("Search type: auto, fast, instant, deep-lite, deep, deep-reasoning")]
        public string? Type { get; init; }

        [CommandOption("--category <CATEGORY>")]
        [Description("Category: company, people, news, research paper, personal site, financial report")]
        public string? Category { get; init; }

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
        [Description("Max characters for highlights (default 4000)")]
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

        [CommandOption("--user-location <CODE>")]
        [Description("Two-letter ISO country code")]
        public string? UserLocation { get; init; }
    }

    protected override async Task<int> ExecuteAsync(CommandContext context, Settings settings, CancellationToken cancellation)
    {
        using var client = settings.CreateClient();

        var body = new Dictionary<string, object> { ["query"] = settings.Query };

        if (settings.NumResults is not null) body["numResults"] = settings.NumResults;
        if (settings.Type is not null) body["type"] = settings.Type;
        if (settings.Category is not null) body["category"] = settings.Category;
        if (settings.UserLocation is not null) body["userLocation"] = settings.UserLocation;
        if (settings.StartPublishedDate is not null) body["startPublishedDate"] = settings.StartPublishedDate;
        if (settings.EndPublishedDate is not null) body["endPublishedDate"] = settings.EndPublishedDate;

        if (settings.IncludeDomains is not null)
            body["includeDomains"] = SplitDomains(settings.IncludeDomains);

        if (settings.ExcludeDomains is not null)
            body["excludeDomains"] = SplitDomains(settings.ExcludeDomains);

        var contents = BuildContents(settings);
        if (contents.Count > 0) body["contents"] = contents;

        var result = await client.PostAsync("search", body);
        YamlOutput.Write(result);
        return 0;
    }

    private static Dictionary<string, object> BuildContents(Settings settings)
    {
        var contents = new Dictionary<string, object>();

        if (settings.Highlights || settings.HighlightsMaxChars is not null)
        {
            var highlights = new Dictionary<string, object>();
            highlights["maxCharacters"] = settings.HighlightsMaxChars ?? 4000;
            contents["highlights"] = highlights;
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

        return contents;
    }

    private static string[] SplitDomains(string domains) =>
        domains.Split(',', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);
}
