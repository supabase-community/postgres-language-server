# Upgrading to 0.27

Version 0.27 gives every linter rule a flat name and adds static type checking. Existing configurations and suppression comments keep working, but print deprecation warnings. This guide shows how to move to the new forms.

## Rule names

Rules are identified by their name alone. The group is no longer part of it.

| Where | Before | Now |
| --- | --- | --- |
| Configuration | `linter.rules.safety.banDropColumn` | `linter.rules.banDropColumn` |
| Suppression comments | `-- pgls-ignore lint/safety/banDropColumn` | `-- pgls-ignore banDropColumn` |
| Diagnostic category | `lint/safety/banDropColumn` | `lint/banDropColumn` |

The rules used to be in a single `safety` group. They are now grouped into `correctness`, `safety`, `destructive`, `style`, `typecheck`, and `nursery`. See the [rules reference](../reference/rules.md) for the group of each rule.

## Configuration

Move the rules out of `linter.rules.safety`:

```jsonc
// before
{
  "linter": {
    "rules": {
      "safety": { "banDropColumn": "off", "banDropTable": "warn" }
    }
  }
}

// now
{
  "linter": {
    "rules": { "banDropColumn": "off", "banDropTable": "warn" }
  }
}
```

Groups are configured separately, under `linter.groups`:

```json
{
  "linter": {
    "groups": { "style": "off", "destructive": "error" }
  }
}
```

`linter.rules.safety.recommended` and `linter.rules.safety.all` become `linter.rules.recommended` and `linter.rules.all`. See [Configuration](../configuration.md#precedence) for the order in which these settings apply.

The CLI prints a `Warning:` line for each deprecated setting, and the editor shows them once per session.

## Removed rules

| Rule | Replacement |
| --- | --- |
| `preferBigintOverInt` | `preferBigInt` with `{ "checkInt": true }` |
| `preferBigintOverSmallint` | `preferBigInt` with `{ "checkSmallint": true }` |
| `concurrentRefreshMatviewLock` | `requireConcurrentRefreshMatview` |

Suppression comments that name a removed rule suppress its replacement.

## Suppression comments

`lint/<group>/<rule>` and `lint/<group>` still work, and report a deprecation warning that names the new form. `lint/safety` keeps its old meaning of every lint rule. See [Suppressions](suppressions.md).

## What changes without a configuration change

- **Diagnostic categories** are `lint/<rule>`. Scripts that match on the category in the JSON, GitHub, GitLab, or JUnit reporter output, or on the diagnostic code in the editor, need updating.
- **`linter.enabled: false`** now turns the linter off. It used to be ignored.
- **`migrations.migrationsDir`**: when it is set, rules that only apply to migrations skip files outside it.
- **Type checking**: the new [`typecheck` rules](../features/type_checking.md) are recommended. With a database connection, they report errors that the language server didn't report before. Turn them off with `"groups": { "typecheck": "off" }`.
