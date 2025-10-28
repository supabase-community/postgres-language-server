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
      - uses: supabase-community/pglt-cli-action@main
        with:
          version: latest
      - run: postgres-language-server check --skip-db sql/
```

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
      - uses: supabase-community/pglt-cli-action@main
        with:
          version: latest
      - run: postgres-language-server check sql/
```

A common use-case is to check your migration files. Check out [the dedicated guide](./checking_migrations.md) for details.

