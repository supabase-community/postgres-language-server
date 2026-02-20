# Database Linter Rules

Below is the list of database linting rules supported by the Postgres Language Server, divided by group. These rules analyze your live database schema to detect issues.

All rules are powered by [Splinter](https://github.com/supabase/splinter).

Here's a legend of the emojis:

- The icon ✅ indicates that the rule is part of the recommended rules.
- The icon ⚡ indicates that the rule requires a Supabase database.

[//]: # (BEGIN SPLINTER_RULES_INDEX)

## Performance

Rules that detect potential performance issues in your database schema.

| Rule name | Description | Properties |
| --- | --- | --- |
| [authRlsInitplan](./rules/auth-rls-initplan.md) | Detects if calls to \`current_setting()\` and \`auth.<function>()\` in RLS policies are being unnecessarily re-evaluated for each row | ✅ ⚡ |
| [duplicateIndex](./rules/duplicate-index.md) | Detects cases where two ore more identical indexes exist. | ✅  |
| [multiplePermissivePolicies](./rules/multiple-permissive-policies.md) | Detects if multiple permissive row level security policies are present on a table for the same \`role\` and \`action\` (e.g. insert). Multiple permissive policies are suboptimal for performance as each policy must be executed for every relevant query. | ✅  |
| [noPrimaryKey](./rules/no-primary-key.md) | Detects if a table does not have a primary key. Tables without a primary key can be inefficient to interact with at scale. | ✅  |
| [tableBloat](./rules/table-bloat.md) | Detects if a table has excess bloat and may benefit from maintenance operations like vacuum full or cluster. | ✅  |
| [unindexedForeignKeys](./rules/unindexed-foreign-keys.md) | Identifies foreign key constraints without a covering index, which can impact database performance. | ✅  |
| [unusedIndex](./rules/unused-index.md) | Detects if an index has never been used and may be a candidate for removal. | ✅  |

## Security

Rules that detect potential security vulnerabilities in your database schema.

| Rule name | Description | Properties |
| --- | --- | --- |
| [authUsersExposed](./rules/auth-users-exposed.md) | Detects if auth.users is exposed to anon or authenticated roles via a view or materialized view in schemas exposed to PostgREST, potentially compromising user data security. | ✅ ⚡ |
| [extensionInPublic](./rules/extension-in-public.md) | Detects extensions installed in the \`public\` schema. | ✅  |
| [extensionVersionsOutdated](./rules/extension-versions-outdated.md) | Detects extensions that are not using the default (recommended) version. | ✅  |
| [fkeyToAuthUnique](./rules/fkey-to-auth-unique.md) | Detects user defined foreign keys to unique constraints in the auth schema. | ✅ ⚡ |
| [foreignTableInApi](./rules/foreign-table-in-api.md) | Detects foreign tables that are accessible over APIs. Foreign tables do not respect row level security policies. | ✅ ⚡ |
| [functionSearchPathMutable](./rules/function-search-path-mutable.md) | Detects functions where the search_path parameter is not set. | ✅  |
| [insecureQueueExposedInApi](./rules/insecure-queue-exposed-in-api.md) | Detects cases where an insecure Queue is exposed over Data APIs | ✅ ⚡ |
| [materializedViewInApi](./rules/materialized-view-in-api.md) | Detects materialized views that are accessible over the Data APIs. | ✅ ⚡ |
| [policyExistsRlsDisabled](./rules/policy-exists-rls-disabled.md) | Detects cases where row level security (RLS) policies have been created, but RLS has not been enabled for the underlying table. | ✅  |
| [rlsDisabledInPublic](./rules/rls-disabled-in-public.md) | Detects cases where row level security (RLS) has not been enabled on tables in schemas exposed to PostgREST | ✅ ⚡ |
| [rlsEnabledNoPolicy](./rules/rls-enabled-no-policy.md) | Detects cases where row level security (RLS) has been enabled on a table but no RLS policies have been created. | ✅  |
| [rlsPolicyAlwaysTrue](./rules/rls-policy-always-true.md) | Detects RLS policies that use overly permissive expressions like USING (true) or WITH CHECK (true) for UPDATE, DELETE, or INSERT operations. SELECT policies with USING (true) are intentionally excluded as this pattern is often used deliberately for public read access. | ✅ ⚡ |
| [rlsReferencesUserMetadata](./rules/rls-references-user-metadata.md) | Detects when Supabase Auth user_metadata is referenced insecurely in a row level security (RLS) policy. | ✅ ⚡ |
| [securityDefinerView](./rules/security-definer-view.md) | Detects views defined with the SECURITY DEFINER property. These views enforce Postgres permissions and row level security policies (RLS) of the view creator, rather than that of the querying user | ✅ ⚡ |
| [sensitiveColumnsExposed](./rules/sensitive-columns-exposed.md) | Detects tables exposed via API that contain columns with potentially sensitive data (PII, credentials, financial info) without RLS protection. | ✅ ⚡ |
| [unsupportedRegTypes](./rules/unsupported-reg-types.md) | Identifies columns using unsupported reg* types outside pg_catalog schema, which prevents database upgrades using pg_upgrade. | ✅  |

[//]: # (END SPLINTER_RULES_INDEX)
