# AGENTS.md

Rust crate `hevy-mcp`: a library (`src/lib.rs`: `client`, `tools`, `types`) plus
two binaries, `hevy-mcp` (MCP server over stdio or streamable-http) and
`hevy-cli` (direct CLI) for the [Hevy](https://hevy.com) API. Conventions for
tests and adding a tool are in `.cursorrules`.

## Build and test

Requires a Rust toolchain and OpenSSL development headers (`libssl-dev` on
Debian/Ubuntu).

```sh
cargo build
cargo test                   # unit + wiremock + integration tests; never hits the live API
cargo fmt
cargo clippy -- -D warnings
```

## Verification workflow

Run the fast local checks (`cargo fmt`, `cargo test`) while iterating. Leave the
full build to GitHub Actions: after pushing, run `scripts/watch-ci.sh` (needs an
authenticated `gh`; run it as a background shell so you are notified). It polls
every 5 seconds until every workflow run for `HEAD` finishes and exits non-zero
if any fails.

## CI and releases

- **CI** (`.github/workflows/ci.yml`) runs on every push, pull request to
  `master`, and manual dispatch: formatting check, tests, and a Linux x86_64
  build. It uploads the artifact `hevy-linux-x86_64-<commit-sha>` containing
  archives for both binaries and `dist/SHA256SUMS`. CI does not run clippy.
- **Release** (`.github/workflows/release.yml`) runs only when a `v*` tag is
  pushed: it builds Linux, macOS and Windows binaries, runs the tests and
  publishes a GitHub release. Pushing to `master` does not release.
- To cut a release: bump `version` in `Cargo.toml` (and `Cargo.lock`), add a
  `CHANGELOG.md` entry, commit and push, wait for CI to pass, then push a
  `vX.Y.Z` tag. Removing a public item from the library crate is a breaking
  change and should be reflected in the version and changelog.
- Tag and publish releases only when asked.
