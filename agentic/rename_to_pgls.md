This project is currently named `postgrestools`, with `pg-language-server.com` as its domain, and `pgt` in short. Our config file is `postgres-language-server.jsonc`.

We need to rename it to Postgres Language Server, with
- `pgls` as short
- `pg-language-server.com` as our domain and
- `pgls` as binary name
- `postgres-language-server.jsonc` as config name

I want you to go through the repo and rename everything, including all of the above and specifically:
- `PgLSPath` -> `PgLSPath`
- `pgt_` crate prefix -> `pgls_`
- `PGT_` env vars -> `PGLS_`
- `@postgrestools/postgrestools` should become `@pgls/cli`
- `@postgrestools/backend-jsonrpc` -> `@pgls/backend-jsonrpc`

## Full List

### 1. File and Directory Renamings
- All crate directories: `crates/pgt_*` → `crates/pgls_*` (34 crates)
- Package directories:
  - `packages/@postgrestools/postgrestools/` → `packages/@pgls/cli/`
  - `packages/@postgrestools/backend-jsonrpc/` → `packages/@pgls/backend-jsonrpc/`
- Binary file: `packages/@postgrestools/postgrestools/bin/pgls` → `packages/@pgls/cli/bin/pgls`

### 2. Code and Import Renamings
- All Rust crate imports: `use pgt_*` → `use pgls_*`
- All Cargo.toml package names: `name = "pgt_*"` → `name = "pgls_*"`
- All Cargo.toml dependencies: `pgt_*` → `pgls_*`
- Binary name in CLI: `pgls` → `pgls`

### 3. Additional Environment Variables
- `PGLS_CACHE_PATH` → `PGLS_CACHE_PATH`
- `PGLS_CLI_CACHE` → `PGLS_CLI_CACHE`
- `PGLS_DAEMON_HOST` → `PGLS_DAEMON_HOST`
- `PGLS_DAEMON_PORT` → `PGLS_DAEMON_PORT`
- `PGLS_COLORS` → `PGLS_COLORS`

### 4. Test and Snapshot Files
- All test snapshot files: `pgt_tokenizer__tests__*` → `pgls_tokenizer__tests__*`

### 5. Documentation and Comments
- All references to "pgls" in docs, READMEs, and comments
- All URLs from pg-language-server.com to pg-language-server.com

### 6. Lock and Build Files
- Cargo.lock will be updated automatically
- bun.lock, uv.lock, package-lock.json files will need regeneration

### Summary
This refactoring will affect approximately:
- 43 files containing "pgls"
- 9 files containing "pg-language-server.com"
- 426+ files containing "pgt_" prefix
- 12 files containing "PGT_" prefix
- 23 files containing "PgLSPath"
- 12 files containing "@postgrestools"
- 34 crate directories
- Multiple lock files and build artifacts

Please, go ahead and execute the renaming.
