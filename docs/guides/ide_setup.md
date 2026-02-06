# Use in your IDE

Th Postgres Language Server has first-class [LSP](https://microsoft.github.io/language-server-protocol/) support to seamlessly integrate into your favorite editor.

## VSCode

The language server is available on the [VSCode Marketplace](https://marketplace.visualstudio.com/items?itemName=Supabase.postgrestools). It's published from [this repo](https://github.com/supabase-community/postgres-language-server-vscode).

## Neovim

### Installation via Mason (Recommended)

The easiest way to install the language server is via [Mason](https://github.com/williamboman/mason.nvim):

```vim
:MasonInstall postgres-language-server
```

Then configure `nvim-lspconfig` to use it:

```lua
require('lspconfig').postgres_lsp.setup{}
```

### Manual Installation

1. Download the binary for your platform from the [releases page](https://github.com/supabase-community/postgres-language-server/releases)
2. Place it in your `$PATH`
3. Configure `nvim-lspconfig`:

```lua
require('lspconfig').postgres_lsp.setup{
  cmd = { "postgres-language-server", "lsp-proxy" },
}
```

For more details, see the [nvim-lspconfig documentation](https://github.com/neovim/nvim-lspconfig/blob/master/doc/configs.md#postgres_lsp).

### Troubleshooting

If the language server isn't working:

1. **Check that the binary is executable:**
   ```bash
   which postgres-language-server
   postgres-language-server --version
   ```

2. **Check LSP status in Neovim:**
   ```vim
   :LspInfo
   ```
   This shows if the server is attached to your buffer.

3. **Check the logs:**
   ```vim
   :LspLog
   ```
   Look for errors related to `postgres_lsp`.

4. **Verify your file is recognized as SQL:**
   ```vim
   :set filetype?
   ```
   Should output `filetype=sql` for `.sql` files.

## Emacs

The language client is available through [lsp-mode](https://github.com/emacs-lsp/lsp-mode). For more details, refer to their [manual page](https://emacs-lsp.github.io/lsp-mode/page/lsp-postgres/).

## Zed

The language server is available as an Extension. It's published from [this repo](https://github.com/LoamStudios/zed-postgres-language-server).

> [!NOTE]
> Is there a extension for an editor that is not listed here? Please file a PR and we will be happy to add it to the list

## Sublime Text

See [install instructions](https://lsp.sublimetext.io/language_servers/#postgresql).

## Integrate in an editor extension

Is your favorite editor missing? Thanks to the language server protocol, integrating the Postgres Language Server protocol should be straightforward.

### Use the LSP proxy
The CLI has a command called `lsp-proxy`. When executed, we will spawn two processes:

- a daemon that does execute the requested operations;
- a server that functions as a proxy between the requests of the client - the editor - and the server - the daemon;

If your editor is able to interact with a server and send [JSON-RPC](https://www.jsonrpc.org/) request, you only need to configure the editor run that command.


### Use the daemon with the binary
Using the binary via CLI is very efficient, although you will not be able to provide logs to your users. The CLI allows you to bootstrap a daemon and then use the CLI commands through the daemon itself.

If order to do so, you first need to start a daemon process with the start command:

```shell
postgres-language-server start
```

Then, every command needs to add the `--use-server` options, e.g.:

```shell
postgres-language-server check --use-server --stdin-file-path=dummy.sql
```


> [!Note]
> If you decide to use the daemon, you’re also responsible to restart/kill the process with the stop command, to avoid having ghost processes.
Caution

Operations via the daemon are significantly slower than the CLI itself, so it’s advised to run operations only on single files.

### Daemon logs

The daemon saves logs in your file system. Logs are stored in a folder called `pgls-logs`. The path of this folder changes based on your operative system:

- Linux: `~/.cache/pgls;`
- Windows: `C:\Users\<UserName>\AppData\Local\supabase-community\pgls\cache`
- macOS: `/Users/<UserName>/Library/Caches/dev.supabase-community.pgls`

For other operative systems, you can find the folder in the system’s temporary directory.

You can change the location of the `pgls-logs` folder via the `PGLS_LOG_PATH` variable.

