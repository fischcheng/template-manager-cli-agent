# Template Manager for CLI Agent (tm)

`tm` is a Cargo-based Rust CLI for scaffolding and maintaining agent workspace files.

It currently supports three agent targets:

- `claude`
- `codex`
- `gemini`

The CLI can:

- initialize a workspace for a specific agent
- optionally try a Spec-Kit bootstrap during initialization
- update managed files from the embedded or resolved manifest
- report workspace readiness with `doctor`

## Requirements

- Rust toolchain with Cargo
- optional: `uvx` if you want `--with-spec-kit`

## Install

Quick install from GitHub Releases:

```bash
curl -fsSL https://raw.githubusercontent.com/fischcheng/template-manager-cli-agent/main/install.sh | sh
```

You can override the repo slug or install location:

```bash
curl -fsSL https://raw.githubusercontent.com/fischcheng/template-manager-cli-agent/main/install.sh | TM_REPO=your-org/tm TM_INSTALL_DIR="$HOME/.local/bin" sh
```

To install a specific tagged version:

```bash
curl -fsSL https://raw.githubusercontent.com/fischcheng/template-manager-cli-agent/main/install.sh | TM_VERSION=v0.1.0 sh
```

## Build

```bash
cargo build
```

## Run

Show help:

```bash
cargo run -- --help
```

Initialize a Codex workspace using internal scaffolding only:

```bash
cargo run -- init codex --lite
```

Initialize a Claude workspace and try Spec-Kit first:

```bash
cargo run -- init claude --with-spec-kit
```

Refresh detected agent scaffolds in the current directory:

```bash
cargo run -- update
```

Check whether managed files are out of date without writing changes:

```bash
cargo run -- update --check
```

Inspect workspace and manifest status:

```bash
cargo run -- doctor
```

## Commands

### `init <agent>`

Creates the managed workspace files for one agent target.

Options:

- `--lite`: skip all external tool execution and scaffold using the internal manifest only
- `--with-spec-kit`: attempt Spec-Kit before applying `tm` normalization

### `update`

Detects existing agent workspaces in the current directory and reapplies managed content for those agents.

Options:

- `--manifest-path <path>`: use a manifest file directly
- `--check`: print pending changes and exit non-zero when updates are needed

### `doctor`

Prints a short report covering:

- whether `uvx` is available
- embedded vs cached manifest version
- readiness of each supported agent workspace

## Test

```bash
cargo test
```

## Release

Push a semver tag to trigger the GitHub Actions release workflow:

```bash
git tag v0.1.0
git push origin v0.1.0
```

The workflow builds release archives for:

- `x86_64-unknown-linux-gnu`
- `aarch64-apple-darwin`
- `x86_64-apple-darwin`
