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


### Combining with pgfence for Migration Safety

While Postgres Language Server catches SQL syntax and semantic issues, it does not analyze the **lock modes** and **runtime risk** of DDL statements. [pgfence](https://pgfence.dev) is a complementary CLI that fills this gap — it tells you which PostgreSQL lock mode each migration statement acquires, what it blocks, and provides safe rewrite recipes when it detects dangerous patterns.

By running both tools in CI, you get comprehensive migration checking: PGLT ensures your SQL is correct, and pgfence ensures it is safe to run against a live database.

```yaml
jobs:
  check-migrations:
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

      # 1. Check SQL syntax and semantics with PGLT
      - uses: supabase-community/pglt-cli-action@main
        with:
          version: latest
      - name: Check SQL syntax (PGLT)
        run: postgres-language-server check sql/

      # 2. Check migration safety with pgfence
      - name: Check migration safety (pgfence)
        run: npx --yes @flvmnt/pgfence@0.2.1 analyze --ci migrations/*.sql
```

pgfence exits with code 1 when it finds high-risk issues, so it integrates naturally as a CI gate. You can adjust the threshold with `--max-risk`:

```sh
# Block on HIGH or above (default)
npx --yes @flvmnt/pgfence@0.2.1 analyze --ci migrations/*.sql

# Block on MEDIUM or above (stricter)
npx --yes @flvmnt/pgfence@0.2.1 analyze --ci --max-risk medium migrations/*.sql
```

For more details, see the [pgfence documentation](https://pgfence.dev).
