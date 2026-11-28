---
name: write-lint-rule
description: Write or change a lint rule of pgls_analyser, from scaffolding to spec tests and codegen. Use when adding a rule, changing what a rule reports, or fixing a false positive. For a rule taken from Squawk, Eugene or pgfence, also load port-lint-rule.
---

# Write a lint rule

[`crates/pgls_analyser/CONTRIBUTING.md`](../../../crates/pgls_analyser/CONTRIBUTING.md) is the guide: rule names, the fields of `declare_lint_rule!`, the doc comment, spec files. Read the sections you need, then two or three rules in `crates/pgls_analyser/src/lint/<group>/` that match similar statements.

## Steps

1. Scaffold with `just new-lintrule <group> <ruleName> <info|warn|error>`. It runs all of `just gen-lint`; for several rules, run `cargo run -p xtask_codegen -- new-lintrule --category=lint --group=<group> --name=<ruleName> --severity=<severity>` for each, then `just gen-lint` once.
2. Implement `run`. Reuse what exists before writing a helper: AST matching shared by rules in `src/utils.rs`, token spans and fixes in `src/fix.rs`, names in `pgls_query_ext::utils` (`QualifiedName`), statement options in `pgls_query_ext::options`. The context has the statement (`ctx.stmt()`), the catalog and session before it (`ctx.catalog()`, `ctx.session()`), and its tokens (`ctx.tokens()`).
3. Document it in the doc comment. Every `sql,expect_diagnostic` snippet must produce exactly one diagnostic, or `docs_codegen` fails.
4. Add specs under `crates/pgls_analyser/tests/specs/<group>/<ruleName>/`: one `-- expect_lint/<ruleName>` line per expected diagnostic, or `-- expect_no_diagnostics`. Only the count per rule is checked. Cover the valid cases that look like the invalid ones.
5. Run `cargo test -p pgls_analyser --test rules_tests` (`INSTA_UPDATE=always` writes new snapshots; read the diff), then `just gen-lint`, and commit the generated files.

## Traps

- Keep `version: "next"`; the release workflow replaces it.
- The vendored libpg_query is Postgres 18 and parses syntax older servers reject. Where the server version matters, check `server_version_num()` of the catalog; specs set it with `-- server_version_num: 150000`, and `-- catalog: builtin` gives a rule outside `typecheck` the built-in catalog.
- The parser truncates identifiers to 63 bytes. For the text as written, read the tokens (`ctx.tokens()`, `pgls_query::scan`).
- `ctx.stmt().iter()` walks every nested node (CTEs, subqueries, `ON CONFLICT`, ...).
- Rules about locks, scans and rewrites skip tables created earlier in the same transaction: `ctx.is_new_relation(relation)`. Rules that only matter in migrations set `applies_to: pgls_analyse::AppliesTo::Migration`.
- `typecheck` rules and `banDuplicateColumnAssignments` run over Postgres' regression suite (`tests/postgres_regress.rs`); a finding on a statement Postgres accepted fails it. Only report what Postgres certainly rejects.
- To see a statement's AST, print `pgls_query::parse(sql)?.into_root()` with `{:#?}`, or edit and run the ignored `debug_test` in `src/lib.rs`.
