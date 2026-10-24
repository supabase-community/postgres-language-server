# Linting Migrations

Postgres Language Tools comes with a `check` command that can be integrated into your development workflow to catch problematic schema changes and encourage best practices.

To run it, simply point at your migrations directory.

```sh
postgres-language-server check supabase/migrations
```

When you are setting it up in an existing project, you might want to ignore all migrations that are already applied. To do so, add `migrationsDir` and `after` to your `postgres-language-server.jsonc` file.

The `after` value is the numeric prefix extracted from your migration filenames (the part before the first `_`), with leading zeros stripped. Only migrations with a prefix greater than this value will be checked.

For example, if your last applied migration is `20250301120000_create_users.sql`, use `20250301120000`:

```json
{
    "migrations": {
        "migrationsDir": "supabase/migrations",
        "after": 20250301120000
    }
}
```

Alternatively, pass them directly.

```
postgres-language-server check supabase/migrations --migrations-dir="supabase/migrations" --after=20250301120000
```

This will only check migrations after the specified migration id.

Setting `migrationsDir` also limits where migration-only rules run, such as most rules in the `safety` and `destructive` groups. Files outside the directory are still linted, but skip these rules. Without `migrationsDir`, every rule runs on every file. The [rules reference](../reference/rules.md) marks which rules are migration-only.

!!! note
    The prefix format depends on your migration tool:

    - **Supabase**: datetime prefix, e.g. `20250301120000_create_users.sql` → use `20250301120000`
    - **Sequential**: numeric prefix, e.g. `000201_create_users.sql` → use `201` (leading zeros are stripped)

For pre-commit hooks and when working locally, use `--staged` to only lint files that have been staged. In CI environments, you most likely want to use `--changed` to only lint files that have been changed compared to your `vcs.default_branch` configuration. If `default_branch` is not set in your `postgres-language-server.jsonc`, use `--since=REF` to specify the base branch to compare against.

## Up and down migrations in one file

Some migration tools put the up and the down migration in the same file, separated by marker comments, and run each section on its own. Set `sectionMarkers` to the tool you use, so that each section is checked as a migration of its own:

```json
{
    "migrations": {
        "migrationsDir": "db/migrations",
        "sectionMarkers": "dbmate"
    }
}
```

| `sectionMarkers` | Markers | Without a transaction |
| --- | --- | --- |
| `"dbmate"` | `-- migrate:up`, `-- migrate:down` | `-- migrate:up transaction:false` (per section) |
| `"goose"` | `-- +goose Up`, `-- +goose Down` | `-- +goose NO TRANSACTION` (whole file) |
| `"none"` (default) | | |

With section markers:

- The transaction, lock, and session state starts fresh in each section. A lock taken or a `lock_timeout` set in the up section doesn't carry over to the down section.
- Sections run in a transaction, like the tools run them, unless the markers disable it. `banConcurrentIndexCreationInTransaction` flags `CONCURRENTLY` in every section that runs in a transaction, even when it's the only statement, and `banNonTransactionalStatementInTransaction` flags the other statements Postgres can't run in a transaction block, like `VACUUM` or `CREATE DATABASE`. In a section without a transaction, explicit `BEGIN` and `COMMIT` are fine for `transactionNesting`.
- dbmate sends each section to Postgres as one query, and Postgres runs a query with more than one statement in a transaction block. So `CONCURRENTLY` only works in a `transaction:false` section that has no other statement, and locks are held until the end of the section. goose sends the statements of a `NO TRANSACTION` file one by one, except for a `-- +goose StatementBegin` block, which is one query. A statement that goose sends on its own commits on its own, so its locks are released.
- A down section sees the objects of the up sections before it, because it only runs after them. Its own changes don't carry over to the sections after it. For example, `drop table posts` in the down section knows `posts` from the up section.
- Statements before the first marker are checked like a file without markers.
- goose's `-- +goose StatementBegin` and `-- +goose StatementEnd` need no setup: statements, including function bodies, are split by their SQL syntax.

Down sections are checked by default, since they run when you roll back. To only check the up sections, set `lintDown` to `false`. Syntax errors in down sections are still reported.

```json
{
    "migrations": {
        "sectionMarkers": "goose",
        "lintDown": false
    }
}
```
