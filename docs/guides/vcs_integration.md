# Integrate with VCS

The VCS (Version Control System) integration is meant to take advantage of additional features that only a VCS can provide. The integration is opt-in. You have to enable `vcs.enabled` and set `vcs.clientKind` in the configuration file:

    <!-- /// Whether we should integrate itself with the VCS client -->
    <!-- #[partial(bpaf(long("vcs-enabled"), argument("true|false")))] -->
    <!-- pub enabled: bool, -->
    <!---->
    <!-- /// The kind of client. -->
    <!-- #[partial(bpaf(long("vcs-client-kind"), argument("git"), optional))] -->
    <!-- #[partial(deserializable(bail_on_error))] -->
    <!-- pub client_kind: VcsClientKind, -->
    <!---->
    <!-- /// Whether we should use the VCS ignore file. When [true], we will ignore the files -->
    <!-- /// specified in the ignore file. -->
    <!-- #[partial(bpaf(long("vcs-use-ignore-file"), argument("true|false")))] -->
    <!-- pub use_ignore_file: bool, -->
    <!---->
    <!-- /// The folder where we should check for VCS files. By default, we will use the same -->
    <!-- /// folder where `postgrestools.jsonc` was found. -->
    <!-- /// -->
    <!-- /// If we can't find the configuration, it will attempt to use the current working directory. -->
    <!-- /// If no current working directory can't be found, we won't use the VCS integration, and a diagnostic -->
    <!-- /// will be emitted -->
    <!-- #[partial(bpaf(long("vcs-root"), argument("PATH"), optional))] -->
    <!-- pub root: String, -->
    <!---->
    <!-- /// The main branch of the project -->
    <!-- #[partial(bpaf(long("vcs-default-branch"), argument("BRANCH"), optional))] -->
    <!-- pub default_branch: String, -->


```postgrestools.jsonc
biome.json
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


```postgrestools.jsonc
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

```postgrestools.jsonc
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

```
postgrestools check --changed
```

Alternatively, you can use the option `--since` to specify an arbitrary branch. This option takes precedence over the option `vcs.defaultBranch`. For example, you might want to check your changes against the `next` branch:

```postgrestools.jsonc
postgrestools check --changed --since=next
```

## Process only staged files

Before committing your changes, you may want to check the files that have been added to the index, also known as staged files. Add the `--staged` option to process only those files:

```postgrestools.jsonc
postgrestools check --staged
```

