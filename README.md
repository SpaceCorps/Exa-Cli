# Exa.Console

CLI for the [Exa](https://exa.ai) AI search API. YAML-first output optimized for LLM agent consumption.

## Install

```bash
dotnet tool install --global Exa.Console
```

## Authentication

Set the `EXA_API_KEY` environment variable, or pass `--api-key` on every command.

## Commands

### search

Search the web with natural language.

```bash
exa search "latest developments in quantum computing"
exa search "senior ML engineers at fintech companies" --category people --highlights
exa search "AI regulation updates" --category news --start-date 2025-01-01 --num 20
exa search "agtech companies series A" --category company --highlights --highlights-chars 2000
exa search "react state management" --include-domains "github.com,dev.to" --text
exa search "frontier AI models comparison" --type deep --summary
```

### contents

Extract clean, LLM-ready content from URLs.

```bash
exa contents "https://arxiv.org/abs/2301.07041" --text
exa contents "https://example.com/article" --highlights --highlights-query "methodology"
exa contents "https://stripe.com" --summary --subpages 5 --subpage-target "about,careers,press"
```

### answer

Get an LLM-generated answer with citations.

```bash
exa answer "What is the latest valuation of SpaceX?"
exa answer "How does transformer attention work?" --text
```

### find-similar

Find pages similar to a given URL.

```bash
exa find-similar "https://arxiv.org/abs/2307.06435" --num 5
exa find-similar "https://example.com/blog" --highlights --exclude-domains "example.com"
```

## License

MIT
