# Continuous Integration

Running the Postgres Language Server in a CI environment can boost your teams confidence by linting your schema changes and catching errors early.

### GitHub Actions

We provide a first-party [GitHub Action](https://github.com/supabase-community/postgres-language-server-cli-action) to setup the CLI in your runner. Here's what a simple workflow might look like:

```yaml
jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: supabase-community/postgres-language-server-cli-action@main
        with:
          version: latest
      - run: postgres-language-server check --disable-db sql/
```

`--disable-db` runs the checks that do not need a database. Without it, `check` fails when a database is configured but cannot be reached (see [Connection failures](./configure_database.md#connection-failures)).

You likely want to setup Postgres to enable more advanced checks:

```yaml
jobs:
  lint:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:latest
        env:
          POSTGRES_USER: postgres
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: postgres
        ports:
          - 5432:5432
    steps:
      - uses: actions/checkout@v4
      - uses: supabase-community/postgres-language-server-cli-action@main
        with:
          version: latest
      - run: postgres-language-server check sql/
```

### Checking formatting

Without `--write`, `postgres-language-server format` does not modify any file. It prints a diff for every statement that would change and exits with a non-zero code if any file is not formatted, so it fails the job:

```yaml
jobs:
  format:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: supabase-community/postgres-language-server-cli-action@main
        with:
          version: latest
      - run: postgres-language-server format sql/
```

Run `postgres-language-server format sql/ --write` locally to fix the reported files. See [Formatting](../features/formatting.md#check-mode) for details.

### Checking migrations

A common use-case is to check your migration files. Check out [the dedicated guide](./checking_migrations.md) for details.
