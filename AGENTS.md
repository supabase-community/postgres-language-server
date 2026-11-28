# AGENTS.md

Postgres Language Server: a Rust workspace (`crates/pgls_*`) with an LSP server, a CLI (`postgres-language-server`, legacy name `postgrestools`) and TypeScript packages in `packages/`. It lints, type-checks, formats and completes SQL against a snapshot of a Postgres database.

Read these instead of guessing:

- [CONTRIBUTING.md](./CONTRIBUTING.md): setup, tests, common tasks and the engineering rules.
- [ARCHITECTURE.md](./ARCHITECTURE.md): what each crate does and how a file flows through them.
- [crates/pgls_analyser/CONTRIBUTING.md](./crates/pgls_analyser/CONTRIBUTING.md): how a lint rule is written.

Skills for repeated workflows live in `.agents/skills/` (`.claude/skills` links there): `write-lint-rule`, `port-lint-rule`.

## Running commands

- In a fresh checkout or worktree, run `nix develop -c just setup` first. It checks out the `libpg_query` submodule and installs the JS dependencies; without it `just format` fails with `Script not found "oxfmt"`.
- Building `pgls_query` needs libclang, and codegen needs nightly `rustfmt`, `taplo` and `bun`. The Nix flake provides them; outside it, builds fail with `Unable to find libclang`. In a non-interactive shell, wrap each command: `nix develop -c cargo test -p pgls_analyser`, or `nix develop -c bash -c '...'` for several. The first `nix develop` takes minutes.
- Database tests need `just db` (`DATABASE_URL` in `.env`). All worktrees share that one container; without it, tests fail at once with `test database not reachable at <url>`.
- Build and test one crate at a time (`cargo test -p <crate>`); the workspace suite is slow. CI runs `cargo nextest run --workspace --profile ci` (see `.config/nextest.toml`) and the doctests separately.
- A worktree's `target/` grows to about 6 GB; `just clean` removes it. With several worktrees building at once, `CARGO_INCREMENTAL=0` saves about 2 GB each. Never share one `CARGO_TARGET_DIR` between worktrees: the workspace crates overwrite each other. `RUSTC_WRAPPER=sccache` (in the Nix shell) caches crates.io dependencies across worktrees, and the workspace crates only with `CARGO_INCREMENTAL=0`.
- The WASM e2e tests (`bun run test:e2e` in `packages/@postgres-language-server/wasm`) serve on port 3000; set `PGLS_E2E_PORT` if it's taken.

## Commands

```bash
just check                        # what CI's Rust jobs gate on: format check, clippy -D warnings, lint-features, clean codegen
just format                       # Rust, TOML and JS/TS
just gen-lint                     # after changing rules, configuration or the workspace API
cargo run -p pgls_cli -- check file.sql
just tree-print file.sql          # tree-sitter tree, for completions and hover
just record-regress <major>       # re-record the Postgres regression fixtures (Docker); --catalog-only after changing snapshot queries
just corpus                       # CLI and fuzz oracles over real-world SQL (crates/pgls_cli/tests/corpus.toml)
just fuzz <target> [seconds]      # libFuzzer targets in fuzz/: splitter_vs_parser, formatter_roundtrip, lexer_vs_scan, lsp_messages
just lint-actions                 # actionlint over .github/workflows
```

## Rules

- Follow the [engineering rules](./CONTRIBUTING.md#engineering-rules): no recursion whose depth grows with the input; a named limit (`*_max`, unit last) on every loop, walk, queue and cache over external input, logged when hit; no `unwrap`/`expect`/indexing on data from the client, database, file system or config; paired `debug_assert!`s where ranges cross a boundary; coalesce external events; set stack sizes, timeouts, pool sizes and recursion limits explicitly; state the cost and a 20k-statement timing of work on the request path; turn "Not covered" lists into issues.
- Convert offsets and positions with `pgls_line_index`, never by hand.
- Write database tests as `#[pgls_test_utils::db_test] async fn name(test_db: PgPool)`, not `#[sqlx::test]`: it fails fast without a database and caps the pool, so tests don't time out when worktrees share the database.
- Type rules (`pgls_catalog::typing`, `pgls_resolve`) port Postgres' parser. Link the Postgres function next to each port at the tag of the newest fixtures (e.g. `https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_coerce.c#L557`), and only report what Postgres certainly rejects; anything not modelled is unknown. `crates/pgls_analyser/tests/postgres_regress.rs` fails on any finding on a statement Postgres accepted.
- Snapshot tests use `insta`: `INSTA_UPDATE=always cargo test -p <crate>` writes the new snapshots; read their diff before committing.
- Turn a fuzz crash (`fuzz/artifacts/<target>/`) into a test of the crate it's in before fixing it.
- Generated files are committed. Run `just gen-lint` after changing rules or configuration; CI fails if it changes anything. After changing a query in `crates/pgls_catalog/src/snapshot/queries/`, run `cargo sqlx prepare --workspace` against the test database.
- Fixtures under `crates/pgls_postgres_regress/data/<major>/` are recorded, never edited by hand. A splitter fix needs no re-record: `crates/pgls_postgres_regress/tests/splitter.rs` snapshots where the splitter differs from them.
- Commits and PR titles follow [Conventional Commits](https://www.conventionalcommits.org), e.g. `fix(completions): ...`.
- Never add "Claude" or any other AI assistant's name to commit messages or pull request descriptions.

## Code Generation

Parse node code is generated by `pgls_query_macros` from the `libpg_query` submodule. To upgrade libpg_query, check out the new tag in `crates/pgls_query/vendor/libpg_query` and commit the submodule: `pgls_query_macros` and `pgls_pretty_print_codegen` read `protobuf/pg_query.proto` from it, and `pgls_tokenizer`'s build script reads `src/postgres/include/parser/kwlist.h`. `crates/pgls_query/src/protobuf.rs` is only regenerated when `protoc` is on the `PATH`, which the Nix shell doesn't provide: prepend the `bin` of `nix build --print-out-paths nixpkgs#protobuf` to `PATH` for one `cargo build -p pgls_query`.

Builds download nothing. The database lints are vendored at the upstream commit in their `vendor/COMMIT_SHA.txt`:

- pglinter's `sql/rules.sql` is copied as is into `crates/pgls_pglinter/vendor/sql/`. To upgrade, download `https://raw.githubusercontent.com/pmpetit/pglinter/<sha>/sql/rules.sql` into it, write `<sha>` to `COMMIT_SHA.txt`, then run `just gen-lint`.
- splinter's lints are adapted by hand into `crates/pgls_splinter/vendor/<category>/<name>.sql`: a `-- meta:` header instead of `create view`, `"name!"`-style column aliases for sqlx, no trailing `;`. To upgrade, adapt the changed files of `lints/` the same way, update `COMMIT_SHA.txt`, then run `just gen-lint`.
