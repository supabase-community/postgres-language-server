# Getting Started

The Postgres Language Server can be installed as a development dependency of your project, a standalone executable, or as an extension of your favorite editor.

## Configuration

We recommend creating a `postgres-language-server.jsonc` configuration file for each project. This eliminates repetitive CLI options and ensures that consistent configuration in your editor. Some options are only available from a configuration file. This step is optional though: if you are happy with the defaults, you don’t need a configuration file. To create the `postgres-language-server.jsonc` file, run the `init` command in the root folder of your project:

```sh
postgres-language-server init
```

You’ll now have a `postgres-language-server.jsonc` file in your directory:

[//]: # "BEGIN DEFAULT_CONFIGURATION"

```json
{
  "$schema": "https://pg-language-server.com/latest/schema.json",
  "vcs": {
    "enabled": false,
    "clientKind": "git",
    "useIgnoreFile": false
  },
  "files": {
    "ignore": []
  },
  "linter": {
    "enabled": true,
    "rules": {
      "recommended": true
    }
  },
  "db": {
    "host": "127.0.0.1",
    "port": 5432,
    "username": "postgres",
    "password": "postgres",
    "database": "postgres",
    "connTimeoutSecs": 10,
    "allowStatementExecutionsAgainst": ["127.0.0.1/*", "localhost/*"]
  }
}
```

[//]: # "END DEFAULT_CONFIGURATION"

Make sure to edit the database connection settings to connect to your local development database. To see all options, run `postgres-language-server --help`.

You can use your current `postgres-language-server` version instead of "latest" in the `$schema` URL, e.g. `https://pg-language-server.com/0.8.1/schema.json`.

## Usage

Lets get a quick overview of how to use the Postgres Language Server in your project.

### Command-line interface

The CLI exposes a `check` command that will run all checks on the given files or paths.

```sh
# check a specific file
postgres-language-server check myfile.sql

# check a directory
postgres-language-server check supabase/migrations
```

Run `postgres-language-server --help` for all options. The CLI options take precedence over what is loaded from `postgres-language-server.jsonc`.

### Editor Integrations

The Postgres Language Server is available as an extension in your favorite editors.

- VSCode: The language server is available on the [VSCode Marketplace](https://marketplace.visualstudio.com/items?itemName=Supabase.postgrestools). It's published from [this repo](https://github.com/supabase-community/postgres-language-server-vscode).
- Neovim: You will have to install `nvim-lspconfig`, and follow the [instructions](https://github.com/neovim/nvim-lspconfig/blob/master/doc/configs.md#postgres_lsp).
- Emacs: The language client is available through [lsp-mode](https://github.com/emacs-lsp/lsp-mode). For more details, refer to their [manual page](https://emacs-lsp.github.io/lsp-mode/page/lsp-postgres/).
- Zed: The language server is available as an Extension. It's published from [this repo](https://github.com/LoamStudios/zed-postgres-language-server).

### Continuous Integration

Run `postgres-language-server check` in your CI pipeline to lint your schema changes and enforce code quality across your team. We provide a [GitHub Action](https://github.com/supabase-community/postgres-language-server-cli-action) to setup the Postgres Language Server in your runner.

See the [Continuous Integration](/guides/continuous_integration) guide for an example.


