---
title: "Authentication Guide"
description: "Authentication methods, credential storage, and error handling for developers and AI agents using the Exa CLI."
author: "SpaceCorps"
date: "2026-09-24"
---

# Authentication Guide for Exa CLI

This document outlines authentication methods, credential storage, and error handling for developers and AI agents using the Exa CLI.

## Overview
The Exa CLI interfaces directly with the Exa AI search REST API (`https://api.exa.ai`). Authentication requires an API key issued through your Exa dashboard. Keys can be stored in the host operating system's native keystore or supplied directly via environment variables and standard input.

## Prerequisites
- An Exa account ([exa.ai](https://exa.ai))
- An API key generated from the dashboard (`https://dashboard.exa.ai/api-keys`)
- Exa CLI installed (`cargo install --git https://github.com/SpaceCorps/Exa-Cli --locked`)

## Authentication Methods

### 1. Interactive Browser Login (`exa login`)
The recommended flow for local developer machines:
```bash
exa login [account_name]
```
1. The CLI launches your system browser to `https://dashboard.exa.ai/api-keys`.
2. You copy or generate an API key.
3. Paste the key into the CLI prompt (input characters are masked).
4. The CLI validates the key with a test search query to `POST /search`.
5. Upon confirmation, the key is securely saved to the native OS keyring under the account name (defaults to `default`).

### 2. Scripted & Headless Pipelines
For headless CI/CD environments, Docker containers, or autonomous agent runners:
```bash
printf %s "$EXA_API_KEY" | exa accounts add ci --api-key-stdin
```
Or pass the token directly as a CLI flag:
```bash
exa accounts add ci --api-key "$EXA_API_KEY"
```

### 3. Environment Variable
When no keychain account is specified, the CLI automatically checks for:
- `EXA_API_KEY`: API key used if no `--account` or `--api-key` is explicitly supplied.

### 4. Direct Command Override
Every API command supports `--api-key`:
```bash
exa search "frontier models" --api-key "$EXA_API_KEY"
```

## Multi-Account Management
Switch or verify accounts using:
```bash
exa accounts list --check
exa accounts test [account_name]
exa accounts remove [account_name] --yes
```

## Error Handling
When authentication fails, commands exit with non-zero exit codes and output standardized JSON error payloads:
- `auth_required` (exit code 3): No key provided or key rejected by upstream API.
- `no_account` (exit code 7): Specified account does not exist in keystore.
- `rate_limited` (exit code 5): Exa API rate limits reached.

## Security Best Practices
1. **Never Commit Keys**: Keep `.env` or plaintext key files out of git repositories.
2. **Use Native OS Keystore**: The CLI automatically utilizes macOS Keychain, Windows DPAPI, or Linux Secret Service (`secret-tool`).
3. **Machine Verification**: When writing agent automation scripts, always pass `--json` to reliably capture machine-readable error codes.
