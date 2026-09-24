//! The manual an agent reads before its first call. Markdown by default so it can be pasted
//! into a system prompt or a prompt context; `--json` gives the same rules as structured data.

use crate::{obj, output};

pub fn print() {
    if output::json() {
        output::write(&obj! {
            "tool" => "exa",
            "apiVersion" => API_VERSION,
            "rules" => RULES,
            "exitCodes" => obj! {
                "0" => "ok",
                "1" => "error - unclassified, report and stop",
                "2" => "network - retry once, then stop",
                "3" => "auth_required - stop, surface the remediation to a human",
                "4" => "not_found - do not retry",
                "5" => "rate_limited - back off before retrying",
                "6" => "invalid_input - fix the call arguments",
                "7" => "no_account - run exa accounts list or set EXA_API_KEY",
            },
        });
        return;
    }
    println!("{README}");
}

/// The Exa API version targeted by this CLI build.
pub const API_VERSION: &str = "1.0.0";

const RULES: &[&str] = &[
    "Pass --account or set EXA_API_KEY / --api-key. Multi-account setups should always pass --account.",
    "Run 'exa accounts list' if you do not know which accounts exist.",
    "On code auth_required, stop and surface the remediation string. Do not retry.",
    "At least one content mode (--text, --highlights, or --summary) is required for 'exa contents'.",
    "Use --json when you are going to parse the output in scripts or agent loops.",
    "Always check exit codes: non-zero indicates an error envelope on stderr.",
];

const README: &str = r#"# exa - agent operating manual

A CLI over the Exa AI search API: web search, similar page discovery, content extraction,
and LLM-generated answers. Results are YAML on stdout, errors are YAML on stderr, and `--json`
switches both to JSON.

## Authentication & Accounts

An API key can be supplied in three ways:
1. `--account <name>` (short `-a <name>`): Uses a saved account from the OS keystore.
2. `--api-key <key>`: Explicit API key on the command line.
3. `EXA_API_KEY` environment variable.

### Managing accounts

    exa login [<name>] [--api-key <key>]  # opens browser to copy API key
    exa accounts add <name> --api-key <key> [--force] [--no-verify]
    printf %s "$KEY" | exa accounts add <name> --api-key-stdin
    exa accounts list [--check]
    exa accounts test <name>
    exa accounts remove <name> --yes

`add` tests the key before saving it unless `--no-verify` is passed. The key is stored
securely in your OS keystore (macOS Keychain, Windows DPAPI, Linux Secret Service).
`list --check` tests stored keys against the Exa API.

## Core Commands

### Search

Search the web with natural language queries:

    exa search "latest developments in quantum computing"
    exa search "senior ML engineers at fintech companies" --category people --highlights
    exa search "AI regulation updates" --category news --start-date 2025-01-01 --num 20
    exa search "agtech companies series A" --category company --highlights --highlights-chars 2000
    exa search "react state management" --include-domains "github.com,dev.to" --text
    exa search "frontier AI models comparison" --type deep --summary

Options:
- `--num <COUNT>`: Number of results (1-100, default 10)
- `--type <TYPE>`: Search type (auto, fast, instant, deep-lite, deep, deep-reasoning)
- `--category <CATEGORY>`: Category filter (company, people, news, research paper, personal site, financial report)
- `--include-domains <DOMAINS>`: Comma-separated domain whitelist
- `--exclude-domains <DOMAINS>`: Comma-separated domain blacklist
- `--start-date <DATE>`: Minimum published date (ISO 8601)
- `--end-date <DATE>`: Maximum published date (ISO 8601)
- `--highlights`: Return query-relevant excerpts
- `--highlights-chars <COUNT>`: Max characters for highlights (default 4000)
- `--text`: Return full page text as markdown
- `--text-chars <COUNT>`: Max characters for text
- `--summary`: Return LLM-generated summary
- `--user-location <CODE>`: Two-letter ISO country code

### Contents

Extract clean, LLM-ready markdown content from one or more URLs:

    exa contents "https://arxiv.org/abs/2301.07041" --text
    exa contents "https://example.com/article" --highlights --highlights-query "methodology"
    exa contents "https://stripe.com" --summary --subpages 5 --subpage-target "about,careers,press"

Note: At least one content mode (`--text`, `--highlights`, or `--summary`) is required.

Options:
- `--text`: Return full page text as markdown
- `--text-chars <COUNT>`: Max characters for text
- `--highlights`: Return query-relevant excerpts
- `--highlights-chars <COUNT>`: Max characters for highlights
- `--highlights-query <QUERY>`: Custom query to direct highlight selection
- `--summary`: Return LLM-generated summary
- `--summary-query <QUERY>`: Custom query for the summary
- `--max-age <HOURS>`: Max cache age in hours (0=always livecrawl, -1=cache only)
- `--subpages <COUNT>`: Number of subpages to crawl per URL
- `--subpage-target <KEYWORDS>`: Comma-separated keywords to prioritize subpage selection

### Find Similar

Find pages similar to a reference URL:

    exa find-similar "https://arxiv.org/abs/2307.06435" --num 5
    exa find-similar "https://example.com/blog" --highlights --exclude-domains "example.com"

Options:
- `--num <COUNT>`: Number of results (1-100, default 10)
- `--include-domains <DOMAINS>`: Comma-separated domain whitelist
- `--exclude-domains <DOMAINS>`: Comma-separated domain blacklist
- `--start-date <DATE>`: Minimum published date (ISO 8601)
- `--end-date <DATE>`: Maximum published date (ISO 8601)
- `--highlights`: Return query-relevant excerpts
- `--highlights-chars <COUNT>`: Max characters for highlights
- `--text`: Return full page text as markdown
- `--text-chars <COUNT>`: Max characters for text
- `--summary`: Return LLM-generated summary

### Answer

Get an LLM-generated answer with citations:

    exa answer "What is the latest valuation of SpaceX?"
    exa answer "How does transformer attention work?" --text

Options:
- `--text`: Include full text content in citations

## Output Format & Error Codes

Output is YAML on stdout by default; pass `--json` to receive structured JSON.
Errors are printed to stderr as an envelope with a stable exit code:

    0  ok
    1  error          unclassified - report it and stop
    2  network        retry once, then stop
    3  auth_required  stop; give the human the `remediation` string verbatim
    4  not_found      the resource does not exist; do not retry
    5  rate_limited   back off before trying again
    6  invalid_input  fix the call arguments
    7  no_account     run `exa accounts list` or set EXA_API_KEY
"#;
