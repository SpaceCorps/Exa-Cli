# Exa CLI (`exa`)

[![CI](https://github.com/SpaceCorps/Exa-Cli/actions/workflows/ci.yml/badge.svg)](https://github.com/SpaceCorps/Exa-Cli/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/release/SpaceCorps/Exa-Cli?color=brightgreen)](https://github.com/SpaceCorps/Exa-Cli/releases)

A blazing-fast native command-line tool and agent interface for the [Exa](https://exa.ai) AI search API, written in Rust. Features YAML-first output optimized for terminal and LLM readability, raw JSON via `--json`, secure OS keystore credential storage, and multi-account support.

Ported from the original C# prototype [`Exa.Console`](https://github.com/nielsbosma/Exa.Console) by Niels Bosma to high-performance native Rust under the [SpaceCorps](https://github.com/SpaceCorps) organization.

---

## Features

- ⚡ **Native Speed & Zero Dependencies:** Standalone static binary with 1–3 ms cold start times.
- 🔐 **OS Keystore Integration:** API keys are stored in macOS Keychain, Windows DPAPI, or Linux Secret Service (`secret-tool`).
- 🤖 **Agentic Protocol:** Clean YAML on stdout by default; structured JSON with `--json`; deterministic error envelopes on stderr with stable exit codes.
- 👥 **Multi-Account Safety:** Configure multiple named accounts (`exa accounts add work`) and target them explicitly with `--account work` (`-a work`).
- 📖 **Self-Documenting:** Embedded `exa agent-readme [--json]` delivers complete machine-readable rules and documentation directly to LLM agents.

---

## Installation

### Precompiled Binaries
Download prebuilt standalone binaries for macOS, Linux, and Windows from [GitHub Releases](https://github.com/SpaceCorps/Exa-Cli/releases/latest).

### From Source via Cargo
```bash
cargo install --git https://github.com/SpaceCorps/Exa-Cli --locked
```

---

## Authentication

You can authenticate in three ways:

### 1. Interactive Login (Recommended)
```bash
exa login [account_name]
```
Opens your browser to `https://dashboard.exa.ai/api-keys`, securely prompts for your key without terminal echo, verifies it against the API, and stores it in your OS keystore.

### 2. Multi-Account Management
```bash
# Add a named account (prompted securely)
exa accounts add work

# Or pass via stdin in CI/CD pipelines
printf %s "$EXA_API_KEY" | exa accounts add ci --api-key-stdin

# List and verify stored accounts
exa accounts list --check

# Test account connectivity
exa accounts test work
```

### 3. Environment Variable or Flag
Set `EXA_API_KEY` in your environment or pass `--api-key <key>` on any command:
```bash
export EXA_API_KEY="your-api-key"
exa search "frontier AI models comparison"
```

---

## Commands

### `search`
Search the web with natural language queries.

```bash
# Basic natural language search
exa search "latest developments in quantum computing"

# Filter by category and extract query-relevant highlights
exa search "senior ML engineers at fintech companies" --category people --highlights

# Filter by news and date range
exa search "AI regulation updates" --category news --start-date 2025-01-01 --num 20

# Filter by domain whitelist and return full page text
exa search "react state management" --include-domains "github.com,dev.to" --text

# Deep search with LLM summary
exa search "frontier AI models comparison" --type deep --summary
```

**Options:**
- `--num <COUNT>`: Number of results (1–100, default 10)
- `--type <TYPE>`: Search type (`auto`, `fast`, `instant`, `deep-lite`, `deep`, `deep-reasoning`)
- `--category <CATEGORY>`: Filter category (`company`, `people`, `news`, `research paper`, `personal site`, `financial report`)
- `--include-domains <DOMAINS>`: Comma-separated domain whitelist
- `--exclude-domains <DOMAINS>`: Comma-separated domain blacklist
- `--start-date <DATE>`: Minimum published date (ISO 8601)
- `--end-date <DATE>`: Maximum published date (ISO 8601)
- `--highlights`: Return query-relevant excerpts
- `--highlights-chars <COUNT>`: Max characters for highlights (default 4000)
- `--text`: Return full page text as markdown
- `--text-chars <COUNT>`: Max characters for text
- `--summary`: Return LLM-generated summary
- `--user-location <CODE>`: Two-letter ISO country code (e.g. `US`)

### `contents`
Extract clean, LLM-ready markdown content from one or more URLs.

```bash
# Extract full markdown text from a paper
exa contents "https://arxiv.org/abs/2301.07041" --text

# Extract query-relevant highlights
exa contents "https://example.com/article" --highlights --highlights-query "methodology"

# Extract summary and crawl subpages
exa contents "https://stripe.com" --summary --subpages 5 --subpage-target "about,careers,press"
```

*Note: At least one content mode (`--text`, `--highlights`, or `--summary`) is required.*

### `answer`
Get an LLM-generated answer grounded in real-time web citations.

```bash
# Ask a direct question
exa answer "What is the latest valuation of SpaceX?"

# Include full text in citation sources
exa answer "How does transformer attention work?" --text
```

### `find-similar`
Find web pages similar to a reference URL.

```bash
# Find papers similar to an arXiv submission
exa find-similar "https://arxiv.org/abs/2307.06435" --num 5

# Find similar blogs, excluding the original domain
exa find-similar "https://example.com/blog" --highlights --exclude-domains "example.com"
```

### `agent-readme`
Outputs the embedded operating manual for AI agents driving this CLI:

```bash
# Print markdown manual
exa agent-readme

# Print structured JSON specifications
exa agent-readme --json
```

---

## Agentic & Scripting Protocol

Pass `--json` to any command to receive raw JSON instead of YAML:

```bash
exa search "quantum computing" --json | jq '.results[].url'
```

### Structured Error Envelope
Errors are printed to `stderr` with machine-readable exit codes matching the `code` field:

```json
{
  "error": "The API key was rejected.",
  "code": "auth_required",
  "detail": "HTTP 401: Unauthorized",
  "remediation": "Set EXA_API_KEY, use --api-key <key>, or run: exa login"
}
```

| Exit Code | `code:` | Meaning | Agent Action |
| :--- | :--- | :--- | :--- |
| `0` | `ok` | Success | Continue |
| `1` | `error` | General unclassified error | Report error |
| `2` | `network` | Network timeout or 5xx error | Retry once with backoff |
| `3` | `auth_required` | Invalid or rejected API key | Surface remediation to user; do NOT retry |
| `4` | `not_found` | Resource not found | Do not retry |
| `5` | `rate_limited` | Upstream rate limit reached (HTTP 429) | Back off before retrying |
| `6` | `invalid_input` | Missing arguments or invalid parameters | Correct arguments |
| `7` | `no_account` | No account specified and no key available | Prompt user to configure account |

---

## Documentation

Full documentation, guides, and agent discovery manifests are available at:
**[https://spacecorps.github.io/Exa-Cli/](https://spacecorps.github.io/Exa-Cli/)**

- [Authentication Guide](https://spacecorps.github.io/Exa-Cli/auth.md)
- [Agent Manual (llms.txt)](https://spacecorps.github.io/Exa-Cli/llms.txt)
- [Full Agent Specification (llms-full.txt)](https://spacecorps.github.io/Exa-Cli/llms-full.txt)
- [A2A Agent Card](https://spacecorps.github.io/Exa-Cli/.well-known/agent-card.json)
- [AgentSkills Manifest](https://spacecorps.github.io/Exa-Cli/.well-known/agent-skills/index.json)

---

## License

MIT License. See [LICENSE](LICENSE) for details.
Original prototype created by [Niels Bosma](https://github.com/nielsbosma).
Maintained by [SpaceCorps](https://github.com/SpaceCorps).
