![Postgres Language Server](images/pls-github.png)

# Postgres Language Server

A collection of language tools and a Language Server Protocol (LSP) implementation for Postgres, focusing on developer experience and reliable SQL tooling.

---

**Source Code**: <a href="https://github.com/supabase-community/postgres-language-server" target="_blank">https://github.com/supabase-community/postgres-language-server</a>

---

The language server is built on Postgres' own parser `libpg_query` to ensure 100% syntax compatibility. It uses a Server-Client architecture and is a transport-agnostic. This means all features can be accessed through the Language Server Protocol as well as a CLI.

The following features are available today:

- [Autocompletion & Hover](features/editor_features.md)
- [Syntax Diagnostics](features/syntax_diagnostics.md)
- [Type Checking](features/type_checking.md) (via `EXPLAIN` error insights)
- [Formatting](features/formatting.md)
- [Migration Linting](features/linting.md)
- [Database Linting](features/database_linting.md)
- [PL/pgSQL Support](features/plpgsql.md)

For future plans and opportunities to contribute, please check out the issues and discussions. Any contributions are welcome!
