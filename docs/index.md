# Exa CLI (`exa`)

> A blazing fast native command-line tool and agent interface for the Exa AI search API, written in Rust.

Canonical URL: [https://spacecorps.github.io/Exa-Cli/](https://spacecorps.github.io/Exa-Cli/)

## Core Advantages

- **1–3 ms Startup:** A single static binary that executes in milliseconds, eliminating runtime overhead.
- **OS Keystore Security:** API keys are stored in macOS Keychain, Windows DPAPI, or Linux Secret Service (`secret-tool`).
- **Agentic Protocol:** Clean YAML on stdout by default; structured JSON with `--json`; deterministic error envelopes on stderr with stable exit codes.
- **Multi-Account Safety:** Manage separate work, personal, and CI accounts with explicit `--account` targeting.
- **Self-Documenting:** Embedded `exa agent-readme [--json]` delivers complete machine-readable rules directly to LLM agents.

## Quickstart

```bash
# 1. Install
cargo install --git https://github.com/SpaceCorps/Exa-Cli --locked

# 2. Login (interactively opens browser, prompts for key, verifies with API)
exa login

# 3. Search the web
exa search "latest developments in quantum computing"
```

## Commands

### `search`
Search the web with natural language queries:
```bash
exa search "frontier AI models comparison" --type deep --summary
exa search "senior ML engineers at fintech companies" --category people --highlights
exa search "react state management" --include-domains "github.com,dev.to" --text
```

### `contents`
Extract clean, LLM-ready markdown content from URLs:
```bash
exa contents "https://arxiv.org/abs/2301.07041" --text
exa contents "https://example.com/article" --highlights --highlights-query "methodology"
exa contents "https://stripe.com" --summary --subpages 5 --subpage-target "about,careers,press"
```

### `answer`
Get an LLM-generated answer grounded in real-time web citations:
```bash
exa answer "What is the latest valuation of SpaceX?"
exa answer "How does transformer attention work?" --text
```

### `find-similar`
Find web pages similar to a reference URL:
```bash
exa find-similar "https://arxiv.org/abs/2307.06435" --num 5
exa find-similar "https://example.com/blog" --highlights --exclude-domains "example.com"
```

### `accounts`
Manage multiple accounts and credentials:
```bash
exa login [name]
exa accounts add work
exa accounts list --check
exa accounts test work
exa accounts remove work --yes
```

### `agent-readme`
View embedded agent instructions and rules:
```bash
exa agent-readme
exa agent-readme --json
```

## Agentic Interface & Manifests

- [LLMs Overview (llms.txt)](https://spacecorps.github.io/Exa-Cli/llms.txt)
- [Full Agent Manual (llms-full.txt)](https://spacecorps.github.io/Exa-Cli/llms-full.txt)
- [A2A Agent Card](https://spacecorps.github.io/Exa-Cli/.well-known/agent-card.json)
- [AgentSkills Manifest](https://spacecorps.github.io/Exa-Cli/.well-known/agent-skills/index.json)
- [Authentication Guide](https://spacecorps.github.io/Exa-Cli/auth.md)
- [Pricing & Licensing](https://spacecorps.github.io/Exa-Cli/pricing.md)
