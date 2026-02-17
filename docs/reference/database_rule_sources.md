# Database Linter Rule Sources

Many database linter rules are inspired by or directly ported from other tools. This page lists the sources of each rule.

## Exclusive rules

_No exclusive rules available._

## Rules from other sources

[//]: # (BEGIN DATABASE_RULE_SOURCES)

### Splinter

| Splinter Rule Name | Rule Name |
| ---- | ---- |
| [authRlsInitplan](https://github.com/supabase/splinter) | [authRlsInitplan](./rules/auth-rls-initplan.md) |
| [duplicateIndex](https://github.com/supabase/splinter) | [duplicateIndex](./rules/duplicate-index.md) |
| [multiplePermissivePolicies](https://github.com/supabase/splinter) | [multiplePermissivePolicies](./rules/multiple-permissive-policies.md) |
| [noPrimaryKey](https://github.com/supabase/splinter) | [noPrimaryKey](./rules/no-primary-key.md) |
| [tableBloat](https://github.com/supabase/splinter) | [tableBloat](./rules/table-bloat.md) |
| [unindexedForeignKeys](https://github.com/supabase/splinter) | [unindexedForeignKeys](./rules/unindexed-foreign-keys.md) |
| [unusedIndex](https://github.com/supabase/splinter) | [unusedIndex](./rules/unused-index.md) |
| [authUsersExposed](https://github.com/supabase/splinter) | [authUsersExposed](./rules/auth-users-exposed.md) |
| [extensionInPublic](https://github.com/supabase/splinter) | [extensionInPublic](./rules/extension-in-public.md) |
| [extensionVersionsOutdated](https://github.com/supabase/splinter) | [extensionVersionsOutdated](./rules/extension-versions-outdated.md) |
| [fkeyToAuthUnique](https://github.com/supabase/splinter) | [fkeyToAuthUnique](./rules/fkey-to-auth-unique.md) |
| [foreignTableInApi](https://github.com/supabase/splinter) | [foreignTableInApi](./rules/foreign-table-in-api.md) |
| [functionSearchPathMutable](https://github.com/supabase/splinter) | [functionSearchPathMutable](./rules/function-search-path-mutable.md) |
| [insecureQueueExposedInApi](https://github.com/supabase/splinter) | [insecureQueueExposedInApi](./rules/insecure-queue-exposed-in-api.md) |
| [materializedViewInApi](https://github.com/supabase/splinter) | [materializedViewInApi](./rules/materialized-view-in-api.md) |
| [policyExistsRlsDisabled](https://github.com/supabase/splinter) | [policyExistsRlsDisabled](./rules/policy-exists-rls-disabled.md) |
| [rlsDisabledInPublic](https://github.com/supabase/splinter) | [rlsDisabledInPublic](./rules/rls-disabled-in-public.md) |
| [rlsEnabledNoPolicy](https://github.com/supabase/splinter) | [rlsEnabledNoPolicy](./rules/rls-enabled-no-policy.md) |
| [rlsPolicyAlwaysTrue](https://github.com/supabase/splinter) | [rlsPolicyAlwaysTrue](./rules/rls-policy-always-true.md) |
| [rlsReferencesUserMetadata](https://github.com/supabase/splinter) | [rlsReferencesUserMetadata](./rules/rls-references-user-metadata.md) |
| [securityDefinerView](https://github.com/supabase/splinter) | [securityDefinerView](./rules/security-definer-view.md) |
| [sensitiveColumnsExposed](https://github.com/supabase/splinter) | [sensitiveColumnsExposed](./rules/sensitive-columns-exposed.md) |
| [unsupportedRegTypes](https://github.com/supabase/splinter) | [unsupportedRegTypes](./rules/unsupported-reg-types.md) |

[//]: # (END DATABASE_RULE_SOURCES)
