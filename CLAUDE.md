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

### Building and Testing
```bash
# Run all tests
just test
# or: cargo test run --no-fail-fast

# Test specific crate
just test-crate pgt_lsp

# Run doc tests
just test-doc
```

### Code Quality
```bash
# Format all code (Rust, TOML, JS/TS)
just format

# Lint entire codebase
just lint
# or: cargo clippy && cargo run -p rules_check && bun biome lint

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
The main CLI binary is `postgres-language-server`:
```bash
cargo run -p pgt_cli -- check file.sql
# or after building:
./target/release/postgres-language-server check file.sql
```

## Architecture

### Crate Structure
The project uses a modular Rust workspace with crates prefixed with `pgt_`:

**Core Infrastructure:**
- `pgt_workspace` - Main API and workspace management
- `pgt_lsp` - Language Server Protocol implementation
- `pgt_cli` - Command-line interface
- `pgt_fs` - Virtual file system abstraction
- `pgt_configuration` - Configuration management

**Parser and Language Processing:**
- `pgt_query` - Postgres query parsing (wraps libpg_query)
- `pgt_lexer` - SQL tokenizer with whitespace handling
- `pgt_statement_splitter` - Splits source into individual statements
- `pgt_treesitter` - Tree-sitter integration for additional parsing

**Features:**
- `pgt_completions` - Autocompletion engine
- `pgt_hover` - Hover information provider
- `pgt_analyser` & `pgt_analyse` - Linting and analysis framework
- `pgt_typecheck` - Type checking via EXPLAIN
- `pgt_schema_cache` - In-memory database schema representation

**Utilities:**
- `pgt_diagnostics` - Error and warning reporting
- `pgt_console` - Terminal output and formatting
- `pgt_text_edit` - Text manipulation utilities
- `pgt_suppressions` - Rule suppression handling

### TypeScript Packages
Located in `packages/` and `editors/`:
- VSCode extension in `editors/code/`
- Backend JSON-RPC bridge in `packages/@postgres-language-server/backend-jsonrpc/`
- Main TypeScript package in `packages/@postgres-language-server/postgres-language-server/`

### Database Integration
The server connects to a Postgres database to build an in-memory schema cache containing tables, columns, functions, and type information. This enables accurate autocompletion and type checking.

### Statement Processing Flow
1. Input source code is split into individual SQL statements
2. Each statement is parsed using libpg_query (via `pgt_query`)
3. Statements are analyzed against the schema cache
4. Results are cached and updated incrementally on file changes

## Testing

### Test Data Location
- SQL test cases: `crates/pgt_statement_splitter/tests/data/`
- Analyzer test specs: `crates/pgt_analyser/tests/specs/`
- Example SQL files: `example/`, `test.sql`

### Snapshot Testing
The project uses `insta` for snapshot testing. Update snapshots with:
```bash
cargo insta review
```

## Configuration Files

### Rust Configuration
- `Cargo.toml` - Workspace definition with all crate dependencies
- `rust-toolchain.toml` - Rust version specification
- `rustfmt.toml` - Code formatting configuration
- `clippy.toml` - Clippy linting configuration

### Other Tools
- `biome.jsonc` - Biome formatter/linter configuration for JS/TS
- `taplo.toml` - TOML formatting configuration
- `justfile` - Task runner with all development commands
- `docker-compose.yml` - Database setup for testing

## Development Notes

### Code Generation
Many parser structures are generated from PostgreSQL's protobuf definitions using procedural macros in `pgt_query_macros`. Run `just gen-lint` after modifying analyzer rules or configurations.

### Database Schema
The `pgt_schema_cache` crate contains SQL queries in `src/queries/` that introspect the database schema to build the in-memory cache.

### Multi-Platform Support
The project includes platform-specific allocators and build configurations for Windows, macOS, and Linux.
- Seeing the Treesitter tree for an SQL query can be helpful to debug and implement features. To do this, create a file with an SQL query, and run `just tree-print <file.sql>`.
