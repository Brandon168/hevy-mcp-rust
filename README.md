# hevy-mcp / hevy-cli (Rust)

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

Rust tools for the [Hevy fitness tracking app](https://www.hevyapp.com/) and its
[API](https://api.hevyapp.com/docs/):

- **`hevy-mcp`**: a Model Context Protocol server for MCP-capable clients such
  as Claude Desktop, Cursor, LobeChat, LibreChat, and IDE plugins.
- **`hevy-cli`**: a direct command-line client for scripts, shell automation,
  and agent skills that should run one command and exit.

Both binaries use the same typed Rust Hevy API client and require no Node or
Python runtime.

> **Requires a Hevy PRO subscription** to access the Hevy API.

## Which Should I Use?

| Use case | Use this | Why |
| --- | --- | --- |
| Your AI client supports MCP tools and can keep a tool server configured | `hevy-mcp` | Exposes Hevy operations as MCP tools over stdio or streamable HTTP |
| You want one-off shell commands, scripts, cron jobs, or CI automation | `hevy-cli` | Runs a single command, prints JSON, and exits |
| You are packaging an agent skill around Hevy access | `hevy-cli` + `skills/hevy` | Skills can invoke the binary directly without running an MCP server |
| You want HTTP/SSE access for a hosted or web client | `hevy-mcp --transport streamable-http` | Provides an MCP endpoint at `/mcp` |

## Features

- **Workout Management** — Fetch, create, and update workouts
- **Routine Management** — Access and manage workout routines and folders
- **Exercise Templates** — Browse available templates; create custom ones
- **Exercise History** — Query past sets for any exercise template
- **Webhook Subscriptions** — Create, view, and delete webhook subscriptions
- **Dual Transport** — Runs over `stdio` (default) or `streamable-http` (SSE)
- **Direct CLI** — `hevy-cli` provides JSON-first commands and guarded writes
  with `--confirm`
- **Agent Skill** — `skills/hevy` documents how agents should call `hevy-cli`

## Quick Start

### Install `hevy-mcp` via Binary

**macOS / Linux:**

```bash
curl -fsSL https://raw.githubusercontent.com/Brandon168/hevy-mcp-rust/master/install.sh | sh
```

**Windows (PowerShell):** Download the `hevy-mcp-windows-x86_64.zip` from the
[Releases page](https://github.com/Brandon168/hevy-mcp-rust/releases), extract
it, and place the `hevy-mcp.exe` in your PATH, or run it directly:

```powershell
.\hevy-mcp.exe --help
```

### Install `hevy-cli` via Release Asset

Download the matching `hevy-cli-*` archive from the
[Releases page](https://github.com/Brandon168/hevy-mcp-rust/releases), extract
it, and place `hevy-cli` on your `PATH`.

Example for Linux x86_64:

```bash
curl -L -o hevy-cli-linux-x86_64.tar.gz \
  https://github.com/Brandon168/hevy-mcp-rust/releases/latest/download/hevy-cli-linux-x86_64.tar.gz
tar -xzf hevy-cli-linux-x86_64.tar.gz
chmod +x hevy-cli
sudo mv hevy-cli /usr/local/bin/
hevy-cli --help
```

### Run from Source

```bash
git clone https://github.com/Brandon168/hevy-mcp-rust.git
cd hevy-mcp-rust
HEVY_API_KEY=sk_live_... cargo run --release --bin hevy-mcp
HEVY_API_KEY=sk_live_... cargo run --release --bin hevy-cli -- auth test
```

## Prerequisites

- **Hevy PRO Subscription** — Required to access the Hevy API.
- **Hevy API Key** — [Generate an API key](https://api.hevyapp.com/docs/) in
  your Hevy account.
- **Rust 1.75+** — Only required if **building from source** (pre-compiled
  binaries have no dependencies).

> **Platform support:** Runs on macOS, Linux, and Windows. The binary is fully
> cross-platform — `cargo build --release` produces a native executable on each
> platform (`hevy-mcp.exe` on Windows). Note: macOS and Linux are tested and
> work well. Windows and WSL should work but are currently untested; feedback is
> welcome!

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

### `hevy-cli`

Use `hevy-cli` for direct shell usage, automation, or agent skills. It does not
start an MCP server.

```bash
hevy-cli auth test
hevy-cli workouts list --page 1 --page-size 10
hevy-cli workouts get --id <workout_id>
hevy-cli export workouts --weeks 3 --full
hevy-cli export routine-bundle --routine-id <routine_id> --weeks 3
```

Write commands require `--confirm`:

```bash
hevy-cli routines create --input routine.json --confirm
hevy-cli webhooks delete --confirm
```

To install the local skill wrapper after building a release binary, pass the
destination expected by your agent runtime:

```bash
cargo build --release --bin hevy-cli
./scripts/install-hevy-skill.sh /path/to/agent/skills/hevy
```

### `hevy-mcp`

Use `hevy-mcp` when an MCP-capable client should discover and call Hevy tools.
It runs as a server over stdio by default or over streamable HTTP when requested.

### Transport Mode

| Flag                          | Default    | Description                                                          |
| ----------------------------- | ---------- | -------------------------------------------------------------------- |
| `--transport stdio`           | ✅ default | JSON-RPC over stdin/stdout — works with Claude Desktop, Cursor, etc. |
| `--transport streamable-http` | —          | HTTP server on `--port` (default 3000) at `/mcp`                     |
| `--port <PORT>`               | `3000`     | Port for streamable-http mode                                        |

All flags can also be set via environment variables: `MCP_TRANSPORT`,
`MCP_PORT`.

## Integration with AI Clients

### Cursor / Claude Desktop (Stdio)

Add to `~/.cursor/mcp.json` or `claude_desktop_config.json`:

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

### Streamable HTTP (SSE)

When using **hevy-mcp** with a client that supports the `streamable-http`
transport (such as **LobeChat**, **LibreChat**, or **IDE plugins**):

1. **Start the server**:
   ```bash
   hevy-mcp --transport streamable-http --port 3333
   ```
2. **Configure Client**:
   - **Endpoint URL**: `http://localhost:3333/mcp`
   - **Type**: `Streamable HTTP` (or `SSE`)

> **Note:** The server supports a full MCP over SSE implementation. The result
> of the `initialize` call is streamed via SSE, while subsequent requests use
> standard HTTP POST to the manifest endpoint.

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
├── skills/
│   └── hevy/               # Agent skill wrapper for hevy-cli
├── scripts/
│   └── install-hevy-skill.sh
├── src/
│   ├── main.rs             # hevy-mcp entry point
│   ├── lib.rs              # Library root (exports client, tools, types)
│   ├── client.rs           # HevyClient — typed REST API wrapper
│   ├── bin/
│   │   └── hevy-cli.rs     # Direct command-line client
│   ├── types.rs            # Serde + JsonSchema typed structs
│   └── tools.rs            # HevyTools — all 20 MCP tool implementations
├── tests/
│   ├── cli_test.rs         # hevy-cli command tests
│   ├── integration_test.rs # Full E2E tests (stdio & streamable-http)
│   ├── client_test.rs      # Unit tests for HevyClient using wiremock
│   └── deserialize_test.rs # Verification of API response parsing
└── openapi-spec.json       # Cleaned Hevy API specification
```

### Testing & Verification

We maintain a high bar for reliability. Before submitting changes, always run:

```bash
cargo test
```

- **Mock Testing**: We use `wiremock` in `client_test.rs` to verify HTTP
  interactions without hitting the live API.
- **Integration Handshakes**: `integration_test.rs` spawns the binary to verify
  the full MCP lifecycle over both Stdio and SSE.
- **Schema Validation**: Unit tests in `tools.rs` ensure JSON Schemas remain
  compatible with the reference implementation.

> **AI Developer Note**: When adding new tools or changing types, ensure you
> update the corresponding mock in `client_test.rs` and verify deserialization
> in `deserialize_test.rs`. Use `HEVY_BASE_URL` to point the client to your mock
> server.

## License

This project is licensed under the MIT License.

## Acknowledgements

- [Model Context Protocol](https://github.com/modelcontextprotocol) for the MCP
  SDK and specification
- [Hevy](https://www.hevyapp.com/) for their fitness tracking platform and API
- [chrisdoc/hevy-mcp](https://github.com/chrisdoc/hevy-mcp) — the original
  TypeScript implementation this was ported from
