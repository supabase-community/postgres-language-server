# Formatting Declarative Schemas

Tools that generate SQL, like the Supabase CLI with [pg-delta](https://github.com/supabase/pg-toolbelt), print just enough formatting to keep their output readable. They don't format the SQL you write by hand, and they keep your function bodies as written. Run the formatter after each generation to give the generated and the handwritten SQL the same style, the same way you would run Prettier on a generated JSON file before committing it.

## Supabase declarative schemas

The Supabase CLI writes your schema to `supabase/schemas/` (or the directory set by `experimental.pgdelta.declarative_schema_path`) and the migrations to `supabase/migrations/`. Format both after any command that writes them, such as `supabase db schema declarative generate`, `supabase db schema declarative sync`, or `supabase db pull --declarative`:

```sh
supabase db schema declarative sync
postgres-language-server format supabase/schemas supabase/migrations --write
```

The first run also formats the migrations you already have. Once they are formatted, each run only changes the files that were just generated. To leave existing migrations as they are, pass the new migration file instead of the `supabase/migrations` directory.

pg-delta loads the schema files through Postgres, so formatting doesn't change the diff, with one exception: Postgres stores the body of a `language sql` function written as a string as it was written, and the formatter reformats these bodies by default. Set `skipFnBodies` to keep function bodies as written, so that a formatted schema still syncs without changes:

```json
{
  "format": {
    "skipFnBodies": true
  }
}
```

Bodies written with `begin atomic` are stored as a parsed tree and are not affected.

## Checking formatting in CI

Without `--write`, `format` exits with a non-zero code if a file is not formatted. Run it in CI to catch a schema or migration that was generated but not formatted:

```sh
postgres-language-server format supabase/schemas supabase/migrations
```

To only check the files a pull request changes, run `postgres-language-server format --changed` instead. It checks every file changed compared to your default branch, in place of the paths (see [Integrate with VCS](vcs_integration.md#process-only-changed-files)). See [Continuous Integration](continuous_integration.md) to install the CLI in your pipeline, and [Formatting](../features/formatting.md) for all formatter options.
