using Exa.Console.Commands;
using Spectre.Console.Cli;

var app = new CommandApp();

app.Configure(config =>
{
    config.SetApplicationName("exa");

    config.AddCommand<SearchCommand>("search")
        .WithDescription("Search the web with natural language");

    config.AddCommand<ContentsCommand>("contents")
        .WithDescription("Extract clean content from URLs");

    config.AddCommand<AnswerCommand>("answer")
        .WithDescription("Get an LLM-generated answer with citations");

    config.AddCommand<FindSimilarCommand>("find-similar")
        .WithDescription("Find pages similar to a given URL");
});

return app.Run(args);
