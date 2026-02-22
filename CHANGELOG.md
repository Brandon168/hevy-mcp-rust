# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2026-02-22

### Added

- Comprehensive integration test for the **Streamable HTTP (SSE)** transport,
  verifying the full handshake (initialize -> notify -> list tools).
- Detailed documentation in README for integrating with streamable-http clients
  like LobeChat and IDE plugins.

### Fixed

- **SSE Compatibility**: Disabled `sse_retry` priming events which were sending
  empty `data:` lines, causing "unexpected end of JSON input" errors in strict
  MCP clients.
- **SSE Stability**: Disabled `sse_keep_alive` by default to prevent empty data
  events from crashing IDE-based clients.
- **Naming Conventions**: Renamed internal types (e.g., `ax_req` to `AxReq`) to
  satisfy Rust non-camel-case linting warnings.

### Changed

- Refined session management configuration to favor protocol compatibility over
  default library pings.
- Improved integration test logic to handle the multi-step handshake required by
  the `rmcp` library.

### Removed

- Temporary diagnostic and validation scripts (`check_tools.py`, `dump_mcp.py`,
  `validate_streamable.py`) to clean up the project root.

## [0.1.0] - 2026-02-21

### Added

- Initial Rust port of the Hevy MCP server.
- Support for **stdio** and **streamable-http** transports.
- Implementation of 20 MCP tools covering Workouts, Routines, Exercises, and
  Webhooks.
- Automatic JSON Schema generation for all tool parameters using `schemars`.
- Integration test suite for binary execution and stdio transport.
- Binary installation script (`install.sh`) and multi-platform CI/CD via GitHub
  Actions.
