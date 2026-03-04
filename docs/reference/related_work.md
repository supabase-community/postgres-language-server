# Related Work

Several tools address migration safety and schema change linting for PostgreSQL. This page lists tools that solve related problems, with an honest summary of what each does best.

Some of our [lint rules are ported from these tools](./rule_sources.md).

## Migration Safety Linters

### Eugene

- **Language**: Rust
- **Repository**: [github.com/kaaveland/eugene](https://github.com/kaaveland/eugene)
- **Approach**: DDL lint with a unique trace mode that runs migrations against a real database and reports actual lock events
- **Best for**: Teams that want ground-truth lock analysis from observed Postgres behavior, not just static analysis
- **Scope**: Raw SQL migrations

### Squawk

- **Language**: Rust
- **Repository**: [github.com/sbdchd/squawk](https://github.com/sbdchd/squawk)
- **Approach**: Static SQL linter with a large rule set and mature GitHub Action
- **Best for**: Teams already using raw SQL migrations who want broad coverage of DDL anti-patterns and naming conventions
- **Scope**: Raw SQL migrations

### pgfence

- **Language**: TypeScript / Node.js
- **Repository**: [github.com/flvmnt/pgfence](https://github.com/flvmnt/pgfence)
- **Approach**: CLI that analyzes migrations for lock modes, risk levels, and outputs safe rewrite recipes
- **Best for**: Teams using ORMs (TypeORM, Knex, Sequelize, Drizzle, Prisma) who need safety analysis on generated SQL, or teams that want size-aware risk scoring
- **Scope**: Raw SQL and ORM-generated migrations

### strong_migrations

- **Language**: Ruby
- **Repository**: [github.com/ankane/strong_migrations](https://github.com/ankane/strong_migrations)
- **Approach**: Runtime checks integrated into ActiveRecord migrations
- **Best for**: Rails teams, where it catches dangerous patterns at migration execution time
- **Scope**: Rails / ActiveRecord only

## Migration Executors

### pgroll

- **Language**: Go
- **Repository**: [github.com/xataio/pgroll](https://github.com/xataio/pgroll)
- **Approach**: Migration executor that implements expand/contract deployments natively using dual-version schemas with views
- **Best for**: Teams that want zero-downtime schema changes handled at the execution layer, not just analysis
- **Scope**: Operates as a migration runner, not a linter

## How These Tools Relate to Postgres Language Server

Postgres Language Server provides real-time feedback in the editor via LSP and a CLI for CI. The tools above focus on migration-specific concerns. They can be used together:

- Use **Postgres Language Server** in your editor for syntax, type checking, and safety lint rules as you write SQL
- Use a **migration linter** (Eugene, Squawk, pgfence) in CI to catch migration-specific risks before merge
- Use a **migration executor** (pgroll) at deploy time for zero-downtime schema changes
