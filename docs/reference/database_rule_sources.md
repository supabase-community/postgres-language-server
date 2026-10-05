# Database Linter Rule Sources

Many database linter rules are inspired by or directly ported from other tools. This page lists the sources of each rule.

## Exclusive rules

_No exclusive rules available._

## Rules from other sources

[//]: # (BEGIN DATABASE_RULE_SOURCES)

### Splinter

| Splinter Rule Name | Rule Name |
| ---- | ---- |
| [authRlsInitplan](https://github.com/supabase/splinter) | [authRlsInitplan](./database-rules/auth-rls-initplan.md) |
| [duplicateIndex](https://github.com/supabase/splinter) | [duplicateIndex](./database-rules/duplicate-index.md) |
| [multiplePermissivePolicies](https://github.com/supabase/splinter) | [multiplePermissivePolicies](./database-rules/multiple-permissive-policies.md) |
| [noPrimaryKey](https://github.com/supabase/splinter) | [noPrimaryKey](./database-rules/no-primary-key.md) |
| [tableBloat](https://github.com/supabase/splinter) | [tableBloat](./database-rules/table-bloat.md) |
| [unindexedForeignKeys](https://github.com/supabase/splinter) | [unindexedForeignKeys](./database-rules/unindexed-foreign-keys.md) |
| [unusedIndex](https://github.com/supabase/splinter) | [unusedIndex](./database-rules/unused-index.md) |
| [authUsersExposed](https://github.com/supabase/splinter) | [authUsersExposed](./database-rules/auth-users-exposed.md) |
| [extensionInPublic](https://github.com/supabase/splinter) | [extensionInPublic](./database-rules/extension-in-public.md) |
| [extensionVersionsOutdated](https://github.com/supabase/splinter) | [extensionVersionsOutdated](./database-rules/extension-versions-outdated.md) |
| [fkeyToAuthUnique](https://github.com/supabase/splinter) | [fkeyToAuthUnique](./database-rules/fkey-to-auth-unique.md) |
| [foreignTableInApi](https://github.com/supabase/splinter) | [foreignTableInApi](./database-rules/foreign-table-in-api.md) |
| [functionSearchPathMutable](https://github.com/supabase/splinter) | [functionSearchPathMutable](./database-rules/function-search-path-mutable.md) |
| [insecureQueueExposedInApi](https://github.com/supabase/splinter) | [insecureQueueExposedInApi](./database-rules/insecure-queue-exposed-in-api.md) |
| [materializedViewInApi](https://github.com/supabase/splinter) | [materializedViewInApi](./database-rules/materialized-view-in-api.md) |
| [policyExistsRlsDisabled](https://github.com/supabase/splinter) | [policyExistsRlsDisabled](./database-rules/policy-exists-rls-disabled.md) |
| [rlsDisabledInPublic](https://github.com/supabase/splinter) | [rlsDisabledInPublic](./database-rules/rls-disabled-in-public.md) |
| [rlsEnabledNoPolicy](https://github.com/supabase/splinter) | [rlsEnabledNoPolicy](./database-rules/rls-enabled-no-policy.md) |
| [rlsPolicyAlwaysTrue](https://github.com/supabase/splinter) | [rlsPolicyAlwaysTrue](./database-rules/rls-policy-always-true.md) |
| [rlsReferencesUserMetadata](https://github.com/supabase/splinter) | [rlsReferencesUserMetadata](./database-rules/rls-references-user-metadata.md) |
| [securityDefinerView](https://github.com/supabase/splinter) | [securityDefinerView](./database-rules/security-definer-view.md) |
| [sensitiveColumnsExposed](https://github.com/supabase/splinter) | [sensitiveColumnsExposed](./database-rules/sensitive-columns-exposed.md) |
| [unsupportedRegTypes](https://github.com/supabase/splinter) | [unsupportedRegTypes](./database-rules/unsupported-reg-types.md) |

[//]: # (END DATABASE_RULE_SOURCES)
