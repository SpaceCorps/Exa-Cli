using System.ComponentModel;
using Exa.Console.Infrastructure;
using Spectre.Console;
using Spectre.Console.Cli;

namespace Exa.Console.Commands;

public sealed class ContentsCommand : AsyncCommand<ContentsCommand.Settings>
{
    public sealed class Settings : GlobalSettings
    {
        [CommandArgument(0, "<URLS>")]
        [Description("Comma-separated URLs to extract content from")]
        public required string Urls { get; init; }

        [CommandOption("--text")]
        [Description("Return full page text as markdown")]
        public bool Text { get; init; }

        [CommandOption("--text-chars <COUNT>")]
        [Description("Max characters for text")]
        public int? TextMaxChars { get; init; }

        [CommandOption("--highlights")]
        [Description("Return query-relevant excerpts")]
        public bool Highlights { get; init; }

        [CommandOption("--highlights-chars <COUNT>")]
        [Description("Max characters for highlights")]
        public int? HighlightsMaxChars { get; init; }

        [CommandOption("--highlights-query <QUERY>")]
        [Description("Custom query to direct highlight selection")]
        public string? HighlightsQuery { get; init; }

        [CommandOption("--summary")]
        [Description("Return LLM-generated summary")]
        public bool Summary { get; init; }

        [CommandOption("--summary-query <QUERY>")]
        [Description("Custom query for the summary")]
        public string? SummaryQuery { get; init; }

        [CommandOption("--max-age <HOURS>")]
        [Description("Max cache age in hours (0=always livecrawl, -1=cache only)")]
        public int? MaxAgeHours { get; init; }

        [CommandOption("--subpages <COUNT>")]
        [Description("Number of subpages to crawl per URL")]
        public int? Subpages { get; init; }

        [CommandOption("--subpage-target <KEYWORDS>")]
        [Description("Comma-separated keywords to prioritize subpage selection")]
        public string? SubpageTarget { get; init; }

        public override ValidationResult Validate()
        {
            if (!Text && TextMaxChars is null && !Highlights && HighlightsMaxChars is null
                && HighlightsQuery is null && !Summary && SummaryQuery is null)
                return ValidationResult.Error(
                    "Specify at least one content mode: --text, --highlights, or --summary.");

            return ValidationResult.Success();
        }
    }

    protected override async Task<int> ExecuteAsync(CommandContext context, Settings settings, CancellationToken cancellation)
    {
        using var client = settings.CreateClient();

        var urls = settings.Urls.Split(',',
            StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);

        var body = new Dictionary<string, object> { ["urls"] = urls };

        if (settings.Text || settings.TextMaxChars is not null)
        {
            if (settings.TextMaxChars is not null)
                body["text"] = new Dictionary<string, object> { ["maxCharacters"] = settings.TextMaxChars };
            else
                body["text"] = true;
        }

        if (settings.Highlights || settings.HighlightsMaxChars is not null || settings.HighlightsQuery is not null)
        {
            var highlights = new Dictionary<string, object>();
            if (settings.HighlightsMaxChars is not null) highlights["maxCharacters"] = settings.HighlightsMaxChars;
            if (settings.HighlightsQuery is not null) highlights["query"] = settings.HighlightsQuery;
            body["highlights"] = highlights.Count > 0 ? highlights : true;
        }

        if (settings.Summary || settings.SummaryQuery is not null)
        {
            if (settings.SummaryQuery is not null)
                body["summary"] = new Dictionary<string, object> { ["query"] = settings.SummaryQuery };
            else
                body["summary"] = true;
        }

        if (settings.MaxAgeHours is not null) body["maxAgeHours"] = settings.MaxAgeHours;
        if (settings.Subpages is not null) body["subpages"] = settings.Subpages;

        if (settings.SubpageTarget is not null)
        {
            var targets = settings.SubpageTarget.Split(',',
                StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);
            body["subpageTarget"] = targets;
        }

        var result = await client.PostAsync("contents", body);
        YamlOutput.Write(result);
        return 0;
    }
}
