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
- `pgls_typecheck` - Type checking via EXPLAIN
- `pgls_schema_cache` - In-memory database schema representation

**Utilities:**
- `pgls_diagnostics` - Error and warning reporting
- `pgls_console` - Terminal output and formatting
- `pgls_text_edit` - Text manipulation utilities
- `pgls_suppressions` - Rule suppression handling

### TypeScript Packages
Located in `packages/` and `editors/`:
- VSCode extension in `editors/code/`
- Backend JSON-RPC bridge in `packages/@postgres-language-server/backend-jsonrpc/` (legacy: `packages/@postgrestools/backend-jsonrpc/`)
- Main TypeScript package in `packages/@postgres-language-server/postgres-language-server/` (legacy: `packages/@postgrestools/postgrestools/`)

### Database Integration
The server connects to a Postgres database to build an in-memory schema cache containing tables, columns, functions, and type information. This enables accurate autocompletion and type checking.

### Statement Processing Flow
1. Input source code is split into individual SQL statements
2. Each statement is parsed using libpg_query (via `pgls_query`)
3. Statements are analyzed against the schema cache
4. Results are cached and updated incrementally on file changes

## Testing

### Test Data Location
- SQL test cases: `crates/pgls_statement_splitter/tests/data/`
- Analyzer test specs: `crates/pgls_analyser/tests/specs/`
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
Many parser structures are generated from PostgreSQL's protobuf definitions using procedural macros in `pgls_query_macros`. Run `just gen-lint` after modifying analyzer rules or configurations.

### Database Schema
The `pgls_schema_cache` crate contains SQL queries in `src/queries/` that introspect the database schema to build the in-memory cache.

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