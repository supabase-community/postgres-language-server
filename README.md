![Postgres Language Server](/docs/images/pls-github.png)

# Postgres Language Server

A collection of language tools and a Language Server Protocol (LSP) implementation for Postgres, focusing on developer experience and reliable SQL tooling.

Docs: [pg-language-server.com](https://pg-language-server.com/)

Install: [instructions](https://pg-language-server.com/#installation)

- [CLI releases](https://github.com/supabase-community/postgres-language-server/releases)
- [VSCode](https://marketplace.visualstudio.com/items?itemName=Supabase.postgrestools)
- [Neovim](https://github.com/neovim/nvim-lspconfig/blob/master/doc/configs.md#postgres_lsp)
- [Zed](https://github.com/LoamStudios/zed-postgres-language-server)

## Overview
LSP Demo             |  CLI Demo
:-------------------------:|:-------------------------:
![LSP Demo](/docs/images/lsp-demo.gif)  |  ![CLI Demo](/docs/images/cli-demo.png)

This project provides a toolchain for Postgres development, built on Postgres' own parser `libpg_query` to ensure 100% syntax compatibility. It is built on a Server-Client architecture with a transport-agnostic design. This means all features can be accessed not only through the [Language Server Protocol](https://microsoft.github.io/language-server-protocol/), but also through other interfaces like a CLI, HTTP APIs, or a WebAssembly module. The goal is to make all the great Postgres tooling out there as accessible as possible, and to build anything that is missing ourselves.

The following features are implemented:
- Autocompletion
- Syntax Error Highlighting
- Type-checking (via `EXPLAIN` error insights)
- Linter, inspired by [Squawk](https://squawkhq.com)

Our current focus is on refining and enhancing these core features while building a robust and easily accessible infrastructure. For future plans and opportunities to contribute, please check out the issues and discussions. Any contributions are welcome!

## Contributors

- [psteinroe](https://github.com/psteinroe)
- [juleswritescode](https://github.com/juleswritescode)

## Development

### Using Nix

```bash
nix develop
docker-compose up -d
```

### Using Docker

```bash
docker-compose up -d
```

## Acknowledgements

A big thanks to the following projects, without which this project wouldn't have been possible:

- [libpg_query](https://github.com/pganalyze/libpg_query): For extracting the Postgres' parser
- [Biome](https://github.com/biomejs/biome): For implementing a toolchain infrastructure we could copy from
- [Squawk](https://github.com/sbdchd/squawk): For the linter inspiration
