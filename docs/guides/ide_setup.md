# Use in your IDE

Th Postgres Language Server has first-class [LSP](https://microsoft.github.io/language-server-protocol/) support to seamlessly integrate into your favorite editor.

## VSCode

The language server is available on the [VSCode Marketplace](https://marketplace.visualstudio.com/items?itemName=Supabase.postgrestools). It's published from [this repo](https://github.com/supabase-community/postgres-language-server-vscode).

## Neovim

You will have to install `nvim-lspconfig`, and follow the [instructions](https://github.com/neovim/nvim-lspconfig/blob/master/doc/configs.md#postgres_lsp).

## Emacs

The language client is available through [lsp-mode](https://github.com/emacs-lsp/lsp-mode). For more details, refer to their [manual page](https://emacs-lsp.github.io/lsp-mode/page/lsp-postgres/).

## Zed

The language server is available as an Extension. It's published from [this repo](https://github.com/LoamStudios/zed-postgres-language-server).

> [!NOTE]
> Is there a extension for an editor that is not listed here? Please file a PR and we will be happy to add it to the list

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

The daemon saves logs in your file system. Logs are stored in a folder called `pgt-logs`. The path of this folder changes based on your operative system:

- Linux: `~/.cache/pgt;`
- Windows: `C:\Users\<UserName>\AppData\Local\supabase-community\pgt\cache`
- macOS: `/Users/<UserName>/Library/Caches/dev.supabase-community.pgt`

For other operative systems, you can find the folder in the system’s temporary directory.

You can change the location of the `pgt-logs` folder via the `PGT_LOG_PATH` variable.

