# Related Work

Several tools address migration safety and schema change management for PostgreSQL.

Some of our [lint rules are ported from these tools](./rule_sources.md).

## Migration Safety

- [**Eugene**](https://github.com/kaaveland/eugene) - Lint and trace lock behaviour of SQL migrations for PostgreSQL
- [**Squawk**](https://github.com/sbdchd/squawk) - Linter for PostgreSQL migrations and SQL
- [**pgfence**](https://github.com/flvmnt/pgfence) - PostgreSQL migration safety CLI with lock mode analysis, risk scoring, and safe rewrite recipes
- [**Strong Migrations**](https://github.com/ankane/strong_migrations) - Catch unsafe migrations in development (Ruby / ActiveRecord)

## Migration Execution

- [**pgroll**](https://github.com/xataio/pgroll) - PostgreSQL zero-downtime migrations made easy
