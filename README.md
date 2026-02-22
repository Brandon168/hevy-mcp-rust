# hevy-mcp (Rust)

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

A **Model Context Protocol (MCP) server** written in pure Rust that interfaces
with the [Hevy fitness tracking app](https://www.hevyapp.com/) and its
[API](https://api.hevyapp.com/docs/). This server enables AI assistants (Claude,
Cursor, etc.) to read and manage your workout data, routines, exercise
templates, and more — all through a single, low-overhead binary.

This Rust version idles at **~10MB** — a ~25× reduction from the typescript
version— with no runtime to install.

> **Requires a Hevy PRO subscription** to access the Hevy API.

## Features

- **Workout Management** — Fetch, create, and update workouts
- **Routine Management** — Access and manage workout routines and folders
- **Exercise Templates** — Browse available templates; create custom ones
- **Exercise History** — Query past sets for any exercise template
- **Webhook Subscriptions** — Create, view, and delete webhook subscriptions
- **Dual Transport** — Runs over `stdio` (default) or `streamable-http`
- **Zero runtime overhead** — ~10 MB idle RAM (vs. >256 MB for the Node.js
  version); single static binary, no Node/Python runtime required

## Quick Start

### 1. Install via Binary (Recommended)

If you are on macOS or Linux, you can install the latest pre-compiled binary
with one command:

```bash
curl -fsSL https://raw.githubusercontent.com/Brandon168/hevy-mcp-rust/master/install.sh | sh
```

Alternatively, download the latest binary for your platform from the
[Releases page](https://github.com/Brandon168/hevy-mcp-rust/releases).

### 2. Run via Cargo (from source)

```bash
git clone https://github.com/Brandon168/hevy-mcp-rust.git
cd hevy-mcp-rust
# Provide API key and run
HEVY_API_KEY=sk_live_... cargo run --release
```

## Prerequisites

- [Rust](https://rustup.rs/) 1.75 or higher (`rustup update stable`)
- A Hevy API key ([Hevy PRO subscription required](https://www.hevyapp.com/))

> **Platform support:** Runs on macOS, Linux, and Windows. The binary is fully
> cross-platform — `cargo build --release` produces a native executable on each
> platform (`hevy-mcp.exe` on Windows).

## Installation & Configuration

### API Key

Provide your Hevy API key via environment variable or CLI flag:

```bash
# Environment variable (recommended)
export HEVY_API_KEY=sk_live_your_key_here

# Or as a CLI flag
./hevy-mcp --hevy-api-key=sk_live_your_key_here
```

You can also place it in a `.env` file in the project root (loaded automatically
at startup):

```env
HEVY_API_KEY=sk_live_your_key_here
```

> **Never commit your `.env` file or API keys to source control.**

### Transport Mode

| Flag                          | Default    | Description                                                          |
| ----------------------------- | ---------- | -------------------------------------------------------------------- |
| `--transport stdio`           | ✅ default | JSON-RPC over stdin/stdout — works with Claude Desktop, Cursor, etc. |
| `--transport streamable-http` | —          | HTTP server on `--port` (default 3000) at `/mcp`                     |
| `--port <PORT>`               | `3000`     | Port for streamable-http mode                                        |

All flags can also be set via environment variables: `MCP_TRANSPORT`,
`MCP_PORT`.

## Integration with AI Clients

### Cursor (`~/.cursor/mcp.json`)

```json
{
  "mcpServers": {
    "hevy-mcp": {
      "command": "/path/to/hevy-mcp",
      "env": {
        "HEVY_API_KEY": "sk_live_your_key_here"
      }
    }
  }
}
```

Or, if you prefer to run directly from the repo with `cargo run`:

```json
{
  "mcpServers": {
    "hevy-mcp": {
      "command": "cargo",
      "args": [
        "run",
        "--release",
        "--manifest-path",
        "/path/to/hevy-mcp-rust/Cargo.toml",
        "--"
      ],
      "env": {
        "HEVY_API_KEY": "sk_live_your_key_here"
      }
    }
  }
}
```

### Claude Desktop (`~/Library/Application Support/Claude/claude_desktop_config.json`)

```json
{
  "mcpServers": {
    "hevy-mcp": {
      "command": "/path/to/hevy-mcp",
      "env": {
        "HEVY_API_KEY": "sk_live_your_key_here"
      }
    }
  }
}
```

## Available MCP Tools

### Workout Tools

| Tool                 | Description                                         |
| -------------------- | --------------------------------------------------- |
| `get-workouts`       | Paginated list of workouts (newest first)           |
| `get-workout`        | Single workout by ID                                |
| `get-workout-count`  | Total number of workouts on account                 |
| `get-workout-events` | Paginated workout update/delete events since a date |
| `create-workout`     | Log a new workout with exercises and sets           |
| `update-workout`     | Modify an existing workout                          |

### Routine Tools

| Tool             | Description                  |
| ---------------- | ---------------------------- |
| `get-routines`   | Paginated list of routines   |
| `get-routine`    | Single routine by ID         |
| `create-routine` | Create a new workout routine |
| `update-routine` | Update an existing routine   |

### Routine Folder Tools

| Tool                    | Description                       |
| ----------------------- | --------------------------------- |
| `get-routine-folders`   | Paginated list of routine folders |
| `get-routine-folder`    | Single folder by ID               |
| `create-routine-folder` | Create a new routine folder       |

### Exercise Template Tools

| Tool                       | Description                                         |
| -------------------------- | --------------------------------------------------- |
| `get-exercise-templates`   | Paginated list of exercise templates                |
| `get-exercise-template`    | Single template by ID                               |
| `get-exercise-history`     | Past sets for a template (with optional date range) |
| `create-exercise-template` | Create a custom exercise template                   |

### Webhook Tools

| Tool                          | Description                         |
| ----------------------------- | ----------------------------------- |
| `get-webhook-subscription`    | View current webhook subscription   |
| `create-webhook-subscription` | Register a new webhook URL          |
| `delete-webhook-subscription` | Remove current webhook subscription |

## Development

### Project Structure

```
hevy-mcp-rust/
├── Cargo.toml              # Package manifest and dependencies
├── .env                    # Local API key (not committed)
├── src/
│   ├── main.rs             # CLI parsing, transport selection, server startup
│   ├── client.rs           # HevyClient — typed reqwest wrapper for the Hevy REST API
│   ├── types.rs            # Serde + JsonSchema typed structs (Workout, Routine, etc.)
│   └── tools.rs            # HevyTools — all 20 MCP tool implementations
├── tests/
│   └── integration_test.rs # Full end-to-end tests (spawns the binary as a child process)
└── openapi-spec.json       # Cleaned Hevy API specification
```

### Building

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Check for compile errors only (fastest)
cargo check
```

### Running Locally

```bash
# stdio mode (default) — connect from an MCP client
HEVY_API_KEY=your_key cargo run

# Streamable HTTP mode
HEVY_API_KEY=your_key cargo run -- --transport streamable-http --port 3000

# With verbose logging
RUST_LOG=debug HEVY_API_KEY=your_key cargo run
```

### Testing

```bash
# Run all tests (unit + integration)
# NOTE: Integration tests spawn the compiled binary and connect to it via MCP.
# They use a fake "test" API key so no real API calls are made.
cargo test

# Run only unit tests (fast, no binary spawn)
cargo test --lib

# Run only integration tests
cargo test --test integration_test

# Run with output visible
cargo test -- --nocapture
```

#### About the Integration Tests

`tests/integration_test.rs` contains two integration tests that **build and
spawn the actual binary**:

1. **`test_full_mcp_client`** — Spawns `hevy-mcp` in stdio mode, connects a real
   MCP client, and asserts that all 20 tools are returned by `list_tools`.
2. **`test_streamable_http_startup`** — Spawns `hevy-mcp` in streamable-http
   mode on a random free port, waits for the TCP port to become reachable, and
   sends a test HTTP request to `/mcp`.

`src/tools.rs` also contains inline `#[cfg(test)]` unit tests:

1. **`test_tools_list_count`** — Verifies 20 tools are registered without
   spawning a process.
2. **`test_tool_schemas`** — Verifies that each tool's JSON Schema has the
   expected properties and enum constraints (e.g., `create-exercise-template`
   correctly exposes `exercise_type` as an enum).

### Linting

```bash
# Run Clippy (Rust linter)
cargo clippy

# Apply auto-fixable suggestions
cargo clippy --fix
```

### Logging

Structured logs are emitted to **stderr** (stdout is reserved for the stdio MCP
transport). Control verbosity via the `RUST_LOG` environment variable:

```bash
RUST_LOG=info   # default level
RUST_LOG=debug  # verbose (shows HTTP requests)
RUST_LOG=warn   # quiet
```

## Architecture

Key technology choices:

- **[rmcp](https://github.com/modelcontextprotocol/rust-sdk)** — Official Rust
  MCP SDK; tools are declared with `#[tool]` / `#[tool_router]` proc macros
- **[tokio](https://tokio.rs/)** — Async runtime
- **[axum](https://github.com/tokio-rs/axum)** — HTTP server (streamable-http
  transport)
- **[reqwest](https://docs.rs/reqwest)** — Async HTTP client for Hevy API calls
- **[serde](https://serde.rs/) + [schemars](https://docs.rs/schemars)** — JSON
  serialization and automatic JSON Schema generation for tool parameters
- **[tracing](https://docs.rs/tracing)** — Structured, async-aware logging
- **[clap](https://docs.rs/clap)** — CLI argument parsing
- **[anyhow](https://docs.rs/anyhow) + [thiserror](https://docs.rs/thiserror)**
  — Error handling

## License

This project is licensed under the MIT License.

## Acknowledgements

- [Model Context Protocol](https://github.com/modelcontextprotocol) for the MCP
  SDK and specification
- [Hevy](https://www.hevyapp.com/) for their fitness tracking platform and API
- [chrisdoc/hevy-mcp](https://github.com/chrisdoc/hevy-mcp) — the original
  TypeScript implementation this was ported from
