# Contributing

Thanks for your interest in contributing! This guide covers the development workflow. For an overview of how the code base fits together, read [ARCHITECTURE.md](./ARCHITECTURE.md) first.

## Setup

You need Rust (the version is pinned in `rust-toolchain.toml`), [Bun](https://bun.sh), [just](https://github.com/casey/just) and Docker.

```bash
git clone --recurse-submodules https://github.com/supabase-community/postgres-language-server.git
cd postgres-language-server

# install the cargo and JS tooling (tree-sitter-cli, cargo-insta, taplo, sqlx-cli, uv, ...)
just install-tools

# start the Postgres database the tests and the sqlx queries run against
docker compose up -d
```

If you use Nix, `nix develop` provides all tools, including libclang, which the `libpg_query` bindings need. Run `git submodule update --init --recursive` if you cloned without `--recurse-submodules`.

The test database listens on `postgresql://postgres:postgres@127.0.0.1:5432/postgres` (see `.env`).

## Running and debugging

```bash
# run the CLI against a file
cargo run -p pgls_cli -- check file.sql

# print the tree-sitter tree of a file, useful for completions and hover
just tree-print file.sql
```

To try the language server in an editor, build it with `cargo build -p pgls_cli` and point your editor's LSP configuration at `target/debug/postgres-language-server lsp-proxy`. The [IDE guide](./docs/guides/ide_setup.md) shows how each editor launches the server.

## Tests

```bash
just test                   # all tests
just test-crate pgls_lsp    # a single crate
just test-doc               # doc tests
```

- Many tests are [insta](https://insta.rs) snapshot tests. Review changed snapshots with `cargo insta review` and commit the updated `.snap` files.
- Linter rules are tested with the SQL specs in `crates/pgls_analyser/tests/specs/<group>/<rule>/`.
- Statement splitter cases live in `crates/pgls_statement_splitter/tests/data/`.
- `crates/pgls_analyser/tests/postgres_regress.rs` runs the analyser over Postgres' own regression suite for versions 15–18 (fixtures in `crates/pgls_postgres_regress/data/<major>/`) and fails on any finding on a statement Postgres accepted. Re-record a version with `just record-regress <major>` (needs Docker).

## Common tasks

- **New lint rule:** `just new-lintrule <group> <ruleName> [severity]` scaffolds the rule, its spec folder and the generated configuration. Rule docs are generated from the rule's doc comment.
- **Codegen:** after changing rules, configuration or the workspace API, run `just gen-lint`. It regenerates the rule registry, configuration, JSON schema, TypeScript bindings and docs.
- **Snapshot queries:** the introspection queries live in `crates/pgls_catalog/src/snapshot/queries/`. After changing one, run `cargo sqlx prepare --workspace` against the test database and commit the `.sqlx/` changes.
- **Type checking:** the rules in `pgls_catalog::typing` and `pgls_catalog::resolve` port Postgres' parser. Link the Postgres function next to each port and only report what Postgres would certainly reject.
- **Docs:** the documentation site lives in `docs/`. Preview it with `just serve-docs`.

## Before opening a pull request

```bash
just format   # Rust, TOML and JS/TS
just lint     # clippy, rules_check and oxlint
just ready    # codegen, format and lint; fails if anything changes
```

`just ready` expects a clean working tree, so commit first. CI runs the same checks, plus tests on Linux, macOS and Windows.

Use [Conventional Commits](https://www.conventionalcommits.org) for commit messages and pull request titles, e.g. `fix(completions): ...` or `feat(analyser): ...`. Releases and the changelog are generated from them.

## Getting help

Open a [discussion](https://github.com/supabase-community/postgres-language-server/discussions) or comment on the issue you're working on.
