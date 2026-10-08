# CLI Reference

[//]: # "BEGIN CLI_REF"



# Command summary

  * [`postgres-language-server`↴](#postgres-language-server)
  * [`postgres-language-server version`↴](#postgres-language-server-version)
  * [`postgres-language-server dblint`↴](#postgres-language-server-dblint)
  * [`postgres-language-server check`↴](#postgres-language-server-check)
  * [`postgres-language-server format`↴](#postgres-language-server-format)
  * [`postgres-language-server start`↴](#postgres-language-server-start)
  * [`postgres-language-server stop`↴](#postgres-language-server-stop)
  * [`postgres-language-server init`↴](#postgres-language-server-init)
  * [`postgres-language-server lsp-proxy`↴](#postgres-language-server-lsp-proxy)
  * [`postgres-language-server parse`↴](#postgres-language-server-parse)
  * [`postgres-language-server schema-export`↴](#postgres-language-server-schema-export)

## postgres-language-server

Postgres Language Server official CLI. Use it to check the health of your project or run it to check single files.

**Usage**: **`postgres-language-server`** (_`COMMAND ...`_ | **`--clean`**)

**Available options:**
- **`    --clean`** &mdash; 
  Cleans the logs emitted by the daemon.
- **`-h`**, **`--help`** &mdash; 
  Prints help information
- **`-V`**, **`--version`** &mdash; 
  Prints version information



**Available commands:**
- **`version`** &mdash; 
  Shows the version information and quit.
- **`dblint`** &mdash; 
  Lints your database schema.
- **`check`** &mdash; 
  Runs everything to the requested files.
- **`format`** &mdash; 
  Formats the requested files.
- **`start`** &mdash; 
  Starts the daemon server process.
- **`stop`** &mdash; 
  Stops the daemon server process.
- **`init`** &mdash; 
  Bootstraps a new project. Creates a configuration file with some defaults.
- **`lsp-proxy`** &mdash; 
  Acts as a server for the Language Server Protocol over stdin/stdout.
- **`parse`** &mdash; 
  Parses a SQL file (or standard input) and writes the parse tree as a protobuf `ParseResult` message.
- **`schema-export`** &mdash; 
  Exports the database schema to JSON for use with WASM bindings.


## postgres-language-server version

Shows the version information and quit.

**Usage**: **`postgres-language-server`** **`version`** 

**Global options applied to all commands**
- **`    --colors`**=_`<off|force>`_ &mdash; 
  Set the formatting mode for markup: "off" prints everything as plain text, "force" forces the formatting of markup using ANSI even if the console output is determined to be incompatible
- **`    --use-server`** &mdash; 
  Connect to a running instance of the daemon server.
- **`    --verbose`** &mdash; 
  Print additional diagnostics, and some diagnostics show more information. Also, print out what files were processed and which ones were modified.
- **`    --config-path`**=_`PATH`_ &mdash; 
  Set the file path to the configuration file, or the directory path to find `postgres-language-server.jsonc`. If used, it disables the default configuration file resolution.
- **`    --max-diagnostics`**=_`<none|<NUMBER>>`_ &mdash; 
  Cap the amount of diagnostics displayed. When `none` is provided, the limit is lifted.
   
  [default: 20]
- **`    --skip-errors`** &mdash; 
  Skip over files containing syntax errors instead of emitting an error diagnostic.
- **`    --no-errors-on-unmatched`** &mdash; 
  Silence errors that would be emitted in case no files were processed during the execution of the command.
- **`    --error-on-warnings`** &mdash; 
  Tell Postgres Language Server to exit with an error code if some diagnostics emit warnings.
- **`    --reporter`**=_`<json|json-pretty|github|junit|summary|gitlab>`_ &mdash; 
  Allows to change how diagnostics and summary are reported.
- **`    --log-level`**=_`<none|debug|info|warn|error>`_ &mdash; 
  The level of logging. In order, from the most verbose to the least verbose: debug, info, warn, error.

  The value `none` won't show any logging.
   
  Uses environment variable **`PGLS_LOG_LEVEL`**
   
  [default: none]
- **`    --log-kind`**=_`<pretty|compact|json>`_ &mdash; 
  How the log should look like.
   
  [default: pretty]
- **`    --diagnostic-level`**=_`<info|warn|error>`_ &mdash; 
  The level of diagnostics to show. In order, from the lowest to the most important: info, warn, error. Passing `--diagnostic-level=error` will cause Postgres Tools to print only diagnostics that contain only errors.
   
  [default: info]



**Available options:**
- **`-h`**, **`--help`** &mdash; 
  Prints help information


## postgres-language-server dblint

Lints your database schema.

**Usage**: **`postgres-language-server`** **`dblint`** 

**The configuration that is contained inside the configuration file.**
- **`    --vcs-enabled`**=_`<true|false>`_ &mdash; 
  Whether we should integrate itself with the VCS client
- **`    --vcs-client-kind`**=_`<git>`_ &mdash; 
  The kind of client.
- **`    --vcs-use-ignore-file`**=_`<true|false>`_ &mdash; 
  Whether we should use the VCS ignore file. When [true], we will ignore the files specified in the ignore file.
- **`    --vcs-root`**=_`PATH`_ &mdash; 
  The folder where we should check for VCS files. By default, we will use the same folder where `postgres-language-server.jsonc` was found.

  If we can't find the configuration, it will attempt to use the current working directory. If no current working directory can't be found, we won't use the VCS integration, and a diagnostic will be emitted
- **`    --vcs-default-branch`**=_`BRANCH`_ &mdash; 
  The main branch of the project
- **`    --files-max-size`**=_`NUMBER`_ &mdash; 
  The maximum allowed size for source code files in bytes. Files above this limit will be ignored for performance reasons. Defaults to 1 MiB
- **`    --migrations-dir`**=_`ARG`_ &mdash; 
  The directory where the migration files are stored
- **`    --after`**=_`ARG`_ &mdash; 
  Ignore any migrations before this timestamp
- **`    --line-width`**=_`ARG`_ &mdash; 
  Maximum line width before breaking. Default: 100.
- **`    --indent-size`**=_`ARG`_ &mdash; 
  Number of spaces (or tab width) for indentation. Default: 2.
- **`    --indent-style`**=_`ARG`_ &mdash; 
  Indentation style: "spaces" or "tabs". Default: "spaces".
- **`    --keyword-case`**=_`ARG`_ &mdash; 
  Keyword casing: "upper" or "lower". Default: "lower".
- **`    --constant-case`**=_`ARG`_ &mdash; 
  Constant casing (NULL, TRUE, FALSE): "upper" or "lower". Default: "lower".
- **`    --type-case`**=_`ARG`_ &mdash; 
  Data type casing (text, varchar, int): "upper" or "lower". Default: "lower".
- **`    --comma-style`**=_`ARG`_ &mdash; 
  Where a comma sits when a list breaks: "trailing" or "leading". Default: "trailing".
- **`    --logical-operator-placement`**=_`ARG`_ &mdash; 
  Where a boolean operator sits when a condition breaks: "trailing" or "leading". Default: "trailing".
- **`    --layout`**=_`ARG`_ &mdash; 
  How a statement is laid out: "fit" breaks only when a line would exceed the line width, "expanded" always breaks between clauses. Default: "fit".
- **`    --cast-style`**=_`ARG`_ &mdash; 
  How an explicit cast is spelled: "cast" for `CAST(x AS t)`, "operator" for `x::t`. Default: "cast".
- **`    --clause-body-style`**=_`ARG`_ &mdash; 
  Where the body of a clause starts: "break" for a new line, "compact" to keep the first element on the keyword line. Default: "break".
- **`    --isolate-semicolon`**=_`ARG`_ &mdash; 
  If `true`, the terminating semicolon goes on its own line when the statement spans several lines. Default: `false`.
- **`    --skip-fn-bodies`**=_`ARG`_ &mdash; 
  If `true`, skip formatting of SQL function bodies (keep them verbatim). Default: `false`.
- **`    --search_path`**=_`ARG`_ &mdash; 
  Default search path schemas for type checking. Can be a list of schema names or glob patterns like ["public", "app_*"]. If not specified, defaults to ["public"].
- **`    --connection-string`**=_`ARG`_ &mdash; 
  A connection string that encodes the full connection setup. When provided, it takes precedence over the individual fields. Can also be set via the `DATABASE_URL` environment variable.
- **`    --host`**=_`ARG`_ &mdash; 
  The host of the database. Required if you want database-related features. All else falls back to sensible defaults. Can also be set via the `PGHOST` environment variable.
- **`    --port`**=_`ARG`_ &mdash; 
  The port of the database. Can also be set via the `PGPORT` environment variable.
- **`    --username`**=_`ARG`_ &mdash; 
  The username to connect to the database. Can also be set via the `PGUSER` environment variable.
- **`    --password`**=_`ARG`_ &mdash; 
  The password to connect to the database. Can also be set via the `PGPASSWORD` environment variable.
- **`    --database`**=_`ARG`_ &mdash; 
  The name of the database. Can also be set via the `PGDATABASE` environment variable.
- **`    --allow_statement_executions_against`**=_`ARG`_
- **`    --conn_timeout_secs`**=_`ARG`_ &mdash; 
  The connection timeout in seconds.
   
  [default: Some(10)]
- **`    --disable-db`** &mdash; 
  Actively disable all database-related features.



**Global options applied to all commands**
- **`    --colors`**=_`<off|force>`_ &mdash; 
  Set the formatting mode for markup: "off" prints everything as plain text, "force" forces the formatting of markup using ANSI even if the console output is determined to be incompatible
- **`    --use-server`** &mdash; 
  Connect to a running instance of the daemon server.
- **`    --verbose`** &mdash; 
  Print additional diagnostics, and some diagnostics show more information. Also, print out what files were processed and which ones were modified.
- **`    --config-path`**=_`PATH`_ &mdash; 
  Set the file path to the configuration file, or the directory path to find `postgres-language-server.jsonc`. If used, it disables the default configuration file resolution.
- **`    --max-diagnostics`**=_`<none|<NUMBER>>`_ &mdash; 
  Cap the amount of diagnostics displayed. When `none` is provided, the limit is lifted.
   
  [default: 20]
- **`    --skip-errors`** &mdash; 
  Skip over files containing syntax errors instead of emitting an error diagnostic.
- **`    --no-errors-on-unmatched`** &mdash; 
  Silence errors that would be emitted in case no files were processed during the execution of the command.
- **`    --error-on-warnings`** &mdash; 
  Tell Postgres Language Server to exit with an error code if some diagnostics emit warnings.
- **`    --reporter`**=_`<json|json-pretty|github|junit|summary|gitlab>`_ &mdash; 
  Allows to change how diagnostics and summary are reported.
- **`    --log-level`**=_`<none|debug|info|warn|error>`_ &mdash; 
  The level of logging. In order, from the most verbose to the least verbose: debug, info, warn, error.

  The value `none` won't show any logging.
   
  Uses environment variable **`PGLS_LOG_LEVEL`**
   
  [default: none]
- **`    --log-kind`**=_`<pretty|compact|json>`_ &mdash; 
  How the log should look like.
   
  [default: pretty]
- **`    --diagnostic-level`**=_`<info|warn|error>`_ &mdash; 
  The level of diagnostics to show. In order, from the lowest to the most important: info, warn, error. Passing `--diagnostic-level=error` will cause Postgres Tools to print only diagnostics that contain only errors.
   
  [default: info]



**Available options:**
- **`-h`**, **`--help`** &mdash; 
  Prints help information


## postgres-language-server check

Runs everything to the requested files.

**Usage**: **`postgres-language-server`** **`check`** \[**`--staged`**\] \[**`--changed`**\] \[**`--since`**=_`REF`_\] \[_`PATH`_\]...

**The configuration that is contained inside the configuration file.**
- **`    --vcs-enabled`**=_`<true|false>`_ &mdash; 
  Whether we should integrate itself with the VCS client
- **`    --vcs-client-kind`**=_`<git>`_ &mdash; 
  The kind of client.
- **`    --vcs-use-ignore-file`**=_`<true|false>`_ &mdash; 
  Whether we should use the VCS ignore file. When [true], we will ignore the files specified in the ignore file.
- **`    --vcs-root`**=_`PATH`_ &mdash; 
  The folder where we should check for VCS files. By default, we will use the same folder where `postgres-language-server.jsonc` was found.

  If we can't find the configuration, it will attempt to use the current working directory. If no current working directory can't be found, we won't use the VCS integration, and a diagnostic will be emitted
- **`    --vcs-default-branch`**=_`BRANCH`_ &mdash; 
  The main branch of the project
- **`    --files-max-size`**=_`NUMBER`_ &mdash; 
  The maximum allowed size for source code files in bytes. Files above this limit will be ignored for performance reasons. Defaults to 1 MiB
- **`    --migrations-dir`**=_`ARG`_ &mdash; 
  The directory where the migration files are stored
- **`    --after`**=_`ARG`_ &mdash; 
  Ignore any migrations before this timestamp
- **`    --line-width`**=_`ARG`_ &mdash; 
  Maximum line width before breaking. Default: 100.
- **`    --indent-size`**=_`ARG`_ &mdash; 
  Number of spaces (or tab width) for indentation. Default: 2.
- **`    --indent-style`**=_`ARG`_ &mdash; 
  Indentation style: "spaces" or "tabs". Default: "spaces".
- **`    --keyword-case`**=_`ARG`_ &mdash; 
  Keyword casing: "upper" or "lower". Default: "lower".
- **`    --constant-case`**=_`ARG`_ &mdash; 
  Constant casing (NULL, TRUE, FALSE): "upper" or "lower". Default: "lower".
- **`    --type-case`**=_`ARG`_ &mdash; 
  Data type casing (text, varchar, int): "upper" or "lower". Default: "lower".
- **`    --comma-style`**=_`ARG`_ &mdash; 
  Where a comma sits when a list breaks: "trailing" or "leading". Default: "trailing".
- **`    --logical-operator-placement`**=_`ARG`_ &mdash; 
  Where a boolean operator sits when a condition breaks: "trailing" or "leading". Default: "trailing".
- **`    --layout`**=_`ARG`_ &mdash; 
  How a statement is laid out: "fit" breaks only when a line would exceed the line width, "expanded" always breaks between clauses. Default: "fit".
- **`    --cast-style`**=_`ARG`_ &mdash; 
  How an explicit cast is spelled: "cast" for `CAST(x AS t)`, "operator" for `x::t`. Default: "cast".
- **`    --clause-body-style`**=_`ARG`_ &mdash; 
  Where the body of a clause starts: "break" for a new line, "compact" to keep the first element on the keyword line. Default: "break".
- **`    --isolate-semicolon`**=_`ARG`_ &mdash; 
  If `true`, the terminating semicolon goes on its own line when the statement spans several lines. Default: `false`.
- **`    --skip-fn-bodies`**=_`ARG`_ &mdash; 
  If `true`, skip formatting of SQL function bodies (keep them verbatim). Default: `false`.
- **`    --search_path`**=_`ARG`_ &mdash; 
  Default search path schemas for type checking. Can be a list of schema names or glob patterns like ["public", "app_*"]. If not specified, defaults to ["public"].
- **`    --connection-string`**=_`ARG`_ &mdash; 
  A connection string that encodes the full connection setup. When provided, it takes precedence over the individual fields. Can also be set via the `DATABASE_URL` environment variable.
- **`    --host`**=_`ARG`_ &mdash; 
  The host of the database. Required if you want database-related features. All else falls back to sensible defaults. Can also be set via the `PGHOST` environment variable.
- **`    --port`**=_`ARG`_ &mdash; 
  The port of the database. Can also be set via the `PGPORT` environment variable.
- **`    --username`**=_`ARG`_ &mdash; 
  The username to connect to the database. Can also be set via the `PGUSER` environment variable.
- **`    --password`**=_`ARG`_ &mdash; 
  The password to connect to the database. Can also be set via the `PGPASSWORD` environment variable.
- **`    --database`**=_`ARG`_ &mdash; 
  The name of the database. Can also be set via the `PGDATABASE` environment variable.
- **`    --allow_statement_executions_against`**=_`ARG`_
- **`    --conn_timeout_secs`**=_`ARG`_ &mdash; 
  The connection timeout in seconds.
   
  [default: Some(10)]
- **`    --disable-db`** &mdash; 
  Actively disable all database-related features.



**Global options applied to all commands**
- **`    --colors`**=_`<off|force>`_ &mdash; 
  Set the formatting mode for markup: "off" prints everything as plain text, "force" forces the formatting of markup using ANSI even if the console output is determined to be incompatible
- **`    --use-server`** &mdash; 
  Connect to a running instance of the daemon server.
- **`    --verbose`** &mdash; 
  Print additional diagnostics, and some diagnostics show more information. Also, print out what files were processed and which ones were modified.
- **`    --config-path`**=_`PATH`_ &mdash; 
  Set the file path to the configuration file, or the directory path to find `postgres-language-server.jsonc`. If used, it disables the default configuration file resolution.
- **`    --max-diagnostics`**=_`<none|<NUMBER>>`_ &mdash; 
  Cap the amount of diagnostics displayed. When `none` is provided, the limit is lifted.
   
  [default: 20]
- **`    --skip-errors`** &mdash; 
  Skip over files containing syntax errors instead of emitting an error diagnostic.
- **`    --no-errors-on-unmatched`** &mdash; 
  Silence errors that would be emitted in case no files were processed during the execution of the command.
- **`    --error-on-warnings`** &mdash; 
  Tell Postgres Language Server to exit with an error code if some diagnostics emit warnings.
- **`    --reporter`**=_`<json|json-pretty|github|junit|summary|gitlab>`_ &mdash; 
  Allows to change how diagnostics and summary are reported.
- **`    --log-level`**=_`<none|debug|info|warn|error>`_ &mdash; 
  The level of logging. In order, from the most verbose to the least verbose: debug, info, warn, error.

  The value `none` won't show any logging.
   
  Uses environment variable **`PGLS_LOG_LEVEL`**
   
  [default: none]
- **`    --log-kind`**=_`<pretty|compact|json>`_ &mdash; 
  How the log should look like.
   
  [default: pretty]
- **`    --diagnostic-level`**=_`<info|warn|error>`_ &mdash; 
  The level of diagnostics to show. In order, from the lowest to the most important: info, warn, error. Passing `--diagnostic-level=error` will cause Postgres Tools to print only diagnostics that contain only errors.
   
  [default: info]



**Available positional items:**
- _`PATH`_ &mdash; 
  Single file, single path or list of paths



**Available options:**
- **`    --stdin-file-path`**=_`PATH`_ &mdash; 
  Use this option when you want to format code piped from `stdin`, and print the output to `stdout`.

  The file doesn't need to exist on disk, what matters is the extension of the file. Based on the extension, we know how to check the code.

  Example: `echo 'let a;' | pgls_cli check --stdin-file-path=test.sql`
- **`    --staged`** &mdash; 
  When set to true, only the files that have been staged (the ones prepared to be committed) will be linted. This option should be used when working locally.
- **`    --changed`** &mdash; 
  When set to true, only the files that have been changed compared to your `defaultBranch` configuration will be linted. This option should be used in CI environments.
- **`    --since`**=_`REF`_ &mdash; 
  Use this to specify the base branch to compare against when you're using the --changed flag and the `defaultBranch` is not set in your `postgres-language-server.jsonc`
- **`-h`**, **`--help`** &mdash; 
  Prints help information


## postgres-language-server format

Formats the requested files.

**Usage**: **`postgres-language-server`** **`format`** \[**`--write`**\] \[**`--staged`**\] \[**`--changed`**\] \[**`--since`**=_`REF`_\] \[_`PATH`_\]...

**The configuration that is contained inside the configuration file.**
- **`    --vcs-enabled`**=_`<true|false>`_ &mdash; 
  Whether we should integrate itself with the VCS client
- **`    --vcs-client-kind`**=_`<git>`_ &mdash; 
  The kind of client.
- **`    --vcs-use-ignore-file`**=_`<true|false>`_ &mdash; 
  Whether we should use the VCS ignore file. When [true], we will ignore the files specified in the ignore file.
- **`    --vcs-root`**=_`PATH`_ &mdash; 
  The folder where we should check for VCS files. By default, we will use the same folder where `postgres-language-server.jsonc` was found.

  If we can't find the configuration, it will attempt to use the current working directory. If no current working directory can't be found, we won't use the VCS integration, and a diagnostic will be emitted
- **`    --vcs-default-branch`**=_`BRANCH`_ &mdash; 
  The main branch of the project
- **`    --files-max-size`**=_`NUMBER`_ &mdash; 
  The maximum allowed size for source code files in bytes. Files above this limit will be ignored for performance reasons. Defaults to 1 MiB
- **`    --migrations-dir`**=_`ARG`_ &mdash; 
  The directory where the migration files are stored
- **`    --after`**=_`ARG`_ &mdash; 
  Ignore any migrations before this timestamp
- **`    --line-width`**=_`ARG`_ &mdash; 
  Maximum line width before breaking. Default: 100.
- **`    --indent-size`**=_`ARG`_ &mdash; 
  Number of spaces (or tab width) for indentation. Default: 2.
- **`    --indent-style`**=_`ARG`_ &mdash; 
  Indentation style: "spaces" or "tabs". Default: "spaces".
- **`    --keyword-case`**=_`ARG`_ &mdash; 
  Keyword casing: "upper" or "lower". Default: "lower".
- **`    --constant-case`**=_`ARG`_ &mdash; 
  Constant casing (NULL, TRUE, FALSE): "upper" or "lower". Default: "lower".
- **`    --type-case`**=_`ARG`_ &mdash; 
  Data type casing (text, varchar, int): "upper" or "lower". Default: "lower".
- **`    --comma-style`**=_`ARG`_ &mdash; 
  Where a comma sits when a list breaks: "trailing" or "leading". Default: "trailing".
- **`    --logical-operator-placement`**=_`ARG`_ &mdash; 
  Where a boolean operator sits when a condition breaks: "trailing" or "leading". Default: "trailing".
- **`    --layout`**=_`ARG`_ &mdash; 
  How a statement is laid out: "fit" breaks only when a line would exceed the line width, "expanded" always breaks between clauses. Default: "fit".
- **`    --cast-style`**=_`ARG`_ &mdash; 
  How an explicit cast is spelled: "cast" for `CAST(x AS t)`, "operator" for `x::t`. Default: "cast".
- **`    --clause-body-style`**=_`ARG`_ &mdash; 
  Where the body of a clause starts: "break" for a new line, "compact" to keep the first element on the keyword line. Default: "break".
- **`    --isolate-semicolon`**=_`ARG`_ &mdash; 
  If `true`, the terminating semicolon goes on its own line when the statement spans several lines. Default: `false`.
- **`    --skip-fn-bodies`**=_`ARG`_ &mdash; 
  If `true`, skip formatting of SQL function bodies (keep them verbatim). Default: `false`.
- **`    --search_path`**=_`ARG`_ &mdash; 
  Default search path schemas for type checking. Can be a list of schema names or glob patterns like ["public", "app_*"]. If not specified, defaults to ["public"].
- **`    --connection-string`**=_`ARG`_ &mdash; 
  A connection string that encodes the full connection setup. When provided, it takes precedence over the individual fields. Can also be set via the `DATABASE_URL` environment variable.
- **`    --host`**=_`ARG`_ &mdash; 
  The host of the database. Required if you want database-related features. All else falls back to sensible defaults. Can also be set via the `PGHOST` environment variable.
- **`    --port`**=_`ARG`_ &mdash; 
  The port of the database. Can also be set via the `PGPORT` environment variable.
- **`    --username`**=_`ARG`_ &mdash; 
  The username to connect to the database. Can also be set via the `PGUSER` environment variable.
- **`    --password`**=_`ARG`_ &mdash; 
  The password to connect to the database. Can also be set via the `PGPASSWORD` environment variable.
- **`    --database`**=_`ARG`_ &mdash; 
  The name of the database. Can also be set via the `PGDATABASE` environment variable.
- **`    --allow_statement_executions_against`**=_`ARG`_
- **`    --conn_timeout_secs`**=_`ARG`_ &mdash; 
  The connection timeout in seconds.
   
  [default: Some(10)]
- **`    --disable-db`** &mdash; 
  Actively disable all database-related features.



**Global options applied to all commands**
- **`    --colors`**=_`<off|force>`_ &mdash; 
  Set the formatting mode for markup: "off" prints everything as plain text, "force" forces the formatting of markup using ANSI even if the console output is determined to be incompatible
- **`    --use-server`** &mdash; 
  Connect to a running instance of the daemon server.
- **`    --verbose`** &mdash; 
  Print additional diagnostics, and some diagnostics show more information. Also, print out what files were processed and which ones were modified.
- **`    --config-path`**=_`PATH`_ &mdash; 
  Set the file path to the configuration file, or the directory path to find `postgres-language-server.jsonc`. If used, it disables the default configuration file resolution.
- **`    --max-diagnostics`**=_`<none|<NUMBER>>`_ &mdash; 
  Cap the amount of diagnostics displayed. When `none` is provided, the limit is lifted.
   
  [default: 20]
- **`    --skip-errors`** &mdash; 
  Skip over files containing syntax errors instead of emitting an error diagnostic.
- **`    --no-errors-on-unmatched`** &mdash; 
  Silence errors that would be emitted in case no files were processed during the execution of the command.
- **`    --error-on-warnings`** &mdash; 
  Tell Postgres Language Server to exit with an error code if some diagnostics emit warnings.
- **`    --reporter`**=_`<json|json-pretty|github|junit|summary|gitlab>`_ &mdash; 
  Allows to change how diagnostics and summary are reported.
- **`    --log-level`**=_`<none|debug|info|warn|error>`_ &mdash; 
  The level of logging. In order, from the most verbose to the least verbose: debug, info, warn, error.

  The value `none` won't show any logging.
   
  Uses environment variable **`PGLS_LOG_LEVEL`**
   
  [default: none]
- **`    --log-kind`**=_`<pretty|compact|json>`_ &mdash; 
  How the log should look like.
   
  [default: pretty]
- **`    --diagnostic-level`**=_`<info|warn|error>`_ &mdash; 
  The level of diagnostics to show. In order, from the lowest to the most important: info, warn, error. Passing `--diagnostic-level=error` will cause Postgres Tools to print only diagnostics that contain only errors.
   
  [default: info]



**Available positional items:**
- _`PATH`_ &mdash; 
  Single file, single path or list of paths



**Available options:**
- **`    --write`** &mdash; 
  Write formatted output back to files
- **`    --staged`** &mdash; 
  When set to true, only the files that have been staged (the ones prepared to be committed) will be formatted. This option should be used when working locally.
- **`    --changed`** &mdash; 
  When set to true, only the files that have been changed compared to your `defaultBranch` configuration will be formatted. This option should be used in CI environments.
- **`    --since`**=_`REF`_ &mdash; 
  Use this to specify the base branch to compare against when you're using the --changed flag and the `defaultBranch` is not set in your `postgres-language-server.jsonc`
- **`-h`**, **`--help`** &mdash; 
  Prints help information


## postgres-language-server start

Starts the daemon server process.

**Usage**: **`postgres-language-server`** **`start`** \[**`--config-path`**=_`PATH`_\]

**Available options:**
- **`    --log-prefix-name`**=_`STRING`_ &mdash; 
  Allows to change the prefix applied to the file name of the logs.
   
  Uses environment variable **`PGLS_LOG_PREFIX_NAME`**
   
  [default: server.log]
- **`    --log-path`**=_`PATH`_ &mdash; 
  Allows to change the folder where logs are stored.
   
  Uses environment variable **`PGLS_LOG_PATH`**
- **`    --config-path`**=_`PATH`_ &mdash; 
  Allows to set a custom file path to the configuration file, or a custom directory path to find `postgres-language-server.jsonc`
   
  Uses environment variable **`PGLS_CONFIG_PATH`**
- **`-h`**, **`--help`** &mdash; 
  Prints help information


## postgres-language-server stop

Stops the daemon server process.

**Usage**: **`postgres-language-server`** **`stop`** 

**Available options:**
- **`-h`**, **`--help`** &mdash; 
  Prints help information


## postgres-language-server init

Bootstraps a new project. Creates a configuration file with some defaults.

**Usage**: **`postgres-language-server`** **`init`** 

**Available options:**
- **`-h`**, **`--help`** &mdash; 
  Prints help information


## postgres-language-server lsp-proxy

Acts as a server for the Language Server Protocol over stdin/stdout.

**Usage**: **`postgres-language-server`** **`lsp-proxy`** \[**`--config-path`**=_`PATH`_\]

**Available options:**
- **`    --log-prefix-name`**=_`STRING`_ &mdash; 
  Allows to change the prefix applied to the file name of the logs.
   
  Uses environment variable **`PGLS_LOG_PREFIX_NAME`**
   
  [default: server.log]
- **`    --log-path`**=_`PATH`_ &mdash; 
  Allows to change the folder where logs are stored.
   
  Uses environment variable **`PGLS_LOG_PATH`**
- **`    --config-path`**=_`PATH`_ &mdash; 
  Allows to set a custom file path to the configuration file, or a custom directory path to find `postgres-language-server.jsonc`
   
  Uses environment variable **`PGLS_CONFIG_PATH`**
- **`-h`**, **`--help`** &mdash; 
  Prints help information


## postgres-language-server parse

Parses a SQL file (or standard input) and writes the parse tree as a protobuf `ParseResult` message.

**Usage**: **`postgres-language-server`** **`parse`** \[**`--file`**=_`PATH`_\]

**Available options:**
- **`    --file`**=_`PATH`_ &mdash; 
  Path to the SQL file to parse. When omitted, the SQL is read from standard input.
- **`    --stdin-file-path`**=_`PATH`_ &mdash; 
  Alias of --file, mirroring the format command.
- **`-h`**, **`--help`** &mdash; 
  Prints help information


## postgres-language-server schema-export

Exports the database schema to JSON for use with WASM bindings. Writes to stdout by default, or to a file if --output is specified.

**Usage**: **`postgres-language-server`** **`schema-export`** \[**`-c`**=_`URL`_\] \[**`-o`**=_`PATH`_\]

**Available options:**
- **`-c`**, **`--connection-string`**=_`URL`_ &mdash; 
  PostgreSQL connection string (e.g., postgres://user:pass@host/db). Can also be set via the `DATABASE_URL` environment variable.
- **`-o`**, **`--output`**=_`PATH`_ &mdash; 
  Output file path for the JSON schema (defaults to stdout if not specified)
- **`-h`**, **`--help`** &mdash; 
  Prints help information



[//]: # "END CLI_REF"
