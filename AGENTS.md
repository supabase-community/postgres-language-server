# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Postgres Language Server implementation providing LSP features for SQL development, including autocompletion, syntax error highlighting, type-checking, and linting. The project is built in Rust using a modular crate architecture and includes TypeScript packages for editor integrations.

## Key Commands

### Development Setup
```bash
# Install development tools
just install-tools

# Start database for schema introspection
docker-compose up -d

# For Nix users
nix develop && docker-compose up -d
```

### Running Commands as an Agent
Building `pgls_query` needs libclang, and codegen needs nightly `rustfmt`, `taplo` and `bun`. Outside the Nix shell, builds fail with `Unable to find libclang`. The flake provides all of these and sets `LIBCLANG_PATH`. In a non-interactive shell, `nix develop` alone just opens a subshell, so wrap each command instead:

```bash
nix develop -c cargo build -p pgls_analyser
nix develop -c just gen-lint
nix develop -c bash -c 'cargo clippy --all-targets --all-features && cargo test -p pgls_analyser'
```

- The first `nix develop` downloads the toolchain and takes a few minutes; give it a generous timeout. Later runs start in seconds.
- `just format`, which `just gen-lint` ends with, runs `bun oxfmt`. In a fresh checkout or worktree, run `nix develop -c bun install --frozen-lockfile` first, otherwise it fails with `Script not found "oxfmt"` and leaves the generated TS files unformatted.
- Tests that use `sqlx::test` need the database from `docker-compose up -d` (see `DATABASE_URL` in `.env`).
- A worktree's `target/` takes about 6 GB after building every test. When several worktrees build at once, `CARGO_INCREMENTAL=0` saves about 2 GB more per worktree, at the cost of slower rebuilds. Don't point several worktrees at one `CARGO_TARGET_DIR`: the artifacts of the workspace crates have the same names in every worktree, so the builds overwrite each other.

### Building and Testing
```bash
# Run all tests
just test
# or: cargo test run --no-fail-fast

# Test specific crate
just test-crate pgls_lsp

# Run doc tests
just test-doc
```

### Code Quality
```bash
# Format all code (Rust, TOML, JS/TS)
just format

# Lint entire codebase
just lint
# or: cargo clippy && cargo run -p rules_check && bun oxlint packages/

# Fix linting issues
just lint-fix

# Full ready check (run before committing)
just ready
```

### Code Generation
```bash
# Generate linter code and configuration
just gen-lint

# Create new lint rule
just new-lintrule <group> <rulename> [severity]

# Create new crate
just new-crate <name>
```

### CLI Usage
The main CLI binary is `postgres-language-server` (legacy name: `postgrestools`):
```bash
cargo run -p pgls_cli -- check file.sql
# or after building:
./target/release/postgres-language-server check file.sql
```

## Architecture

### Crate Structure
The project uses a modular Rust workspace with crates prefixed with `pgls_`:

**Core Infrastructure:**
- `pgls_workspace` - Main API and workspace management
- `pgls_lsp` - Language Server Protocol implementation  
- `pgls_cli` - Command-line interface
- `pgls_fs` - Virtual file system abstraction
- `pgls_configuration` - Configuration management

**Parser and Language Processing:**
- `pgls_query` - Postgres query parsing (wraps libpg_query)
- `pgls_lexer` - SQL tokenizer with whitespace handling
- `pgls_statement_splitter` - Splits source into individual statements
- `pgls_treesitter` - Tree-sitter integration for additional parsing

**Features:**
- `pgls_completions` - Autocompletion engine
- `pgls_hover` - Hover information provider
- `pgls_analyser` & `pgls_analyse` - Linting and analysis framework
- `pgls_typecheck` - EXPLAIN fallback for typecheck
- `pgls_catalog` - Database snapshot, the file's DDL on top of it, name resolution, and type inference

**Utilities:**
- `pgls_diagnostics` - Error and warning reporting
- `pgls_console` - Terminal output and formatting
- `pgls_text_edit` - Text manipulation utilities
- `pgls_suppressions` - Rule suppression handling

### TypeScript Packages
Located in `packages/`:
- Backend JSON-RPC bridge in `packages/@postgres-language-server/backend-jsonrpc/` (legacy: `packages/@postgrestools/backend-jsonrpc/`)
- CLI package in `packages/@postgres-language-server/cli/`
- WASM package in `packages/@postgres-language-server/wasm/`
- Legacy main TypeScript package in `packages/@postgrestools/postgrestools/`

### Database Integration
The server connects to a Postgres database and loads a snapshot of its schema (`pgls_catalog::Snapshot`). The catalog applies the DDL of the current file on top of it, so linting, completions, and hover see objects created earlier in the file.

### Statement Processing Flow
1. Input source code is split into individual SQL statements
2. Each statement is parsed using libpg_query (via `pgls_query`)
3. Statements are analyzed against the catalog
4. Results are cached and updated incrementally on file changes

## Testing

### Test Data Location
- SQL test cases: `crates/pgls_statement_splitter/tests/data/`
- Analyzer test specs: `crates/pgls_analyser/tests/specs/`
- Example SQL files: `example/`, `test.sql`
- Postgres regression fixtures: `crates/pgls_postgres_regress/data/<major>/` (15–18): the upstream regression SQL, Postgres' verdict per statement, and the catalog of a fresh database. Re-record a version with `just record-regress <major>` (needs Docker), or only its catalog with `just record-regress <major> --catalog-only` after changing the snapshot queries.

### Type Checking
The type rules in `pgls_catalog::typing` and `pgls_catalog::resolve` port Postgres' parser from the latest supported version. Link the Postgres function next to each port, at the tag of the newest fixtures (e.g. `https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_coerce.c#L557`). Only report what Postgres would certainly reject; anything not modelled is unknown and never reported. `crates/pgls_analyser/tests/postgres_regress.rs` fails on any finding on a statement Postgres accepted, on every version.

### Snapshot Testing
The project uses `insta` for snapshot testing. Update snapshots with:
```bash
cargo insta review
# or accept all new and changed snapshots without the interactive review:
INSTA_UPDATE=always cargo test -p pgls_analyser --test rules_tests
```

### Lint Rules
`crates/pgls_analyser/CONTRIBUTING.md` explains how to write a rule. Things that are easy to miss:
- `just new-lintrule` runs the whole of `just gen-lint` for every rule. To scaffold several rules, run `cargo run -p xtask_codegen -- new-lintrule --category=lint --group=<group> --name=<ruleName> --severity=<info|warn|error>` for each, then `just gen-lint` once.
- New rules keep `version: "next"`; the release workflow replaces it with the released version.
- A spec file under `tests/specs/<group>/<ruleName>/` lists one `-- expect_lint/<ruleName>` line per expected diagnostic, or `-- expect_no_diagnostics`. Only the count per rule is checked, not which statement the line is above.
- Every `expect_diagnostic` snippet in a rule's doc comment must produce exactly one diagnostic. `docs_codegen`, which runs as part of `just gen-lint`, fails otherwise.
- The vendored libpg_query is Postgres 17, so syntax that is new in Postgres 18 has no AST nodes yet.
- The parser truncates identifiers to 63 bytes. For the source text as written, scan the statement with `pgls_query::scan`.
- `ctx.stmt().iter()` walks every nested node of a statement (CTEs, subqueries, `ON CONFLICT`, ...) as a `NodeRef`.
- To see the AST of a statement, print `pgls_query::parse(sql)?.into_root()` with `{:#?}`, e.g. from a throwaway test.

## Configuration Files

### Rust Configuration
- `Cargo.toml` - Workspace definition with all crate dependencies
- `rust-toolchain.toml` - Rust version specification  
- `rustfmt.toml` - Code formatting configuration
- `clippy.toml` - Clippy linting configuration

### Other Tools
- `.oxlintrc.json` / `.oxfmtrc.json` - oxlint linter and oxfmt formatter configuration for JS/TS
- `taplo.toml` - TOML formatting configuration
- `justfile` - Task runner with all development commands
- `docker-compose.yml` - Database setup for testing

## Development Notes

### Code Quality Guidelines
**IMPORTANT**: Always run `cargo clippy --all-targets --all-features` and fix all warnings after making code changes. Clippy warnings must be resolved before committing code to maintain code quality standards.

### Engineering Rules
**IMPORTANT**: Follow the "Engineering rules" in `CONTRIBUTING.md`. In short: no recursion whose depth grows with the input (per statement, file or config); a named limit (`*_max`, unit last) on every loop, walk, queue and cache over external input, logged when hit; no `unwrap`/`expect`/indexing on data from the client, database, file system or config; paired `debug_assert!`s where ranges cross a boundary (statement → file offsets → LSP positions, fix edits → document); coalesce external events; set stack sizes, timeouts, pool sizes and recursion limits explicitly; state the cost and a 20k-statement timing for work on the request path; turn "Not covered" lists into issues.

### Git Commit and PR Guidelines
**IMPORTANT**: NEVER add "Claude" or any AI assistant name to commit messages or pull request descriptions. Commits and PRs should appear as authored by the human developer only.

### Code Generation
Many parser structures are generated from PostgreSQL's protobuf definitions using procedural macros in `pgls_query_macros`. Run `just gen-lint` after modifying analyzer rules or configurations.

### Database Schema
The introspection queries live in `crates/pgls_catalog/src/snapshot/queries/`. After changing one, run `cargo sqlx prepare --workspace` against the test database to update `.sqlx/`.

### Code Refactoring Tools
The project has `ast-grep` available for advanced code search and refactoring tasks. ast-grep is a structural search/replace tool that understands code syntax, making it useful for:
- Renaming types, functions, or variables across the codebase
- Finding and replacing code patterns
- Performing structural code transformations

Example usage:
```bash
# Search for a pattern
ast-grep --pattern 'struct $NAME { $$$FIELDS }'

# Replace a pattern across files
ast-grep --pattern 'OldType' --rewrite 'NewType' --update-all
```

### Multi-Platform Support
The project includes platform-specific allocators and build configurations for Windows, macOS, and Linux.
- Seeing the Treesitter tree for an SQL query can be helpful to debug and implement features. To do this, create a file with an SQL query, and run `just tree-print <file.sql>`.