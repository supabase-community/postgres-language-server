# Integrate with VCS

The VCS (Version Control System) integration is meant to take advantage of additional features that only a VCS can provide. These features include ignoring files based on your VCS ignore patterns, checking only changed files, and checking staged files before commits.

The integration is opt-in. You have to enable `vcs.enabled` and set `vcs.clientKind` in the configuration file:


```json
{
  "vcs": {
    "enabled": true,
    "clientKind": "git"
  }
}
```

This configuration doesn’t do anything per se. You need to opt-in the features you want.

## Use the ignore file

Enable `vcs.useIgnoreFile`, to to ignore all the files and directories listed in the project’s VCS ignore file as well as a `.ignore` file.


```json
{
  "vcs": {
    "enabled": true,
    "clientKind": "git",
    "useIgnoreFile": true
  }
}
```

## Process only changed files

This is a feature that is available only via CLI, and allows processing only the files that have changed from one revision to another.
First, you have to update your configuration file with the default branch via the `vcs.defaultBranch` field:

```json
{
  "vcs": {
    "enabled": true,
    "clientKind": "git",
    "useIgnoreFile": true,
    "defaultBranch": "main"
  }
}
```

Add the `--changed` option to your command to process only those files that your VCS acknowledged as “changed”. The language server will determine the changed files from the branch `main` and your current revision:

```shell
postgres-language-server check --changed
```

Alternatively, you can use the option `--since` to specify an arbitrary branch. This option takes precedence over the option `vcs.defaultBranch`. For example, you might want to check your changes against the `next` branch:

```shell
postgres-language-server check --changed --since=next
```

## Process only staged files

Before committing your changes, you may want to check the files that have been added to the index, also known as staged files. Add the `--staged` option to process only those files:

```shell
postgres-language-server check --staged
```

