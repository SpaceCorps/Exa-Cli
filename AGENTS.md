# AGENTS.md

Notes for whoever extends this next.

`exa` is a native Rust CLI over the Exa AI search REST API, built to be driven by an LLM agent or human developer. It replaces the .NET global tool `Exa.Console` by Niels Bosma and keeps full interface compatibility: the same commands (`search`, `contents`, `find-similar`, `answer`), flags, options, and YAML-first output, while adding native multi-account management, OS keystores, deterministic error envelopes, and self-documenting agent capabilities.

For the manual the *agent* reads, run `exa agent-readme` (or `exa agent-readme --json`) - that text lives in `src/readme.rs` and is the tool's interface for its agentic audience. This file is for the human editing the source.

## Commands

```bash
cargo build --release              # target/release/exa
cargo test                         # unit tests + tests/cli.rs against an in-process mock API
cargo clippy --all-targets --locked -- -D warnings
cargo fmt --check
cargo install --path . --locked    # install to ~/.cargo/bin/exa
```

Use a throwaway config directory when testing so you never touch real credentials:

```bash
export EXA_CONFIG_DIR=$(mktemp -d) EXA_SECRET_STORE=plaintext
```

| Variable | Effect |
| --- | --- |
| `EXA_CONFIG_DIR` | Overrides the config and secrets storage directory |
| `EXA_SECRET_STORE` | Forces a keystore backend: `dpapi`, `keychain`, `libsecret`, `plaintext` |
| `EXA_ALLOW_PLAINTEXT_STORE=1` | Permits the plaintext fallback when no OS keystore is available |
| `EXA_API_URL` | Overrides the API base URL - how `tests/cli.rs` points at its in-process mock |
| `EXA_API_KEY` | Environment variable for direct API key authentication |

## Layout

```
src/
  main.rs          argument parsing, --json pre-scan, clap error formatting
  cli.rs           clap derive command hierarchy and help text
  commands/
    mod.rs         dispatcher
    search.rs      search command implementation
    contents.rs    contents extraction command
    find_similar.rs find-similar command
    answer.rs      answer generation command
    accounts.rs    accounts add|list|test|remove
    login.rs       interactive/scripted browser-assisted login
  client.rs        blocking HTTP (ureq + rustls), status -> ErrorCode mapping
  error.rs         ErrorCode (= exit code) and Error {code, message, detail, remediation}
  output.rs        YAML default (serde_norway), JSON (--json), error envelopes, obj! macro
  account.rs       multi-account resolution and API key management
  config.rs        config.yaml, paths, atomic file writes, 0600 permissions, cross-process lock
  secrets.rs       Keychain (/usr/bin/security), libsecret (secret-tool), DPAPI, plaintext fallback
  readme.rs        agent-readme text and embedded API rules
tests/
  cli.rs           drives the binary against an in-process mock TCP server
```

## Why it is built this way

**Blocking HTTP, no async runtime.** A CLI makes one to a few requests. Tokio would cost more in startup than it saves. Cold starts execute in 1–3 ms.

**The Keychain goes through `/usr/bin/security`, not the Security framework.** Reading through `/usr/bin/security` avoids code-signing prompt loops on rebuilt unsigned binaries.

**Responses stay `serde_json::Value`.** The upstream Exa API adds fields without notice, and this CLI prints whatever comes back. Typed structs would drop new fields or fail on new enum values.

**Multi-account safety:** Accounts are stored with unique names (`exa accounts add <name> --api-key <key>`) and referenced via `--account <name>` (`-a <name>`). Direct `--api-key <key>` and `EXA_API_KEY` are also supported for single-key workflows.

**Deterministic exit codes:** Exit codes map directly to the `error::ErrorCode` enum values (1..=7), matching the `code:` field in stderr error envelopes.

## Releasing

CI (`.github/workflows/ci.yml`) runs fmt, clippy and tests on Linux, macOS and Windows for every push and pull request. Publishing a GitHub Release builds pre-compiled binaries for:
- macOS Apple Silicon (`aarch64-apple-darwin`)
- macOS Intel (`x86_64-apple-darwin`)
- Linux x86_64 static musl (`x86_64-unknown-linux-musl`)
- Linux ARM64 static musl (`aarch64-unknown-linux-musl`)
- Windows x64 (`x86_64-pc-windows-msvc`)
