<div align="center">
  <img src="docs/images/pls-github.png" alt="Postgres Language Server" width="80%">

  # Postgres Language Server
  A collection of language tools and a Language Server Protocol (LSP) implementation for Postgres, focusing on developer experience and reliable SQL tooling.

  [Documentation](https://pg-language-server.com/latest/) | [Installation](https://pg-language-server.com/latest/getting_started/) | [CLI Releases](https://github.com/supabase-community/postgres-language-server/releases)

  [VSCode](https://marketplace.visualstudio.com/items?itemName=Supabase.postgrestools) | [Cursor](https://open-vsx.org/extension/supabase/postgrestools) | [Neovim](https://github.com/neovim/nvim-lspconfig/blob/master/doc/configs.md#postgres_lsp) | [Zed](https://github.com/LoamStudios/zed-postgres-language-server) | [Sublime Text](https://lsp.sublimetext.io/language_servers/#postgresql)
</div>

## Overview

LSP Demo             |  CLI Demo
:-------------------------:|:-------------------------:
![LSP Demo](/docs/images/lsp-demo.gif)  |  ![CLI Demo](/docs/images/cli-demo.png)

This project provides a toolchain for Postgres development, built on Postgres' own parser `libpg_query` to ensure 100% syntax compatibility. It is built on a Server-Client architecture with a transport-agnostic design. All features can be accessed through the [Language Server Protocol](https://microsoft.github.io/language-server-protocol/), a CLI, HTTP APIs, and a WebAssembly module. The goal is to make all the great Postgres tooling out there as accessible as possible, and to build anything that is missing ourselves.

The following features are available today:

- [Autocompletion & Hover](https://pg-language-server.com/latest/features/editor_features/)
- [Syntax Diagnostics](https://pg-language-server.com/latest/features/syntax_diagnostics/)
- [Type Checking](https://pg-language-server.com/latest/features/type_checking/) (via `EXPLAIN` error insights)
- [Formatting](https://pg-language-server.com/latest/features/formatting/)
- [Migration Linting](https://pg-language-server.com/latest/features/linting/)
- [Database Linting](https://pg-language-server.com/latest/features/database_linting/)
- [PL/pgSQL Support](https://pg-language-server.com/latest/features/plpgsql/)

## Development

```bash
nix develop     # or skip if not using Nix
docker-compose up -d
```

## Acknowledgements

A big thanks to the following projects, without which this project wouldn't have been possible:

- [libpg_query](https://github.com/pganalyze/libpg_query): For extracting the Postgres' parser
- [Biome](https://github.com/biomejs/biome): For implementing a toolchain infrastructure we could copy from
- [Squawk](https://github.com/sbdchd/squawk): For the linter inspiration
