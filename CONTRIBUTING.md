# Contributing

Thanks for your interest in contributing! This guide covers the development workflow. For an overview of how the code base fits together, read [ARCHITECTURE.md](./ARCHITECTURE.md) first.

## Setup

You need Rust (the version is pinned in `rust-toolchain.toml`), [Bun](https://bun.sh), [just](https://github.com/casey/just) and Docker.

```bash
git clone --recurse-submodules https://github.com/supabase-community/postgres-language-server.git
cd postgres-language-server

# install the cargo and JS tooling (tree-sitter-cli, cargo-insta, taplo, sqlx-cli, uv, ...)
just install-tools

# check out the libpg_query submodule and install the JS dependencies; run this in every new worktree too
just setup

# start the Postgres database the tests and the sqlx queries run against, and wait until it's ready
just db
```

If you use Nix, `nix develop` provides all tools, including libclang, which the `libpg_query` bindings need.

The test database listens on `postgresql://postgres:postgres@127.0.0.1:5432/postgres` (see `.env`). All checkouts and worktrees share it. `just db` rebuilds the image when the `Dockerfile` changed; the Docker build cache makes that fast otherwise. Tests that need the database use `#[pgls_test_utils::db_test]`.

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
- **Type checking:** the rules in `pgls_catalog::typing` and `pgls_resolve` port Postgres' parser. Link the Postgres function next to each port and only report what Postgres would certainly reject.
- **Docs:** the documentation site lives in `docs/`. Preview it with `just serve-docs`.

## Engineering rules

The language server runs for hours on files we don't control: `pg_dump` output with 20,000 statements, non-ASCII identifiers, half-typed SQL. Most of the bugs we've fixed came from a few patterns, and these rules target them. They are adapted from TigerBeetle's [TigerStyle](https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/TIGER_STYLE.md).

- **No recursion whose depth grows with the input.** Recursion over the AST is fine because the parser bounds its depth; recursion per statement, file, config or list item is not, so write a loop. A statement iterator that recursed once per skipped statement overflowed the stack on a 2,500-statement file and aborted the server for every connected editor.
- **Put a limit on everything that grows with external input:** loops, walks, queues, caches and payloads. Keep the limit in a named constant with the unit last (`entries_max`, `depth_max`, `bytes_max`), add a sentence on why that number, and log when it's hit. Unbounded walks and caches show up as hangs and memory growth in large repositories, and a silent cap, like a fix loop that stops after ten passes, hides the bug instead.
- **No `unwrap`, `expect` or indexing on external data**, meaning anything from the client, the database, the file system or a config file. Return an error or skip the item; panics are for programmer errors. `params.arguments[0]` on a command sent without arguments and `.parse().expect(..)` on plpgsql_check output both panicked.
- **Assert ranges where they cross a boundary.** When a range goes from statement-relative to file offsets, from file offsets to LSP positions, or from a fix edit into the document, `debug_assert!` on both sides that it lies inside its text and on char boundaries. Most offset bugs (UTF-8 vs UTF-16, non-ASCII identifiers, offsets through `''` unescaping, edits outside their statement) were silently wrong ranges; tests and fuzzers run with debug assertions and turn them into failures.
- **Don't react directly to external events.** Coalesce file-watcher and change events, debounce work triggered by edits, and check the document version between units of work. A git checkout used to queue several full configuration reloads, and every keystroke re-walked the file once per feature.
- **Set explicitly the defaults that have bitten us:** thread stack sizes, timeouts, pool sizes and recursion limits, each with a comment on the value. The 2 MiB tokio worker stack is what turned the recursion above into an abort, and prost's default decode recursion limit of 100 rejects statements nested 40 levels deep.
- **State the cost of anything on the request path.** A PR that adds work per keystroke or per request says how it scales (O(statements before the cursor), O(range), ...) and how long it takes on a 20,000-statement file. A quadratic dedupe, a full walk per request and re-running the analyser for every code action all passed the tests on small files.
- **Turn "Not covered" into issues.** Keep the "Not covered" list in the PR description and open an issue for each item, so the gaps don't get lost once the PR is merged.

## API conventions

These describe the shape we're moving the code to. New code follows them, and existing code is brought in line crate by crate, so you will still find code that doesn't. They come from [oxc](https://github.com/oxc-project/oxc) and [biome](https://github.com/biomejs/biome) and from our own review of the code base.

### Structure

- **No standalone helper functions.** Every helper gets a home on the type it's about, as a method or an associated function: `RuleSpecifier::of(diagnostic)`, `select.leftmost()`, `catalog.end_transaction()`, `Lines::new(text)`. Never write a free function that a method only forwards to; fold it into the method. If the type is defined in another crate, add an inherent impl in the crate that owns the type when that crate is the right layer, and use an extension trait only when it isn't. A crate's established entry points (the printer's per-node `emit_*` functions, the splitter's `split`) stay until that crate's refactor, but new helpers inside them still go on a type.
- **Keep logic in the crate that owns it.** Call it from there instead of copying it into the crate that needs it. Line breaks, for example, are found by `pgls_line_index`, not by hand-written `\n`/`\r` searches in each crate.
- **A crate root is a table of contents.** Modules are private, and `lib.rs` has one explicit `pub use` list, without `pub use x::*`. Each item has one public path. A crate re-exports another crate only as a documented facade, like `pgls_diagnostics` does for `pgls_console`. Items that only benchmarks need sit behind a feature and `#[doc(hidden)]`. See [`oxc_semantic/src/lib.rs`](https://github.com/oxc-project/oxc/blob/6774a439d419fde946ddb45c620bcd2b0ed14572/crates/oxc_semantic/src/lib.rs#L30-L57).
- **Entry points take a params struct or a builder and return a named struct.** Recoverable problems are part of the returned value, not the error. A crate has one entry point per operation, and no test-only convenience entry points in its public API; tests use a test helper. See oxc's [`ParserReturn`](https://github.com/oxc-project/oxc/blob/6774a439d419fde946ddb45c620bcd2b0ed14572/crates/oxc_parser/src/lib.rs#L166).
- **Names.** `*Params` are the inputs of one call, `*Options` are reusable knobs, `*Configuration` is the configuration file (only in `pgls_configuration`) and `*Settings` is resolved workspace state (only in `pgls_workspace`). No `get_` prefix. `as_`/`to_`/`into_` follow their cost, as in the [API guidelines](https://rust-lang.github.io/api-guidelines/naming.html). The method names of the `Workspace` trait are wire names used by the TypeScript bindings, so they only change with the protocol.
- **No `bool` pairs or sentinel values in public signatures.** Use an enum or a struct with named fields. No `-1` or `0` for an unknown position, no `""` for "unset", and no `(Option, Option)` tuples with an `unreachable!()` case.
- **Unknown is an error value.** "We can't tell" is `Err(Unknown)`, so `?` propagates it. Lookups keep found, missing and unknown apart.
- **State that always travels together is one value.** A catalog and its search path are one value. A `TextRange` keeps one coordinate system for its whole life; converting it to another one makes a value of another type.
- **Errors.** What a user should see is a `Diagnostic`. An infrastructure failure is the crate's own `thiserror` enum, not named `Error`; don't pass our own failures off as a third-party error such as `sqlx::Error::Protocol`. No third-party error types in the public signatures of crates that don't own the dependency, and no `anyhow` in library crates (`pgls_lsp` may use it). Lower crates return plain error data, such as a parse error with its position, and the workspace turns it into diagnostics.
- **Frontends see workspace types only.** `pgls_lsp`, `pgls_wasm` and `pgls_cli` depend on the types of `pgls_workspace`, not on feature or analysis crates. Wire types live in the workspace, so a rename inside a feature crate can't change the protocol.
- **Proc macros only where a function or `macro_rules!` can't do the job.** Their crates end in `_macros` and use syn 2.
- **Every `lib.rs` starts with `//!` docs** that say what the crate does and doesn't do, and show its main call site. [ARCHITECTURE.md](./ARCHITECTURE.md) is the map and links to them.

### Performance

- **Data-oriented design.** Use flat arrays and indices instead of hash maps and pointers, and a struct of arrays where passes read different fields. The layout is what makes the request path fast on a 20,000-statement file.
- **No caches for costs we control.** When something we compute is slow, fix its layout or its algorithm instead of caching the result. A cache is fine only where the cost is outside our control: libpg_query's parse, database round trips and connection handshakes. Such a cache still has a limit, as the engineering rules ask.

### Build and generated code

- **Lints live in `[workspace.lints]`.** Each crate inherits them with `[lints] workspace = true`, and a crate that opts out of a lint says why. The set includes the engineering rules' panic lints, `allow_attributes_without_reason` and `unreachable_pub`. `cargo shear` catches unused dependencies.
- **Generated code has one input and one generator, is checked in, and is diffed in CI.** Upstream inputs are read from the libpg_query submodule only. Generators are `xtask` commands that write `src/generated/`, and `just check-codegen` re-runs them. Build scripts write only to `OUT_DIR` and never use the network. Grammar node kinds and fields are generated constants, never string literals. Don't edit generated files by hand.

### Tests

- Unit tests go inline, or in a sibling `foo.tests.rs` once they outgrow the module.
- Specs and fixtures go in `tests/`, with their insta snapshots next to them.
- Conformance suites, like the Postgres regression suite, are data: recorded verdicts and one runner that uses the production entry point. Their fixtures don't depend on the code under test.
- Database tests use `#[pgls_test_utils::db_test]`, not `#[sqlx::test]`.

### Pull requests

- **Keep refactors and behaviour changes apart.** A refactor changes no behaviour and no snapshot. A behaviour change comes with spec or server tests. When a refactor uncovers a bug, fix it in its own commit or PR.

## Before opening a pull request

```bash
just format   # Rust, TOML and JS/TS
just lint     # clippy, rules_check and oxlint
just check    # what CI's Rust jobs gate on: format check, clippy -D warnings, clean codegen
just ready    # codegen, format and lint; fails if anything changes
```

`just ready` expects a clean working tree, so commit first. CI runs the same checks, plus tests on Linux, macOS and Windows.

Use [Conventional Commits](https://www.conventionalcommits.org) for commit messages and pull request titles, e.g. `fix(completions): ...` or `feat(analyser): ...`. Releases and the changelog are generated from them.

## Getting help

Open a [discussion](https://github.com/supabase-community/postgres-language-server/discussions) or comment on the issue you're working on.
