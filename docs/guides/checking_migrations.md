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

!!! note
    The prefix format depends on your migration tool:

    - **Supabase**: datetime prefix, e.g. `20250301120000_create_users.sql` → use `20250301120000`
    - **Sequential**: numeric prefix, e.g. `000201_create_users.sql` → use `201` (leading zeros are stripped)

For pre-commit hooks and when working locally, use `--staged` to only lint files that have been staged. In CI environments, you most likely want to use `--changed` to only lint files that have been changed compared to your `vcs.default_branch` configuration. If `default_branch` is not set in your `postgres-language-server.jsonc`, use `--since=REF` to specify the base branch to compare against.

