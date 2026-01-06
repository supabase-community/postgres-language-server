//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rules::{RuleConfiguration, RulePlainConfiguration};
use biome_deserialize_macros::Merge;
use pgls_analyse::RuleFilter;
use pgls_analyser::RuleOptions;
use pgls_diagnostics::{Category, Severity};
use rustc_hash::FxHashSet;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Merge,
    Ord,
    PartialEq,
    PartialOrd,
    serde :: Deserialize,
    serde :: Serialize,
)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum RuleGroup {
    Performance,
    Security,
}
impl RuleGroup {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Performance => Performance::GROUP_NAME,
            Self::Security => Security::GROUP_NAME,
        }
    }
}
impl std::str::FromStr for RuleGroup {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            Performance::GROUP_NAME => Ok(Self::Performance),
            Security::GROUP_NAME => Ok(Self::Security),
            _ => Err("This rule group doesn't exist."),
        }
    }
}
#[derive(Clone, Debug, Default, Deserialize, Eq, Merge, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Rules {
    #[doc = r" It enables the lint rules recommended by Postgres Language Server. `true` by default."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended: Option<bool>,
    #[doc = r" It enables ALL rules. The rules that belong to `nursery` won't be enabled."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance: Option<Performance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Security>,
}
impl Rules {
    #[doc = r" Checks if the code coming from [pgls_diagnostics::Diagnostic] corresponds to a rule."]
    #[doc = r" Usually the code is built like {group}/{rule_name}"]
    pub fn has_rule(group: RuleGroup, rule_name: &str) -> Option<&'static str> {
        match group {
            RuleGroup::Performance => Performance::has_rule(rule_name),
            RuleGroup::Security => Security::has_rule(rule_name),
        }
    }
    #[doc = r" Given a category coming from [Diagnostic](pgls_diagnostics::Diagnostic), this function returns"]
    #[doc = r" the [Severity](pgls_diagnostics::Severity) associated to the rule, if the configuration changed it."]
    #[doc = r" If the severity is off or not set, then the function returns the default severity of the rule,"]
    #[doc = r" which is configured at the rule definition."]
    #[doc = r" The function can return `None` if the rule is not properly configured."]
    pub fn get_severity_from_code(&self, category: &Category) -> Option<Severity> {
        let mut split_code = category.name().split('/');
        let _category_prefix = split_code.next();
        debug_assert_eq!(_category_prefix, Some("splinter"));
        let group = <RuleGroup as std::str::FromStr>::from_str(split_code.next()?).ok()?;
        let rule_name = split_code.next()?;
        let rule_name = Self::has_rule(group, rule_name)?;
        let severity = match group {
            RuleGroup::Performance => self
                .performance
                .as_ref()
                .and_then(|group| group.get_rule_configuration(rule_name))
                .filter(|(level, _)| !matches!(level, RulePlainConfiguration::Off))
                .map_or_else(
                    || Performance::severity(rule_name),
                    |(level, _)| level.into(),
                ),
            RuleGroup::Security => self
                .security
                .as_ref()
                .and_then(|group| group.get_rule_configuration(rule_name))
                .filter(|(level, _)| !matches!(level, RulePlainConfiguration::Off))
                .map_or_else(|| Security::severity(rule_name), |(level, _)| level.into()),
        };
        Some(severity)
    }
    #[doc = r" Ensure that `recommended` is set to `true` or implied."]
    pub fn set_recommended(&mut self) {
        if self.all != Some(true) && self.recommended == Some(false) {
            self.recommended = Some(true)
        }
        if let Some(group) = &mut self.performance {
            group.recommended = None;
        }
        if let Some(group) = &mut self.security {
            group.recommended = None;
        }
    }
    pub(crate) const fn is_recommended_false(&self) -> bool {
        matches!(self.recommended, Some(false))
    }
    pub(crate) const fn is_all_true(&self) -> bool {
        matches!(self.all, Some(true))
    }
    #[doc = r" It returns the enabled rules by default."]
    #[doc = r""]
    #[doc = r" The enabled rules are calculated from the difference with the disabled rules."]
    pub fn as_enabled_rules(&self) -> FxHashSet<RuleFilter<'static>> {
        let mut enabled_rules = FxHashSet::default();
        let mut disabled_rules = FxHashSet::default();
        if let Some(group) = self.performance.as_ref() {
            group.collect_preset_rules(
                self.is_all_true(),
                !self.is_recommended_false(),
                &mut enabled_rules,
            );
            enabled_rules.extend(&group.get_enabled_rules());
            disabled_rules.extend(&group.get_disabled_rules());
        } else if self.is_all_true() {
            enabled_rules.extend(Performance::all_rules_as_filters());
        } else if !self.is_recommended_false() {
            enabled_rules.extend(Performance::recommended_rules_as_filters());
        }
        if let Some(group) = self.security.as_ref() {
            group.collect_preset_rules(
                self.is_all_true(),
                !self.is_recommended_false(),
                &mut enabled_rules,
            );
            enabled_rules.extend(&group.get_enabled_rules());
            disabled_rules.extend(&group.get_disabled_rules());
        } else if self.is_all_true() {
            enabled_rules.extend(Security::all_rules_as_filters());
        } else if !self.is_recommended_false() {
            enabled_rules.extend(Security::recommended_rules_as_filters());
        }
        enabled_rules.difference(&disabled_rules).copied().collect()
    }
    #[doc = r" It returns the disabled rules by configuration."]
    pub fn as_disabled_rules(&self) -> FxHashSet<RuleFilter<'static>> {
        let mut disabled_rules = FxHashSet::default();
        if let Some(group) = self.performance.as_ref() {
            disabled_rules.extend(&group.get_disabled_rules());
        }
        if let Some(group) = self.security.as_ref() {
            disabled_rules.extend(&group.get_disabled_rules());
        }
        disabled_rules
    }
}
#[derive(Clone, Debug, Default, Deserialize, Eq, Merge, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
#[doc = r" A list of rules that belong to this group"]
pub struct Performance {
    #[doc = r" It enables the recommended rules for this group"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended: Option<bool>,
    #[doc = r" It enables ALL rules for this group."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all: Option<bool>,
    #[doc = "/// # Auth RLS Initialization Plan /// /// Detects if calls to `current_setting()` and `auth.()` in RLS policies are being unnecessarily re-evaluated for each row /// /// Note: This rule requires Supabase roles (anon, authenticated, service_role). /// It will be automatically skipped if these roles don't exist in your database. /// /// ## SQL Query /// /// sql /// ( /// with policies as ( ///     select ///         nsp.nspname as schema_name, ///         pb.tablename as table_name, ///         pc.relrowsecurity as is_rls_active, ///         polname as policy_name, ///         polpermissive as is_permissive, -- if not, then restrictive ///         (select array_agg(r::regrole) from unnest(polroles) as x(r)) as roles, ///         case polcmd ///             when 'r' then 'SELECT' ///             when 'a' then 'INSERT' ///             when 'w' then 'UPDATE' ///             when 'd' then 'DELETE' ///             when '*' then 'ALL' ///         end as command, ///         qual, ///         with_check ///     from ///         pg_catalog.pg_policy pa ///         join pg_catalog.pg_class pc ///             on pa.polrelid = pc.oid ///         join pg_catalog.pg_namespace nsp ///             on pc.relnamespace = nsp.oid ///         join pg_catalog.pg_policies pb ///             on pc.relname = pb.tablename ///             and nsp.nspname = pb.schemaname ///             and pa.polname = pb.policyname /// ) /// select ///     'auth_rls_initplan' as \"name!\", ///     'Auth RLS Initialization Plan' as \"title!\", ///     'WARN' as \"level!\", ///     'EXTERNAL' as \"facing!\", ///     array\\['PERFORMANCE'] as \"categories!\", ///     'Detects if calls to \\`current_setting()\\` and \\`auth.\\<function>()\\` in RLS policies are being unnecessarily re-evaluated for each row' as \"description!\", ///     format( ///         'Table \\`%s.%s\\` has a row level security policy \\`%s\\` that re-evaluates current_setting() or auth.\\<function>() for each row. This produces suboptimal query performance at scale. Resolve the issue by replacing \\`auth.\\<function>()\\` with \\`(select auth.\\<function>())\\`. See \\[docs](https://supabase.com/docs/guides/database/postgres/row-level-security#call-functions-with-select) for more info.', ///         schema_name, ///         table_name, ///         policy_name ///     ) as \"detail!\", ///     'https://supabase.com/docs/guides/database/database-linter?lint=0003_auth_rls_initplan' as \"remediation!\", ///     jsonb_build_object( ///         'schema', schema_name, ///         'name', table_name, ///         'type', 'table' ///     ) as \"metadata!\", ///     format('auth_rls_init_plan_%s_%s_%s', schema_name, table_name, policy_name) as \"cache_key!\" /// from ///     policies /// where ///     is_rls_active ///     -- NOTE: does not include realtime in support of monitoring policies on realtime.messages ///     and schema_name not in ( ///         '_timescaledb_cache', '_timescaledb_catalog', '_timescaledb_config', '_timescaledb_internal', 'auth', 'cron', 'extensions', 'graphql', 'graphql_public', 'information_schema', 'net', 'pgmq', 'pgroonga', 'pgsodium', 'pgsodium_masks', 'pgtle', 'pgbouncer', 'pg_catalog', 'pgtle', 'repack', 'storage', 'supabase_functions', 'supabase_migrations', 'tiger', 'topology', 'vault' ///     ) ///     and ( ///         -- Example: auth.uid() ///         ( ///             qual like '%auth.uid()%' ///             and lower(qual) not like '%select auth.uid()%' ///         ) ///         or ( ///             qual like '%auth.jwt()%' ///             and lower(qual) not like '%select auth.jwt()%' ///         ) ///         or ( ///             qual like '%auth.role()%' ///             and lower(qual) not like '%select auth.role()%' ///         ) ///         or ( ///             qual like '%auth.email()%' ///             and lower(qual) not like '%select auth.email()%' ///         ) ///         or ( ///             qual like '%current\\_setting(%)%' ///             and lower(qual) not like '%select current\\_setting(%)%' ///         ) ///         or ( ///             with_check like '%auth.uid()%' ///             and lower(with_check) not like '%select auth.uid()%' ///         ) ///         or ( ///             with_check like '%auth.jwt()%' ///             and lower(with_check) not like '%select auth.jwt()%' ///         ) ///         or ( ///             with_check like '%auth.role()%' ///             and lower(with_check) not like '%select auth.role()%' ///         ) ///         or ( ///             with_check like '%auth.email()%' ///             and lower(with_check) not like '%select auth.email()%' ///         ) ///         or ( ///             with_check like '%current\\_setting(%)%' ///             and lower(with_check) not like '%select current\\_setting(%)%' ///         ) ///     )) ///  /// /// ## Configuration /// /// Enable or disable this rule in your configuration: /// /// json /// { ///   \"splinter\": { ///     \"rules\": { ///       \"performance\": { ///         \"authRlsInitplan\": \"warn\" ///       } ///     } ///   } /// } ///  /// /// ## Remediation /// /// See: https://supabase.com/docs/guides/database/database-linter?lint=0003_auth_rls_initplan"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_rls_initplan: Option<RuleConfiguration<()>>,
    #[doc = "/// # Duplicate Index /// /// Detects cases where two ore more identical indexes exist. /// /// ## SQL Query /// /// sql /// ( /// select ///     'duplicate_index' as \"name!\", ///     'Duplicate Index' as \"title!\", ///     'WARN' as \"level!\", ///     'EXTERNAL' as \"facing!\", ///     array\\['PERFORMANCE'] as \"categories!\", ///     'Detects cases where two ore more identical indexes exist.' as \"description!\", ///     format( ///         'Table \\`%s.%s\\` has identical indexes %s. Drop all except one of them', ///         n.nspname, ///         c.relname, ///         array_agg(pi.indexname order by pi.indexname) ///     ) as \"detail!\", ///     'https://supabase.com/docs/guides/database/database-linter?lint=0009_duplicate_index' as \"remediation!\", ///     jsonb_build_object( ///         'schema', n.nspname, ///         'name', c.relname, ///         'type', case ///             when c.relkind = 'r' then 'table' ///             when c.relkind = 'm' then 'materialized view' ///             else 'ERROR' ///         end, ///         'indexes', array_agg(pi.indexname order by pi.indexname) ///     ) as \"metadata!\", ///     format( ///         'duplicate_index_%s_%s_%s', ///         n.nspname, ///         c.relname, ///         array_agg(pi.indexname order by pi.indexname) ///     ) as \"cache_key!\" /// from ///     pg_catalog.pg_indexes pi ///     join pg_catalog.pg_namespace n ///         on n.nspname  = pi.schemaname ///     join pg_catalog.pg_class c ///         on pi.tablename = c.relname ///         and n.oid = c.relnamespace ///     left join pg_catalog.pg_depend dep ///         on c.oid = dep.objid ///         and dep.deptype = 'e' /// where ///     c.relkind in ('r', 'm') -- tables and materialized views ///     and n.nspname not in ( ///         '_timescaledb_cache', '_timescaledb_catalog', '_timescaledb_config', '_timescaledb_internal', 'auth', 'cron', 'extensions', 'graphql', 'graphql_public', 'information_schema', 'net', 'pgmq', 'pgroonga', 'pgsodium', 'pgsodium_masks', 'pgtle', 'pgbouncer', 'pg_catalog', 'pgtle', 'realtime', 'repack', 'storage', 'supabase_functions', 'supabase_migrations', 'tiger', 'topology', 'vault' ///     ) ///     and dep.objid is null -- exclude tables owned by extensions /// group by ///     n.nspname, ///     c.relkind, ///     c.relname, ///     replace(pi.indexdef, pi.indexname, '') /// having ///     count(*) > 1) ///  /// /// ## Configuration /// /// Enable or disable this rule in your configuration: /// /// json /// { ///   \"splinter\": { ///     \"rules\": { ///       \"performance\": { ///         \"duplicateIndex\": \"warn\" ///       } ///     } ///   } /// } ///  /// /// ## Remediation /// /// See: https://supabase.com/docs/guides/database/database-linter?lint=0009_duplicate_index"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duplicate_index: Option<RuleConfiguration<()>>,
    #[doc = "/// # Multiple Permissive Policies /// /// Detects if multiple permissive row level security policies are present on a table for the same `role` and `action` (e.g. insert). Multiple permissive policies are suboptimal for performance as each policy must be executed for every relevant query. /// /// ## SQL Query /// /// sql /// ( /// select ///     'multiple_permissive_policies' as \"name!\", ///     'Multiple Permissive Policies' as \"title!\", ///     'WARN' as \"level!\", ///     'EXTERNAL' as \"facing!\", ///     array\\['PERFORMANCE'] as \"categories!\", ///     'Detects if multiple permissive row level security policies are present on a table for the same \\`role\\` and \\`action\\` (e.g. insert). Multiple permissive policies are suboptimal for performance as each policy must be executed for every relevant query.' as \"description!\", ///     format( ///         'Table \\`%s.%s\\` has multiple permissive policies for role \\`%s\\` for action \\`%s\\`. Policies include \\`%s\\`', ///         n.nspname, ///         c.relname, ///         r.rolname, ///         act.cmd, ///         array_agg(p.polname order by p.polname) ///     ) as \"detail!\", ///     'https://supabase.com/docs/guides/database/database-linter?lint=0006_multiple_permissive_policies' as \"remediation!\", ///     jsonb_build_object( ///         'schema', n.nspname, ///         'name', c.relname, ///         'type', 'table' ///     ) as \"metadata!\", ///     format( ///         'multiple_permissive_policies_%s_%s_%s_%s', ///         n.nspname, ///         c.relname, ///         r.rolname, ///         act.cmd ///     ) as \"cache_key!\" /// from ///     pg_catalog.pg_policy p ///     join pg_catalog.pg_class c ///         on p.polrelid = c.oid ///     join pg_catalog.pg_namespace n ///         on c.relnamespace = n.oid ///     join pg_catalog.pg_roles r ///         on p.polroles @> array\\[r.oid] ///         or p.polroles = array\\[0::oid] ///     left join pg_catalog.pg_depend dep ///         on c.oid = dep.objid ///         and dep.deptype = 'e', ///     lateral ( ///         select x.cmd ///         from unnest(( ///             select ///                 case p.polcmd ///                     when 'r' then array\\['SELECT'] ///                     when 'a' then array\\['INSERT'] ///                     when 'w' then array\\['UPDATE'] ///                     when 'd' then array\\['DELETE'] ///                     when '*' then array\\['SELECT', 'INSERT', 'UPDATE', 'DELETE'] ///                     else array\\['ERROR'] ///                 end as actions ///         )) x(cmd) ///     ) act(cmd) /// where ///     c.relkind = 'r' -- regular tables ///     and p.polpermissive -- policy is permissive ///     and n.nspname not in ( ///         '_timescaledb_cache', '_timescaledb_catalog', '_timescaledb_config', '_timescaledb_internal', 'auth', 'cron', 'extensions', 'graphql', 'graphql_public', 'information_schema', 'net', 'pgmq', 'pgroonga', 'pgsodium', 'pgsodium_masks', 'pgtle', 'pgbouncer', 'pg_catalog', 'pgtle', 'realtime', 'repack', 'storage', 'supabase_functions', 'supabase_migrations', 'tiger', 'topology', 'vault' ///     ) ///     and r.rolname not like 'pg_%' ///     and r.rolname not like 'supabase%admin' ///     and not r.rolbypassrls ///     and dep.objid is null -- exclude tables owned by extensions /// group by ///     n.nspname, ///     c.relname, ///     r.rolname, ///     act.cmd /// having ///     count(1) > 1) ///  /// /// ## Configuration /// /// Enable or disable this rule in your configuration: /// /// json /// { ///   \"splinter\": { ///     \"rules\": { ///       \"performance\": { ///         \"multiplePermissivePolicies\": \"warn\" ///       } ///     } ///   } /// } ///  /// /// ## Remediation /// /// See: https://supabase.com/docs/guides/database/database-linter?lint=0006_multiple_permissive_policies"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple_permissive_policies: Option<RuleConfiguration<()>>,
    #[doc = "/// # No Primary Key /// /// Detects if a table does not have a primary key. Tables without a primary key can be inefficient to interact with at scale. /// /// ## SQL Query /// /// sql /// ( /// select ///     'no_primary_key' as \"name!\", ///     'No Primary Key' as \"title!\", ///     'INFO' as \"level!\", ///     'EXTERNAL' as \"facing!\", ///     array\\['PERFORMANCE'] as \"categories!\", ///     'Detects if a table does not have a primary key. Tables without a primary key can be inefficient to interact with at scale.' as \"description!\", ///     format( ///         'Table \\`%s.%s\\` does not have a primary key', ///         pgns.nspname, ///         pgc.relname ///     ) as \"detail!\", ///     'https://supabase.com/docs/guides/database/database-linter?lint=0004_no_primary_key' as \"remediation!\", ///      jsonb_build_object( ///         'schema', pgns.nspname, ///         'name', pgc.relname, ///         'type', 'table' ///     ) as \"metadata!\", ///     format( ///         'no_primary_key_%s_%s', ///         pgns.nspname, ///         pgc.relname ///     ) as \"cache_key!\" /// from ///     pg_catalog.pg_class pgc ///     join pg_catalog.pg_namespace pgns ///         on pgns.oid = pgc.relnamespace ///     left join pg_catalog.pg_index pgi ///         on pgi.indrelid = pgc.oid ///     left join pg_catalog.pg_depend dep ///         on pgc.oid = dep.objid ///         and dep.deptype = 'e' /// where ///     pgc.relkind = 'r' -- regular tables ///     and pgns.nspname not in ( ///         '_timescaledb_cache', '_timescaledb_catalog', '_timescaledb_config', '_timescaledb_internal', 'auth', 'cron', 'extensions', 'graphql', 'graphql_public', 'information_schema', 'net', 'pgmq', 'pgroonga', 'pgsodium', 'pgsodium_masks', 'pgtle', 'pgbouncer', 'pg_catalog', 'pgtle', 'realtime', 'repack', 'storage', 'supabase_functions', 'supabase_migrations', 'tiger', 'topology', 'vault' ///     ) ///     and dep.objid is null -- exclude tables owned by extensions /// group by ///     pgc.oid, ///     pgns.nspname, ///     pgc.relname /// having ///     max(coalesce(pgi.indisprimary, false)::int) = 0) ///  /// /// ## Configuration /// /// Enable or disable this rule in your configuration: /// /// json /// { ///   \"splinter\": { ///     \"rules\": { ///       \"performance\": { ///         \"noPrimaryKey\": \"warn\" ///       } ///     } ///   } /// } ///  /// /// ## Remediation /// /// See: https://supabase.com/docs/guides/database/database-linter?lint=0004_no_primary_key"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_primary_key: Option<RuleConfiguration<()>>,
    #[doc = "/// # Table Bloat /// /// Detects if a table has excess bloat and may benefit from maintenance operations like vacuum full or cluster. /// /// ## SQL Query /// /// sql /// ( /// with constants as ( ///     select current_setting('block_size')::numeric as bs, 23 as hdr, 4 as ma /// ), ///  /// bloat_info as ( ///     select ///         ma, ///         bs, ///         schemaname, ///         tablename, ///         (datawidth + (hdr + ma - (case when hdr % ma = 0 then ma else hdr % ma end)))::numeric as datahdr, ///         (maxfracsum * (nullhdr + ma - (case when nullhdr % ma = 0 then ma else nullhdr % ma end))) as nullhdr2 ///     from ( ///         select ///             schemaname, ///             tablename, ///             hdr, ///             ma, ///             bs, ///             sum((1 - null_frac) * avg_width) as datawidth, ///             max(null_frac) as maxfracsum, ///             hdr + ( ///                 select 1 + count(*) / 8 ///                 from pg_stats s2 ///                 where ///                     null_frac \\<> 0 ///                     and s2.schemaname = s.schemaname ///                     and s2.tablename = s.tablename ///             ) as nullhdr ///         from pg_stats s, constants ///         group by 1, 2, 3, 4, 5 ///     ) as foo /// ), ///  /// table_bloat as ( ///     select ///         schemaname, ///         tablename, ///         cc.relpages, ///         bs, ///         ceil((cc.reltuples * ((datahdr + ma - ///           (case when datahdr % ma = 0 then ma else datahdr % ma end)) + nullhdr2 + 4)) / (bs - 20::float)) as otta ///     from ///         bloat_info ///         join pg_class cc ///             on cc.relname = bloat_info.tablename ///         join pg_namespace nn ///             on cc.relnamespace = nn.oid ///             and nn.nspname = bloat_info.schemaname ///             and nn.nspname \\<> 'information_schema' ///         where ///             cc.relkind = 'r' ///             and cc.relam = (select oid from pg_am where amname = 'heap') /// ), ///  /// bloat_data as ( ///     select ///         'table' as type, ///         schemaname, ///         tablename as object_name, ///         round(case when otta = 0 then 0.0 else table_bloat.relpages / otta::numeric end, 1) as bloat, ///         case when relpages \\< otta then 0 else (bs * (table_bloat.relpages - otta)::bigint)::bigint end as raw_waste ///     from ///         table_bloat /// ) ///  /// select ///     'table_bloat' as \"name!\", ///     'Table Bloat' as \"title!\", ///     'INFO' as \"level!\", ///     'EXTERNAL' as \"facing!\", ///     array\\['PERFORMANCE'] as \"categories!\", ///     'Detects if a table has excess bloat and may benefit from maintenance operations like vacuum full or cluster.' as \"description!\", ///     format( ///         'Table `%s`.`%s` has excessive bloat', ///         bloat_data.schemaname, ///         bloat_data.object_name ///     ) as \"detail!\", ///     'Consider running vacuum full (WARNING: incurs downtime) and tweaking autovacuum settings to reduce bloat.' as \"remediation!\", ///     jsonb_build_object( ///         'schema', bloat_data.schemaname, ///         'name', bloat_data.object_name, ///         'type', bloat_data.type ///     ) as \"metadata!\", ///     format( ///         'table_bloat_%s_%s', ///         bloat_data.schemaname, ///         bloat_data.object_name ///     ) as \"cache_key!\" /// from ///     bloat_data /// where ///     bloat > 70.0 ///     and raw_waste > (20 * 1024 * 1024) -- filter for waste > 200 MB /// order by ///     schemaname, ///     object_name) ///  /// /// ## Configuration /// /// Enable or disable this rule in your configuration: /// /// json /// { ///   \"splinter\": { ///     \"rules\": { ///       \"performance\": { ///         \"tableBloat\": \"warn\" ///       } ///     } ///   } /// } ///  /// /// ## Remediation /// /// See: <Consider running vacuum full (WARNING: incurs downtime) and tweaking autovacuum settings to reduce bloat.>"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_bloat: Option<RuleConfiguration<()>>,
    #[doc = "/// # Unindexed foreign keys /// /// Identifies foreign key constraints without a covering index, which can impact database performance. /// /// ## SQL Query /// /// sql /// with foreign_keys as ( ///     select ///         cl.relnamespace::regnamespace::text as schema_name, ///         cl.relname as table_name, ///         cl.oid as table_oid, ///         ct.conname as fkey_name, ///         ct.conkey as col_attnums ///     from ///         pg_catalog.pg_constraint ct ///         join pg_catalog.pg_class cl -- fkey owning table ///             on ct.conrelid = cl.oid ///         left join pg_catalog.pg_depend d ///             on d.objid = cl.oid ///             and d.deptype = 'e' ///     where ///         ct.contype = 'f' -- foreign key constraints ///         and d.objid is null -- exclude tables that are dependencies of extensions ///         and cl.relnamespace::regnamespace::text not in ( ///             'pg_catalog', 'information_schema', 'auth', 'storage', 'vault', 'extensions' ///         ) /// ), /// index_ as ( ///     select ///         pi.indrelid as table_oid, ///         indexrelid::regclass as index_, ///         string_to_array(indkey::text, ' ')::smallint\\[] as col_attnums ///     from ///         pg_catalog.pg_index pi ///     where ///         indisvalid /// ) /// select ///     'unindexed_foreign_keys' as \"name!\", ///     'Unindexed foreign keys' as \"title!\", ///     'INFO' as \"level!\", ///     'EXTERNAL' as \"facing!\", ///     array\\['PERFORMANCE'] as \"categories!\", ///     'Identifies foreign key constraints without a covering index, which can impact database performance.' as \"description!\", ///     format( ///         'Table `%s.%s` has a foreign key `%s` without a covering index. This can lead to suboptimal query performance.', ///         fk.schema_name, ///         fk.table_name, ///         fk.fkey_name ///     ) as \"detail!\", ///     'https://supabase.com/docs/guides/database/database-linter?lint=0001_unindexed_foreign_keys' as \"remediation!\", ///     jsonb_build_object( ///         'schema', fk.schema_name, ///         'name', fk.table_name, ///         'type', 'table', ///         'fkey_name', fk.fkey_name, ///         'fkey_columns', fk.col_attnums ///     ) as \"metadata!\", ///     format('unindexed_foreign_keys_%s_%s_%s', fk.schema_name, fk.table_name, fk.fkey_name) as \"cache_key!\" /// from ///     foreign_keys fk ///     left join index_ idx ///         on fk.table_oid = idx.table_oid ///         and fk.col_attnums = idx.col_attnums\\[1:array_length(fk.col_attnums, 1)] ///     left join pg_catalog.pg_depend dep ///         on idx.table_oid = dep.objid ///         and dep.deptype = 'e' /// where ///     idx.index_ is null ///     and fk.schema_name not in ( ///         '_timescaledb_cache', '_timescaledb_catalog', '_timescaledb_config', '_timescaledb_internal', 'auth', 'cron', 'extensions', 'graphql', 'graphql_public', 'information_schema', 'net', 'pgmq', 'pgroonga', 'pgsodium', 'pgsodium_masks', 'pgtle', 'pgbouncer', 'pg_catalog', 'pgtle', 'realtime', 'repack', 'storage', 'supabase_functions', 'supabase_migrations', 'tiger', 'topology', 'vault' ///     ) ///     and dep.objid is null -- exclude tables owned by extensions /// order by ///     fk.schema_name, ///     fk.table_name, ///     fk.fkey_name ///  /// /// ## Configuration /// /// Enable or disable this rule in your configuration: /// /// json /// { ///   \"splinter\": { ///     \"rules\": { ///       \"performance\": { ///         \"unindexedForeignKeys\": \"warn\" ///       } ///     } ///   } /// } ///  /// /// ## Remediation /// /// See: https://supabase.com/docs/guides/database/database-linter?lint=0001_unindexed_foreign_keys"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unindexed_foreign_keys: Option<RuleConfiguration<()>>,
    #[doc = "/// # Unused Index /// /// Detects if an index has never been used and may be a candidate for removal. /// /// ## SQL Query /// /// sql /// ( /// select ///     'unused_index' as \"name!\", ///     'Unused Index' as \"title!\", ///     'INFO' as \"level!\", ///     'EXTERNAL' as \"facing!\", ///     array\\['PERFORMANCE'] as \"categories!\", ///     'Detects if an index has never been used and may be a candidate for removal.' as \"description!\", ///     format( ///         'Index \\`%s\\` on table \\`%s.%s\\` has not been used', ///         psui.indexrelname, ///         psui.schemaname, ///         psui.relname ///     ) as \"detail!\", ///     'https://supabase.com/docs/guides/database/database-linter?lint=0005_unused_index' as \"remediation!\", ///     jsonb_build_object( ///         'schema', psui.schemaname, ///         'name', psui.relname, ///         'type', 'table' ///     ) as \"metadata!\", ///     format( ///         'unused_index_%s_%s_%s', ///         psui.schemaname, ///         psui.relname, ///         psui.indexrelname ///     ) as \"cache_key!\" ///  /// from ///     pg_catalog.pg_stat_user_indexes psui ///     join pg_catalog.pg_index pi ///         on psui.indexrelid = pi.indexrelid ///     left join pg_catalog.pg_depend dep ///         on psui.relid = dep.objid ///         and dep.deptype = 'e' /// where ///     psui.idx_scan = 0 ///     and not pi.indisunique ///     and not pi.indisprimary ///     and dep.objid is null -- exclude tables owned by extensions ///     and psui.schemaname not in ( ///         '_timescaledb_cache', '_timescaledb_catalog', '_timescaledb_config', '_timescaledb_internal', 'auth', 'cron', 'extensions', 'graphql', 'graphql_public', 'information_schema', 'net', 'pgmq', 'pgroonga', 'pgsodium', 'pgsodium_masks', 'pgtle', 'pgbouncer', 'pg_catalog', 'pgtle', 'realtime', 'repack', 'storage', 'supabase_functions', 'supabase_migrations', 'tiger', 'topology', 'vault' ///     )) ///  /// /// ## Configuration /// /// Enable or disable this rule in your configuration: /// /// json /// { ///   \"splinter\": { ///     \"rules\": { ///       \"performance\": { ///         \"unusedIndex\": \"warn\" ///       } ///     } ///   } /// } ///  /// /// ## Remediation /// /// See: https://supabase.com/docs/guides/database/database-linter?lint=0005_unused_index"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unused_index: Option<RuleConfiguration<()>>,
}
impl Performance {
    const GROUP_NAME: &'static str = "performance";
    pub(crate) const GROUP_RULES: &'static [&'static str] = &[
        "authRlsInitplan",
        "duplicateIndex",
        "multiplePermissivePolicies",
        "noPrimaryKey",
        "tableBloat",
        "unindexedForeignKeys",
        "unusedIndex",
    ];
    const RECOMMENDED_RULES_AS_FILTERS: &'static [RuleFilter<'static>] = &[];
    const ALL_RULES_AS_FILTERS: &'static [RuleFilter<'static>] = &[
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[0]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[1]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[2]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[3]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[4]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[5]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[6]),
    ];
    #[doc = r" Retrieves the recommended rules"]
    pub(crate) fn is_recommended_true(&self) -> bool {
        matches!(self.recommended, Some(true))
    }
    pub(crate) fn is_recommended_unset(&self) -> bool {
        self.recommended.is_none()
    }
    pub(crate) fn is_all_true(&self) -> bool {
        matches!(self.all, Some(true))
    }
    pub(crate) fn is_all_unset(&self) -> bool {
        self.all.is_none()
    }
    pub(crate) fn get_enabled_rules(&self) -> FxHashSet<RuleFilter<'static>> {
        let mut index_set = FxHashSet::default();
        if let Some(rule) = self.auth_rls_initplan.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[0]));
            }
        }
        if let Some(rule) = self.duplicate_index.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[1]));
            }
        }
        if let Some(rule) = self.multiple_permissive_policies.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[2]));
            }
        }
        if let Some(rule) = self.no_primary_key.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[3]));
            }
        }
        if let Some(rule) = self.table_bloat.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[4]));
            }
        }
        if let Some(rule) = self.unindexed_foreign_keys.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[5]));
            }
        }
        if let Some(rule) = self.unused_index.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[6]));
            }
        }
        index_set
    }
    pub(crate) fn get_disabled_rules(&self) -> FxHashSet<RuleFilter<'static>> {
        let mut index_set = FxHashSet::default();
        if let Some(rule) = self.auth_rls_initplan.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[0]));
            }
        }
        if let Some(rule) = self.duplicate_index.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[1]));
            }
        }
        if let Some(rule) = self.multiple_permissive_policies.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[2]));
            }
        }
        if let Some(rule) = self.no_primary_key.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[3]));
            }
        }
        if let Some(rule) = self.table_bloat.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[4]));
            }
        }
        if let Some(rule) = self.unindexed_foreign_keys.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[5]));
            }
        }
        if let Some(rule) = self.unused_index.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[6]));
            }
        }
        index_set
    }
    #[doc = r" Checks if, given a rule name, matches one of the rules contained in this category"]
    pub(crate) fn has_rule(rule_name: &str) -> Option<&'static str> {
        Some(Self::GROUP_RULES[Self::GROUP_RULES.binary_search(&rule_name).ok()?])
    }
    pub(crate) fn recommended_rules_as_filters() -> &'static [RuleFilter<'static>] {
        Self::RECOMMENDED_RULES_AS_FILTERS
    }
    pub(crate) fn all_rules_as_filters() -> &'static [RuleFilter<'static>] {
        Self::ALL_RULES_AS_FILTERS
    }
    #[doc = r" Select preset rules"]
    pub(crate) fn collect_preset_rules(
        &self,
        parent_is_all: bool,
        parent_is_recommended: bool,
        enabled_rules: &mut FxHashSet<RuleFilter<'static>>,
    ) {
        if self.is_all_true() || self.is_all_unset() && parent_is_all {
            enabled_rules.extend(Self::all_rules_as_filters());
        } else if self.is_recommended_true()
            || self.is_recommended_unset() && self.is_all_unset() && parent_is_recommended
        {
            enabled_rules.extend(Self::recommended_rules_as_filters());
        }
    }
    pub(crate) fn severity(rule_name: &str) -> Severity {
        match rule_name {
            "authRlsInitplan" => Severity::Warning,
            "duplicateIndex" => Severity::Warning,
            "multiplePermissivePolicies" => Severity::Warning,
            "noPrimaryKey" => Severity::Information,
            "tableBloat" => Severity::Information,
            "unindexedForeignKeys" => Severity::Information,
            "unusedIndex" => Severity::Information,
            _ => unreachable!(),
        }
    }
    pub(crate) fn get_rule_configuration(
        &self,
        rule_name: &str,
    ) -> Option<(RulePlainConfiguration, Option<RuleOptions>)> {
        match rule_name {
            "authRlsInitplan" => self
                .auth_rls_initplan
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "duplicateIndex" => self
                .duplicate_index
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "multiplePermissivePolicies" => self
                .multiple_permissive_policies
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "noPrimaryKey" => self
                .no_primary_key
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "tableBloat" => self
                .table_bloat
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "unindexedForeignKeys" => self
                .unindexed_foreign_keys
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "unusedIndex" => self
                .unused_index
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            _ => None,
        }
    }
}
#[derive(Clone, Debug, Default, Deserialize, Eq, Merge, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
#[doc = r" A list of rules that belong to this group"]
pub struct Security {
    #[doc = r" It enables the recommended rules for this group"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended: Option<bool>,
    #[doc = r" It enables ALL rules for this group."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all: Option<bool>,
    #[doc = "/// # Exposed Auth Users /// /// Detects if auth.users is exposed to anon or authenticated roles via a view or materialized view in schemas exposed to PostgREST, potentially compromising user data security. /// /// Note: This rule requires Supabase roles (anon, authenticated, service_role). /// It will be automatically skipped if these roles don't exist in your database. /// /// ## SQL Query /// /// sql /// ( /// select ///     'auth_users_exposed' as \"name!\", ///     'Exposed Auth Users' as \"title!\", ///     'ERROR' as \"level!\", ///     'EXTERNAL' as \"facing!\", ///     array\\['SECURITY'] as \"categories!\", ///     'Detects if auth.users is exposed to anon or authenticated roles via a view or materialized view in schemas exposed to PostgREST, potentially compromising user data security.' as \"description!\", ///     format( ///         'View/Materialized View \"%s\" in the public schema may expose \\`auth.users\\` data to anon or authenticated roles.', ///         c.relname ///     ) as \"detail!\", ///     'https://supabase.com/docs/guides/database/database-linter?lint=0002_auth_users_exposed' as \"remediation!\", ///     jsonb_build_object( ///         'schema', n.nspname, ///         'name', c.relname, ///         'type', 'view', ///         'exposed_to', array_remove(array_agg(DISTINCT case when pg_catalog.has_table_privilege('anon', c.oid, 'SELECT') then 'anon' when pg_catalog.has_table_privilege('authenticated', c.oid, 'SELECT') then 'authenticated' end), null) ///     ) as \"metadata!\", ///     format('auth_users_exposed_%s_%s', n.nspname, c.relname) as \"cache_key!\" /// from ///     -- Identify the oid for auth.users ///     pg_catalog.pg_class auth_users_pg_class ///     join pg_catalog.pg_namespace auth_users_pg_namespace ///         on auth_users_pg_class.relnamespace = auth_users_pg_namespace.oid ///         and auth_users_pg_class.relname = 'users' ///         and auth_users_pg_namespace.nspname = 'auth' ///     -- Depends on auth.users ///     join pg_catalog.pg_depend d ///         on d.refobjid = auth_users_pg_class.oid ///     join pg_catalog.pg_rewrite r ///         on r.oid = d.objid ///     join pg_catalog.pg_class c ///         on c.oid = r.ev_class ///     join pg_catalog.pg_namespace n ///         on n.oid = c.relnamespace ///     join pg_catalog.pg_class pg_class_auth_users ///         on d.refobjid = pg_class_auth_users.oid /// where ///     d.deptype = 'n' ///     and ( ///       pg_catalog.has_table_privilege('anon', c.oid, 'SELECT') ///       or pg_catalog.has_table_privilege('authenticated', c.oid, 'SELECT') ///     ) ///     and n.nspname = any(array(select trim(unnest(string_to_array(current_setting('pgrst.db_schemas', 't'), ','))))) ///     -- Exclude self ///     and c.relname \\<> '0002_auth_users_exposed' ///     -- There are 3 insecure configurations ///     and ///     ( ///         -- Materialized views don't support RLS so this is insecure by default ///         (c.relkind in ('m')) -- m for materialized view ///         or ///         -- Standard View, accessible to anon or authenticated that is security_definer ///         ( ///             c.relkind = 'v' -- v for view ///             -- Exclude security invoker views ///             and not ( ///                 lower(coalesce(c.reloptions::text,'{}'))::text\\[] ///                 && array\\[ ///                     'security_invoker=1', ///                     'security_invoker=true', ///                     'security_invoker=yes', ///                     'security_invoker=on' ///                 ] ///             ) ///         ) ///         or ///         -- Standard View, security invoker, but no RLS enabled on auth.users ///         ( ///             c.relkind in ('v') -- v for view ///             -- is security invoker ///             and ( ///                 lower(coalesce(c.reloptions::text,'{}'))::text\\[] ///                 && array\\[ ///                     'security_invoker=1', ///                     'security_invoker=true', ///                     'security_invoker=yes', ///                     'security_invoker=on' ///                 ] ///             ) ///             and not pg_class_auth_users.relrowsecurity ///         ) ///     ) /// group by ///     n.nspname, ///     c.relname, ///     c.oid) ///  /// /// ## Configuration /// /// Enable or disable this rule in your configuration: /// /// json /// { ///   \"splinter\": { ///     \"rules\": { ///       \"security\": { ///         \"authUsersExposed\": \"warn\" ///       } ///     } ///   } /// } ///  /// /// ## Remediation /// /// See: https://supabase.com/docs/guides/database/database-linter?lint=0002_auth_users_exposed"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_users_exposed: Option<RuleConfiguration<()>>,
    #[doc = "/// # Extension in Public /// /// Detects extensions installed in the `public` schema. /// /// ## SQL Query /// /// sql /// ( /// select ///     'extension_in_public' as \"name!\", ///     'Extension in Public' as \"title!\", ///     'WARN' as \"level!\", ///     'EXTERNAL' as \"facing!\", ///     array\\['SECURITY'] as \"categories!\", ///     'Detects extensions installed in the \\`public\\` schema.' as \"description!\", ///     format( ///         'Extension \\`%s\\` is installed in the public schema. Move it to another schema.', ///         pe.extname ///     ) as \"detail!\", ///     'https://supabase.com/docs/guides/database/database-linter?lint=0014_extension_in_public' as \"remediation!\", ///     jsonb_build_object( ///         'schema', pe.extnamespace::regnamespace, ///         'name', pe.extname, ///         'type', 'extension' ///     ) as \"metadata!\", ///     format( ///         'extension_in_public_%s', ///         pe.extname ///     ) as \"cache_key!\" /// from ///     pg_catalog.pg_extension pe /// where ///     -- plpgsql is installed by default in public and outside user control ///     -- confirmed safe ///     pe.extname not in ('plpgsql') ///     -- Scoping this to public is not optimal. Ideally we would use the postgres ///     -- search path. That currently isn't available via SQL. In other lints ///     -- we have used has_schema_privilege('anon', 'extensions', 'USAGE') but that ///     -- is not appropriate here as it would evaluate true for the extensions schema ///     and pe.extnamespace::regnamespace::text = 'public') ///  /// /// ## Configuration /// /// Enable or disable this rule in your configuration: /// /// json /// { ///   \"splinter\": { ///     \"rules\": { ///       \"security\": { ///         \"extensionInPublic\": \"warn\" ///       } ///     } ///   } /// } ///  /// /// ## Remediation /// /// See: https://supabase.com/docs/guides/database/database-linter?lint=0014_extension_in_public"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension_in_public: Option<RuleConfiguration<()>>,
    #[doc = "/// # Extension Versions Outdated /// /// Detects extensions that are not using the default (recommended) version. /// /// ## SQL Query /// /// sql /// ( /// select ///     'extension_versions_outdated' as \"name!\", ///     'Extension Versions Outdated' as \"title!\", ///     'WARN' as \"level!\", ///     'EXTERNAL' as \"facing!\", ///     array\\['SECURITY'] as \"categories!\", ///     'Detects extensions that are not using the default (recommended) version.' as \"description!\", ///     format( ///         'Extension `%s` is using version `%s` but version `%s` is available. Using outdated extension versions may expose the database to security vulnerabilities.', ///         ext.name, ///         ext.installed_version, ///         ext.default_version ///     ) as \"detail!\", ///     'https://supabase.com/docs/guides/database/database-linter?lint=0022_extension_versions_outdated' as \"remediation!\", ///     jsonb_build_object( ///         'extension_name', ext.name, ///         'installed_version', ext.installed_version, ///         'default_version', ext.default_version ///     ) as \"metadata!\", ///     format( ///         'extension_versions_outdated_%s_%s', ///         ext.name, ///         ext.installed_version ///     ) as \"cache_key!\" /// from ///     pg_catalog.pg_available_extensions ext /// join ///     -- ignore versions not in pg_available_extension_versions ///     -- e.g. residue of pg_upgrade ///     pg_catalog.pg_available_extension_versions extv ///     on extv.name = ext.name and extv.installed /// where ///     ext.installed_version is not null ///     and ext.default_version is not null ///     and ext.installed_version != ext.default_version /// order by ///     ext.name) ///  /// /// ## Configuration /// /// Enable or disable this rule in your configuration: /// /// json /// { ///   \"splinter\": { ///     \"rules\": { ///       \"security\": { ///         \"extensionVersionsOutdated\": \"warn\" ///       } ///     } ///   } /// } ///  /// /// ## Remediation /// /// See: https://supabase.com/docs/guides/database/database-linter?lint=0022_extension_versions_outdated"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension_versions_outdated: Option<RuleConfiguration<()>>,
    #[doc = "/// # Foreign Key to Auth Unique Constraint /// /// Detects user defined foreign keys to unique constraints in the auth schema. /// /// Note: This rule requires Supabase roles (anon, authenticated, service_role). /// It will be automatically skipped if these roles don't exist in your database. /// /// ## SQL Query /// /// sql /// ( /// select ///     'fkey_to_auth_unique' as \"name!\", ///     'Foreign Key to Auth Unique Constraint' as \"title!\", ///     'ERROR' as \"level!\", ///     'EXTERNAL' as \"facing!\", ///     array\\['SECURITY'] as \"categories!\", ///     'Detects user defined foreign keys to unique constraints in the auth schema.' as \"description!\", ///     format( ///         'Table `%s`.`%s` has a foreign key `%s` referencing an auth unique constraint', ///         n.nspname, -- referencing schema ///         c_rel.relname, -- referencing table ///         c.conname -- fkey name ///     ) as \"detail!\", ///     'Drop the foreign key constraint that references the auth schema.' as \"remediation!\", ///     jsonb_build_object( ///         'schema', n.nspname, ///         'name', c_rel.relname, ///         'foreign_key', c.conname ///     ) as \"metadata!\", ///     format( ///         'fkey_to_auth_unique_%s_%s_%s', ///         n.nspname, -- referencing schema ///         c_rel.relname, -- referencing table ///         c.conname ///     ) as \"cache_key!\" /// from ///     pg_catalog.pg_constraint c ///     join pg_catalog.pg_class c_rel ///         on c.conrelid = c_rel.oid ///     join pg_catalog.pg_namespace n ///         on c_rel.relnamespace = n.oid ///     join pg_catalog.pg_class ref_rel ///         on c.confrelid = ref_rel.oid ///     join pg_catalog.pg_namespace cn ///         on ref_rel.relnamespace = cn.oid ///     join pg_catalog.pg_index i ///         on c.conindid = i.indexrelid /// where c.contype = 'f' ///     and cn.nspname = 'auth' ///     and i.indisunique ///     and not i.indisprimary) ///  /// /// ## Configuration /// /// Enable or disable this rule in your configuration: /// /// json /// { ///   \"splinter\": { ///     \"rules\": { ///       \"security\": { ///         \"fkeyToAuthUnique\": \"warn\" ///       } ///     } ///   } /// } ///  /// /// ## Remediation /// /// See: "]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fkey_to_auth_unique: Option<RuleConfiguration<()>>,
    #[doc = "/// # Foreign Table in API /// /// Detects foreign tables that are accessible over APIs. Foreign tables do not respect row level security policies. /// /// Note: This rule requires Supabase roles (anon, authenticated, service_role). /// It will be automatically skipped if these roles don't exist in your database. /// /// ## SQL Query /// /// sql /// ( /// select ///     'foreign_table_in_api' as \"name!\", ///     'Foreign Table in API' as \"title!\", ///     'WARN' as \"level!\", ///     'EXTERNAL' as \"facing!\", ///     array\\['SECURITY'] as \"categories!\", ///     'Detects foreign tables that are accessible over APIs. Foreign tables do not respect row level security policies.' as \"description!\", ///     format( ///         'Foreign table \\`%s.%s\\` is accessible over APIs', ///         n.nspname, ///         c.relname ///     ) as \"detail!\", ///     'https://supabase.com/docs/guides/database/database-linter?lint=0017_foreign_table_in_api' as \"remediation!\", ///     jsonb_build_object( ///         'schema', n.nspname, ///         'name', c.relname, ///         'type', 'foreign table' ///     ) as \"metadata!\", ///     format( ///         'foreign_table_in_api_%s_%s', ///         n.nspname, ///         c.relname ///     ) as \"cache_key!\" /// from ///     pg_catalog.pg_class c ///     join pg_catalog.pg_namespace n ///         on n.oid = c.relnamespace ///     left join pg_catalog.pg_depend dep ///         on c.oid = dep.objid ///         and dep.deptype = 'e' /// where ///     c.relkind = 'f' ///     and ( ///         pg_catalog.has_table_privilege('anon', c.oid, 'SELECT') ///         or pg_catalog.has_table_privilege('authenticated', c.oid, 'SELECT') ///     ) ///     and n.nspname = any(array(select trim(unnest(string_to_array(current_setting('pgrst.db_schemas', 't'), ','))))) ///     and n.nspname not in ( ///         '_timescaledb_cache', '_timescaledb_catalog', '_timescaledb_config', '_timescaledb_internal', 'auth', 'cron', 'extensions', 'graphql', 'graphql_public', 'information_schema', 'net', 'pgmq', 'pgroonga', 'pgsodium', 'pgsodium_masks', 'pgtle', 'pgbouncer', 'pg_catalog', 'pgtle', 'realtime', 'repack', 'storage', 'supabase_functions', 'supabase_migrations', 'tiger', 'topology', 'vault' ///     ) ///     and dep.objid is null) ///  /// /// ## Configuration /// /// Enable or disable this rule in your configuration: /// /// json /// { ///   \"splinter\": { ///     \"rules\": { ///       \"security\": { ///         \"foreignTableInApi\": \"warn\" ///       } ///     } ///   } /// } ///  /// /// ## Remediation /// /// See: https://supabase.com/docs/guides/database/database-linter?lint=0017_foreign_table_in_api"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foreign_table_in_api: Option<RuleConfiguration<()>>,
    #[doc = "/// # Function Search Path Mutable /// /// Detects functions where the search_path parameter is not set. /// /// ## SQL Query /// /// sql /// ( /// select ///     'function_search_path_mutable' as \"name!\", ///     'Function Search Path Mutable' as \"title!\", ///     'WARN' as \"level!\", ///     'EXTERNAL' as \"facing!\", ///     array\\['SECURITY'] as \"categories!\", ///     'Detects functions where the search_path parameter is not set.' as \"description!\", ///     format( ///         'Function \\`%s.%s\\` has a role mutable search_path', ///         n.nspname, ///         p.proname ///     ) as \"detail!\", ///     'https://supabase.com/docs/guides/database/database-linter?lint=0011_function_search_path_mutable' as \"remediation!\", ///     jsonb_build_object( ///         'schema', n.nspname, ///         'name', p.proname, ///         'type', 'function' ///     ) as \"metadata!\", ///     format( ///         'function_search_path_mutable_%s_%s_%s', ///         n.nspname, ///         p.proname, ///         md5(p.prosrc) -- required when function is polymorphic ///     ) as \"cache_key!\" /// from ///     pg_catalog.pg_proc p ///     join pg_catalog.pg_namespace n ///         on p.pronamespace = n.oid ///     left join pg_catalog.pg_depend dep ///         on p.oid = dep.objid ///         and dep.deptype = 'e' /// where ///     n.nspname not in ( ///         '_timescaledb_cache', '_timescaledb_catalog', '_timescaledb_config', '_timescaledb_internal', 'auth', 'cron', 'extensions', 'graphql', 'graphql_public', 'information_schema', 'net', 'pgmq', 'pgroonga', 'pgsodium', 'pgsodium_masks', 'pgtle', 'pgbouncer', 'pg_catalog', 'pgtle', 'realtime', 'repack', 'storage', 'supabase_functions', 'supabase_migrations', 'tiger', 'topology', 'vault' ///     ) ///     and dep.objid is null -- exclude functions owned by extensions ///     -- Search path not set ///     and not exists ( ///         select 1 ///         from unnest(coalesce(p.proconfig, '{}')) as config ///         where config like 'search_path=%' ///     )) ///  /// /// ## Configuration /// /// Enable or disable this rule in your configuration: /// /// json /// { ///   \"splinter\": { ///     \"rules\": { ///       \"security\": { ///         \"functionSearchPathMutable\": \"warn\" ///       } ///     } ///   } /// } ///  /// /// ## Remediation /// /// See: https://supabase.com/docs/guides/database/database-linter?lint=0011_function_search_path_mutable"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_search_path_mutable: Option<RuleConfiguration<()>>,
    #[doc = "/// # Insecure Queue Exposed in API /// /// Detects cases where an insecure Queue is exposed over Data APIs /// /// Note: This rule requires Supabase roles (anon, authenticated, service_role). /// It will be automatically skipped if these roles don't exist in your database. /// /// ## SQL Query /// /// sql /// ( /// select ///     'insecure_queue_exposed_in_api' as \"name!\", ///     'Insecure Queue Exposed in API' as \"title!\", ///     'ERROR' as \"level!\", ///     'EXTERNAL' as \"facing!\", ///     array\\['SECURITY'] as \"categories!\", ///     'Detects cases where an insecure Queue is exposed over Data APIs' as \"description!\", ///     format( ///         'Table \\`%s.%s\\` is public, but RLS has not been enabled.', ///         n.nspname, ///         c.relname ///     ) as \"detail!\", ///     'https://supabase.com/docs/guides/database/database-linter?lint=0019_insecure_queue_exposed_in_api' as \"remediation!\", ///     jsonb_build_object( ///         'schema', n.nspname, ///         'name', c.relname, ///         'type', 'table' ///     ) as \"metadata!\", ///     format( ///         'rls_disabled_in_public_%s_%s', ///         n.nspname, ///         c.relname ///     ) as \"cache_key!\" /// from ///     pg_catalog.pg_class c ///     join pg_catalog.pg_namespace n ///         on c.relnamespace = n.oid /// where ///     c.relkind in ('r', 'I') -- regular or partitioned tables ///     and not c.relrowsecurity -- RLS is disabled ///     and ( ///         pg_catalog.has_table_privilege('anon', c.oid, 'SELECT') ///         or pg_catalog.has_table_privilege('authenticated', c.oid, 'SELECT') ///     ) ///     and n.nspname = 'pgmq' -- tables in the pgmq schema ///     and c.relname like 'q_%' -- only queue tables ///     -- Constant requirements ///     and 'pgmq_public' = any(array(select trim(unnest(string_to_array(current_setting('pgrst.db_schemas', 't'), ',')))))) ///  /// /// ## Configuration /// /// Enable or disable this rule in your configuration: /// /// json /// { ///   \"splinter\": { ///     \"rules\": { ///       \"security\": { ///         \"insecureQueueExposedInApi\": \"warn\" ///       } ///     } ///   } /// } ///  /// /// ## Remediation /// /// See: https://supabase.com/docs/guides/database/database-linter?lint=0019_insecure_queue_exposed_in_api"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insecure_queue_exposed_in_api: Option<RuleConfiguration<()>>,
    #[doc = "/// # Materialized View in API /// /// Detects materialized views that are accessible over the Data APIs. /// /// Note: This rule requires Supabase roles (anon, authenticated, service_role). /// It will be automatically skipped if these roles don't exist in your database. /// /// ## SQL Query /// /// sql /// ( /// select ///     'materialized_view_in_api' as \"name!\", ///     'Materialized View in API' as \"title!\", ///     'WARN' as \"level!\", ///     'EXTERNAL' as \"facing!\", ///     array\\['SECURITY'] as \"categories!\", ///     'Detects materialized views that are accessible over the Data APIs.' as \"description!\", ///     format( ///         'Materialized view \\`%s.%s\\` is selectable by anon or authenticated roles', ///         n.nspname, ///         c.relname ///     ) as \"detail!\", ///     'https://supabase.com/docs/guides/database/database-linter?lint=0016_materialized_view_in_api' as \"remediation!\", ///     jsonb_build_object( ///         'schema', n.nspname, ///         'name', c.relname, ///         'type', 'materialized view' ///     ) as \"metadata!\", ///     format( ///         'materialized_view_in_api_%s_%s', ///         n.nspname, ///         c.relname ///     ) as \"cache_key!\" /// from ///     pg_catalog.pg_class c ///     join pg_catalog.pg_namespace n ///         on n.oid = c.relnamespace ///     left join pg_catalog.pg_depend dep ///         on c.oid = dep.objid ///         and dep.deptype = 'e' /// where ///     c.relkind = 'm' ///     and ( ///         pg_catalog.has_table_privilege('anon', c.oid, 'SELECT') ///         or pg_catalog.has_table_privilege('authenticated', c.oid, 'SELECT') ///     ) ///     and n.nspname = any(array(select trim(unnest(string_to_array(current_setting('pgrst.db_schemas', 't'), ','))))) ///     and n.nspname not in ( ///         '_timescaledb_cache', '_timescaledb_catalog', '_timescaledb_config', '_timescaledb_internal', 'auth', 'cron', 'extensions', 'graphql', 'graphql_public', 'information_schema', 'net', 'pgmq', 'pgroonga', 'pgsodium', 'pgsodium_masks', 'pgtle', 'pgbouncer', 'pg_catalog', 'pgtle', 'realtime', 'repack', 'storage', 'supabase_functions', 'supabase_migrations', 'tiger', 'topology', 'vault' ///     ) ///     and dep.objid is null) ///  /// /// ## Configuration /// /// Enable or disable this rule in your configuration: /// /// json /// { ///   \"splinter\": { ///     \"rules\": { ///       \"security\": { ///         \"materializedViewInApi\": \"warn\" ///       } ///     } ///   } /// } ///  /// /// ## Remediation /// /// See: https://supabase.com/docs/guides/database/database-linter?lint=0016_materialized_view_in_api"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub materialized_view_in_api: Option<RuleConfiguration<()>>,
    #[doc = "/// # Policy Exists RLS Disabled /// /// Detects cases where row level security (RLS) policies have been created, but RLS has not been enabled for the underlying table. /// /// ## SQL Query /// /// sql /// ( /// select ///     'policy_exists_rls_disabled' as \"name!\", ///     'Policy Exists RLS Disabled' as \"title!\", ///     'ERROR' as \"level!\", ///     'EXTERNAL' as \"facing!\", ///     array\\['SECURITY'] as \"categories!\", ///     'Detects cases where row level security (RLS) policies have been created, but RLS has not been enabled for the underlying table.' as \"description!\", ///     format( ///         'Table \\`%s.%s\\` has RLS policies but RLS is not enabled on the table. Policies include %s.', ///         n.nspname, ///         c.relname, ///         array_agg(p.polname order by p.polname) ///     ) as \"detail!\", ///     'https://supabase.com/docs/guides/database/database-linter?lint=0007_policy_exists_rls_disabled' as \"remediation!\", ///     jsonb_build_object( ///         'schema', n.nspname, ///         'name', c.relname, ///         'type', 'table' ///     ) as \"metadata!\", ///     format( ///         'policy_exists_rls_disabled_%s_%s', ///         n.nspname, ///         c.relname ///     ) as \"cache_key!\" /// from ///     pg_catalog.pg_policy p ///     join pg_catalog.pg_class c ///         on p.polrelid = c.oid ///     join pg_catalog.pg_namespace n ///         on c.relnamespace = n.oid ///     left join pg_catalog.pg_depend dep ///         on c.oid = dep.objid ///         and dep.deptype = 'e' /// where ///     c.relkind = 'r' -- regular tables ///     and n.nspname not in ( ///         '_timescaledb_cache', '_timescaledb_catalog', '_timescaledb_config', '_timescaledb_internal', 'auth', 'cron', 'extensions', 'graphql', 'graphql_public', 'information_schema', 'net', 'pgmq', 'pgroonga', 'pgsodium', 'pgsodium_masks', 'pgtle', 'pgbouncer', 'pg_catalog', 'pgtle', 'realtime', 'repack', 'storage', 'supabase_functions', 'supabase_migrations', 'tiger', 'topology', 'vault' ///     ) ///     -- RLS is disabled ///     and not c.relrowsecurity ///     and dep.objid is null -- exclude tables owned by extensions /// group by ///     n.nspname, ///     c.relname) ///  /// /// ## Configuration /// /// Enable or disable this rule in your configuration: /// /// json /// { ///   \"splinter\": { ///     \"rules\": { ///       \"security\": { ///         \"policyExistsRlsDisabled\": \"warn\" ///       } ///     } ///   } /// } ///  /// /// ## Remediation /// /// See: https://supabase.com/docs/guides/database/database-linter?lint=0007_policy_exists_rls_disabled"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_exists_rls_disabled: Option<RuleConfiguration<()>>,
    #[doc = "/// # RLS Disabled in Public /// /// Detects cases where row level security (RLS) has not been enabled on tables in schemas exposed to PostgREST /// /// Note: This rule requires Supabase roles (anon, authenticated, service_role). /// It will be automatically skipped if these roles don't exist in your database. /// /// ## SQL Query /// /// sql /// ( /// select ///     'rls_disabled_in_public' as \"name!\", ///     'RLS Disabled in Public' as \"title!\", ///     'ERROR' as \"level!\", ///     'EXTERNAL' as \"facing!\", ///     array\\['SECURITY'] as \"categories!\", ///     'Detects cases where row level security (RLS) has not been enabled on tables in schemas exposed to PostgREST' as \"description!\", ///     format( ///         'Table \\`%s.%s\\` is public, but RLS has not been enabled.', ///         n.nspname, ///         c.relname ///     ) as \"detail!\", ///     'https://supabase.com/docs/guides/database/database-linter?lint=0013_rls_disabled_in_public' as \"remediation!\", ///     jsonb_build_object( ///         'schema', n.nspname, ///         'name', c.relname, ///         'type', 'table' ///     ) as \"metadata!\", ///     format( ///         'rls_disabled_in_public_%s_%s', ///         n.nspname, ///         c.relname ///     ) as \"cache_key!\" /// from ///     pg_catalog.pg_class c ///     join pg_catalog.pg_namespace n ///         on c.relnamespace = n.oid /// where ///     c.relkind = 'r' -- regular tables ///     -- RLS is disabled ///     and not c.relrowsecurity ///     and ( ///         pg_catalog.has_table_privilege('anon', c.oid, 'SELECT') ///         or pg_catalog.has_table_privilege('authenticated', c.oid, 'SELECT') ///     ) ///     and n.nspname = any(array(select trim(unnest(string_to_array(current_setting('pgrst.db_schemas', 't'), ','))))) ///     and n.nspname not in ( ///         '_timescaledb_cache', '_timescaledb_catalog', '_timescaledb_config', '_timescaledb_internal', 'auth', 'cron', 'extensions', 'graphql', 'graphql_public', 'information_schema', 'net', 'pgmq', 'pgroonga', 'pgsodium', 'pgsodium_masks', 'pgtle', 'pgbouncer', 'pg_catalog', 'pgtle', 'realtime', 'repack', 'storage', 'supabase_functions', 'supabase_migrations', 'tiger', 'topology', 'vault' ///     )) ///  /// /// ## Configuration /// /// Enable or disable this rule in your configuration: /// /// json /// { ///   \"splinter\": { ///     \"rules\": { ///       \"security\": { ///         \"rlsDisabledInPublic\": \"warn\" ///       } ///     } ///   } /// } ///  /// /// ## Remediation /// /// See: https://supabase.com/docs/guides/database/database-linter?lint=0013_rls_disabled_in_public"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rls_disabled_in_public: Option<RuleConfiguration<()>>,
    #[doc = "/// # RLS Enabled No Policy /// /// Detects cases where row level security (RLS) has been enabled on a table but no RLS policies have been created. /// /// ## SQL Query /// /// sql /// ( /// select ///     'rls_enabled_no_policy' as \"name!\", ///     'RLS Enabled No Policy' as \"title!\", ///     'INFO' as \"level!\", ///     'EXTERNAL' as \"facing!\", ///     array\\['SECURITY'] as \"categories!\", ///     'Detects cases where row level security (RLS) has been enabled on a table but no RLS policies have been created.' as \"description!\", ///     format( ///         'Table \\`%s.%s\\` has RLS enabled, but no policies exist', ///         n.nspname, ///         c.relname ///     ) as \"detail!\", ///     'https://supabase.com/docs/guides/database/database-linter?lint=0008_rls_enabled_no_policy' as \"remediation!\", ///     jsonb_build_object( ///         'schema', n.nspname, ///         'name', c.relname, ///         'type', 'table' ///     ) as \"metadata!\", ///     format( ///         'rls_enabled_no_policy_%s_%s', ///         n.nspname, ///         c.relname ///     ) as \"cache_key!\" /// from ///     pg_catalog.pg_class c ///     left join pg_catalog.pg_policy p ///         on p.polrelid = c.oid ///     join pg_catalog.pg_namespace n ///         on c.relnamespace = n.oid ///     left join pg_catalog.pg_depend dep ///         on c.oid = dep.objid ///         and dep.deptype = 'e' /// where ///     c.relkind = 'r' -- regular tables ///     and n.nspname not in ( ///         '_timescaledb_cache', '_timescaledb_catalog', '_timescaledb_config', '_timescaledb_internal', 'auth', 'cron', 'extensions', 'graphql', 'graphql_public', 'information_schema', 'net', 'pgmq', 'pgroonga', 'pgsodium', 'pgsodium_masks', 'pgtle', 'pgbouncer', 'pg_catalog', 'pgtle', 'realtime', 'repack', 'storage', 'supabase_functions', 'supabase_migrations', 'tiger', 'topology', 'vault' ///     ) ///     -- RLS is enabled ///     and c.relrowsecurity ///     and p.polname is null ///     and dep.objid is null -- exclude tables owned by extensions /// group by ///     n.nspname, ///     c.relname) ///  /// /// ## Configuration /// /// Enable or disable this rule in your configuration: /// /// json /// { ///   \"splinter\": { ///     \"rules\": { ///       \"security\": { ///         \"rlsEnabledNoPolicy\": \"warn\" ///       } ///     } ///   } /// } ///  /// /// ## Remediation /// /// See: https://supabase.com/docs/guides/database/database-linter?lint=0008_rls_enabled_no_policy"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rls_enabled_no_policy: Option<RuleConfiguration<()>>,
    #[doc = "/// # RLS references user metadata /// /// Detects when Supabase Auth user_metadata is referenced insecurely in a row level security (RLS) policy. /// /// Note: This rule requires Supabase roles (anon, authenticated, service_role). /// It will be automatically skipped if these roles don't exist in your database. /// /// ## SQL Query /// /// sql /// ( /// with policies as ( ///     select ///         nsp.nspname as schema_name, ///         pb.tablename as table_name, ///         polname as policy_name, ///         qual, ///         with_check ///     from ///         pg_catalog.pg_policy pa ///         join pg_catalog.pg_class pc ///             on pa.polrelid = pc.oid ///         join pg_catalog.pg_namespace nsp ///             on pc.relnamespace = nsp.oid ///         join pg_catalog.pg_policies pb ///             on pc.relname = pb.tablename ///             and nsp.nspname = pb.schemaname ///             and pa.polname = pb.policyname /// ) /// select ///     'rls_references_user_metadata' as \"name!\", ///     'RLS references user metadata' as \"title!\", ///     'ERROR' as \"level!\", ///     'EXTERNAL' as \"facing!\", ///     array\\['SECURITY'] as \"categories!\", ///     'Detects when Supabase Auth user_metadata is referenced insecurely in a row level security (RLS) policy.' as \"description!\", ///     format( ///         'Table \\`%s.%s\\` has a row level security policy \\`%s\\` that references Supabase Auth \\`user_metadata\\`. \\`user_metadata\\` is editable by end users and should never be used in a security context.', ///         schema_name, ///         table_name, ///         policy_name ///     ) as \"detail!\", ///     'https://supabase.com/docs/guides/database/database-linter?lint=0015_rls_references_user_metadata' as \"remediation!\", ///     jsonb_build_object( ///         'schema', schema_name, ///         'name', table_name, ///         'type', 'table' ///     ) as \"metadata!\", ///     format('rls_references_user_metadata_%s_%s_%s', schema_name, table_name, policy_name) as \"cache_key!\" /// from ///     policies /// where ///     schema_name not in ( ///         '_timescaledb_cache', '_timescaledb_catalog', '_timescaledb_config', '_timescaledb_internal', 'auth', 'cron', 'extensions', 'graphql', 'graphql_public', 'information_schema', 'net', 'pgmq', 'pgroonga', 'pgsodium', 'pgsodium_masks', 'pgtle', 'pgbouncer', 'pg_catalog', 'pgtle', 'realtime', 'repack', 'storage', 'supabase_functions', 'supabase_migrations', 'tiger', 'topology', 'vault' ///     ) ///     and ( ///         -- Example: auth.jwt() -> 'user_metadata' ///         -- False positives are possible, but it isn't practical to string match ///         -- If false positive rate is too high, this expression can iterate ///         qual like '%auth.jwt()%user_metadata%' ///         or qual like '%current_setting(%request.jwt.claims%)%user_metadata%' ///         or with_check like '%auth.jwt()%user_metadata%' ///         or with_check like '%current_setting(%request.jwt.claims%)%user_metadata%' ///     )) ///  /// /// ## Configuration /// /// Enable or disable this rule in your configuration: /// /// json /// { ///   \"splinter\": { ///     \"rules\": { ///       \"security\": { ///         \"rlsReferencesUserMetadata\": \"warn\" ///       } ///     } ///   } /// } ///  /// /// ## Remediation /// /// See: https://supabase.com/docs/guides/database/database-linter?lint=0015_rls_references_user_metadata"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rls_references_user_metadata: Option<RuleConfiguration<()>>,
    #[doc = "/// # Security Definer View /// /// Detects views defined with the SECURITY DEFINER property. These views enforce Postgres permissions and row level security policies (RLS) of the view creator, rather than that of the querying user /// /// Note: This rule requires Supabase roles (anon, authenticated, service_role). /// It will be automatically skipped if these roles don't exist in your database. /// /// ## SQL Query /// /// sql /// ( /// select ///     'security_definer_view' as \"name!\", ///     'Security Definer View' as \"title!\", ///     'ERROR' as \"level!\", ///     'EXTERNAL' as \"facing!\", ///     array\\['SECURITY'] as \"categories!\", ///     'Detects views defined with the SECURITY DEFINER property. These views enforce Postgres permissions and row level security policies (RLS) of the view creator, rather than that of the querying user' as \"description!\", ///     format( ///         'View \\`%s.%s\\` is defined with the SECURITY DEFINER property', ///         n.nspname, ///         c.relname ///     ) as \"detail!\", ///     'https://supabase.com/docs/guides/database/database-linter?lint=0010_security_definer_view' as \"remediation!\", ///     jsonb_build_object( ///         'schema', n.nspname, ///         'name', c.relname, ///         'type', 'view' ///     ) as \"metadata!\", ///     format( ///         'security_definer_view_%s_%s', ///         n.nspname, ///         c.relname ///     ) as \"cache_key!\" /// from ///     pg_catalog.pg_class c ///     join pg_catalog.pg_namespace n ///         on n.oid = c.relnamespace ///     left join pg_catalog.pg_depend dep ///         on c.oid = dep.objid ///         and dep.deptype = 'e' /// where ///     c.relkind = 'v' ///     and ( ///         pg_catalog.has_table_privilege('anon', c.oid, 'SELECT') ///         or pg_catalog.has_table_privilege('authenticated', c.oid, 'SELECT') ///     ) ///     and substring(pg_catalog.version() from 'PostgreSQL (\\[0-9]+)') >= '15' -- security invoker was added in pg15 ///     and n.nspname = any(array(select trim(unnest(string_to_array(current_setting('pgrst.db_schemas', 't'), ','))))) ///     and n.nspname not in ( ///         '_timescaledb_cache', '_timescaledb_catalog', '_timescaledb_config', '_timescaledb_internal', 'auth', 'cron', 'extensions', 'graphql', 'graphql_public', 'information_schema', 'net', 'pgmq', 'pgroonga', 'pgsodium', 'pgsodium_masks', 'pgtle', 'pgbouncer', 'pg_catalog', 'pgtle', 'realtime', 'repack', 'storage', 'supabase_functions', 'supabase_migrations', 'tiger', 'topology', 'vault' ///     ) ///     and dep.objid is null -- exclude views owned by extensions ///     and not ( ///         lower(coalesce(c.reloptions::text,'{}'))::text\\[] ///         && array\\[ ///             'security_invoker=1', ///             'security_invoker=true', ///             'security_invoker=yes', ///             'security_invoker=on' ///         ] ///     )) ///  /// /// ## Configuration /// /// Enable or disable this rule in your configuration: /// /// json /// { ///   \"splinter\": { ///     \"rules\": { ///       \"security\": { ///         \"securityDefinerView\": \"warn\" ///       } ///     } ///   } /// } ///  /// /// ## Remediation /// /// See: https://supabase.com/docs/guides/database/database-linter?lint=0010_security_definer_view"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_definer_view: Option<RuleConfiguration<()>>,
    #[doc = "/// # Unsupported reg types /// /// Identifies columns using unsupported reg* types outside pg_catalog schema, which prevents database upgrades using pg_upgrade. /// /// ## SQL Query /// /// sql /// ( /// select ///     'unsupported_reg_types' as \"name!\", ///     'Unsupported reg types' as \"title!\", ///     'WARN' as \"level!\", ///     'EXTERNAL' as \"facing!\", ///     array\\['SECURITY'] as \"categories!\", ///     'Identifies columns using unsupported reg* types outside pg_catalog schema, which prevents database upgrades using pg_upgrade.' as \"description!\", ///     format( ///         'Table \\`%s.%s\\` has a column \\`%s\\` with unsupported reg* type \\`%s\\`.', ///         n.nspname, ///         c.relname, ///         a.attname, ///         t.typname ///     ) as \"detail!\", ///     'https://supabase.com/docs/guides/database/database-linter?lint=unsupported_reg_types' as \"remediation!\", ///     jsonb_build_object( ///         'schema', n.nspname, ///         'name', c.relname, ///         'column', a.attname, ///         'type', 'table' ///     ) as \"metadata!\", ///     format( ///         'unsupported_reg_types_%s_%s_%s', ///         n.nspname, ///         c.relname, ///         a.attname ///     ) AS cache_key /// from ///     pg_catalog.pg_attribute a ///     join pg_catalog.pg_class c ///         on a.attrelid = c.oid ///     join pg_catalog.pg_namespace n ///         on c.relnamespace = n.oid ///     join pg_catalog.pg_type t ///         on a.atttypid = t.oid ///     join pg_catalog.pg_namespace tn ///         on t.typnamespace = tn.oid /// where ///     tn.nspname = 'pg_catalog' ///     and t.typname in ('regcollation', 'regconfig', 'regdictionary', 'regnamespace', 'regoper', 'regoperator', 'regproc', 'regprocedure') ///     and n.nspname not in ('pg_catalog', 'information_schema', 'pgsodium')) ///  /// /// ## Configuration /// /// Enable or disable this rule in your configuration: /// /// json /// { ///   \"splinter\": { ///     \"rules\": { ///       \"security\": { ///         \"unsupportedRegTypes\": \"warn\" ///       } ///     } ///   } /// } ///  /// /// ## Remediation /// /// See: https://supabase.com/docs/guides/database/database-linter?lint=unsupported_reg_types"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unsupported_reg_types: Option<RuleConfiguration<()>>,
}
impl Security {
    const GROUP_NAME: &'static str = "security";
    pub(crate) const GROUP_RULES: &'static [&'static str] = &[
        "authUsersExposed",
        "extensionInPublic",
        "extensionVersionsOutdated",
        "fkeyToAuthUnique",
        "foreignTableInApi",
        "functionSearchPathMutable",
        "insecureQueueExposedInApi",
        "materializedViewInApi",
        "policyExistsRlsDisabled",
        "rlsDisabledInPublic",
        "rlsEnabledNoPolicy",
        "rlsReferencesUserMetadata",
        "securityDefinerView",
        "unsupportedRegTypes",
    ];
    const RECOMMENDED_RULES_AS_FILTERS: &'static [RuleFilter<'static>] = &[];
    const ALL_RULES_AS_FILTERS: &'static [RuleFilter<'static>] = &[
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[0]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[1]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[2]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[3]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[4]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[5]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[6]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[7]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[8]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[9]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[10]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[11]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[12]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[13]),
    ];
    #[doc = r" Retrieves the recommended rules"]
    pub(crate) fn is_recommended_true(&self) -> bool {
        matches!(self.recommended, Some(true))
    }
    pub(crate) fn is_recommended_unset(&self) -> bool {
        self.recommended.is_none()
    }
    pub(crate) fn is_all_true(&self) -> bool {
        matches!(self.all, Some(true))
    }
    pub(crate) fn is_all_unset(&self) -> bool {
        self.all.is_none()
    }
    pub(crate) fn get_enabled_rules(&self) -> FxHashSet<RuleFilter<'static>> {
        let mut index_set = FxHashSet::default();
        if let Some(rule) = self.auth_users_exposed.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[0]));
            }
        }
        if let Some(rule) = self.extension_in_public.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[1]));
            }
        }
        if let Some(rule) = self.extension_versions_outdated.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[2]));
            }
        }
        if let Some(rule) = self.fkey_to_auth_unique.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[3]));
            }
        }
        if let Some(rule) = self.foreign_table_in_api.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[4]));
            }
        }
        if let Some(rule) = self.function_search_path_mutable.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[5]));
            }
        }
        if let Some(rule) = self.insecure_queue_exposed_in_api.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[6]));
            }
        }
        if let Some(rule) = self.materialized_view_in_api.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[7]));
            }
        }
        if let Some(rule) = self.policy_exists_rls_disabled.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[8]));
            }
        }
        if let Some(rule) = self.rls_disabled_in_public.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[9]));
            }
        }
        if let Some(rule) = self.rls_enabled_no_policy.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[10]));
            }
        }
        if let Some(rule) = self.rls_references_user_metadata.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[11]));
            }
        }
        if let Some(rule) = self.security_definer_view.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[12]));
            }
        }
        if let Some(rule) = self.unsupported_reg_types.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[13]));
            }
        }
        index_set
    }
    pub(crate) fn get_disabled_rules(&self) -> FxHashSet<RuleFilter<'static>> {
        let mut index_set = FxHashSet::default();
        if let Some(rule) = self.auth_users_exposed.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[0]));
            }
        }
        if let Some(rule) = self.extension_in_public.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[1]));
            }
        }
        if let Some(rule) = self.extension_versions_outdated.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[2]));
            }
        }
        if let Some(rule) = self.fkey_to_auth_unique.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[3]));
            }
        }
        if let Some(rule) = self.foreign_table_in_api.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[4]));
            }
        }
        if let Some(rule) = self.function_search_path_mutable.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[5]));
            }
        }
        if let Some(rule) = self.insecure_queue_exposed_in_api.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[6]));
            }
        }
        if let Some(rule) = self.materialized_view_in_api.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[7]));
            }
        }
        if let Some(rule) = self.policy_exists_rls_disabled.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[8]));
            }
        }
        if let Some(rule) = self.rls_disabled_in_public.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[9]));
            }
        }
        if let Some(rule) = self.rls_enabled_no_policy.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[10]));
            }
        }
        if let Some(rule) = self.rls_references_user_metadata.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[11]));
            }
        }
        if let Some(rule) = self.security_definer_view.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[12]));
            }
        }
        if let Some(rule) = self.unsupported_reg_types.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[13]));
            }
        }
        index_set
    }
    #[doc = r" Checks if, given a rule name, matches one of the rules contained in this category"]
    pub(crate) fn has_rule(rule_name: &str) -> Option<&'static str> {
        Some(Self::GROUP_RULES[Self::GROUP_RULES.binary_search(&rule_name).ok()?])
    }
    pub(crate) fn recommended_rules_as_filters() -> &'static [RuleFilter<'static>] {
        Self::RECOMMENDED_RULES_AS_FILTERS
    }
    pub(crate) fn all_rules_as_filters() -> &'static [RuleFilter<'static>] {
        Self::ALL_RULES_AS_FILTERS
    }
    #[doc = r" Select preset rules"]
    pub(crate) fn collect_preset_rules(
        &self,
        parent_is_all: bool,
        parent_is_recommended: bool,
        enabled_rules: &mut FxHashSet<RuleFilter<'static>>,
    ) {
        if self.is_all_true() || self.is_all_unset() && parent_is_all {
            enabled_rules.extend(Self::all_rules_as_filters());
        } else if self.is_recommended_true()
            || self.is_recommended_unset() && self.is_all_unset() && parent_is_recommended
        {
            enabled_rules.extend(Self::recommended_rules_as_filters());
        }
    }
    pub(crate) fn severity(rule_name: &str) -> Severity {
        match rule_name {
            "authUsersExposed" => Severity::Error,
            "extensionInPublic" => Severity::Warning,
            "extensionVersionsOutdated" => Severity::Warning,
            "fkeyToAuthUnique" => Severity::Error,
            "foreignTableInApi" => Severity::Warning,
            "functionSearchPathMutable" => Severity::Warning,
            "insecureQueueExposedInApi" => Severity::Error,
            "materializedViewInApi" => Severity::Warning,
            "policyExistsRlsDisabled" => Severity::Error,
            "rlsDisabledInPublic" => Severity::Error,
            "rlsEnabledNoPolicy" => Severity::Information,
            "rlsReferencesUserMetadata" => Severity::Error,
            "securityDefinerView" => Severity::Error,
            "unsupportedRegTypes" => Severity::Warning,
            _ => unreachable!(),
        }
    }
    pub(crate) fn get_rule_configuration(
        &self,
        rule_name: &str,
    ) -> Option<(RulePlainConfiguration, Option<RuleOptions>)> {
        match rule_name {
            "authUsersExposed" => self
                .auth_users_exposed
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "extensionInPublic" => self
                .extension_in_public
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "extensionVersionsOutdated" => self
                .extension_versions_outdated
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "fkeyToAuthUnique" => self
                .fkey_to_auth_unique
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "foreignTableInApi" => self
                .foreign_table_in_api
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "functionSearchPathMutable" => self
                .function_search_path_mutable
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "insecureQueueExposedInApi" => self
                .insecure_queue_exposed_in_api
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "materializedViewInApi" => self
                .materialized_view_in_api
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "policyExistsRlsDisabled" => self
                .policy_exists_rls_disabled
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "rlsDisabledInPublic" => self
                .rls_disabled_in_public
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "rlsEnabledNoPolicy" => self
                .rls_enabled_no_policy
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "rlsReferencesUserMetadata" => self
                .rls_references_user_metadata
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "securityDefinerView" => self
                .security_definer_view
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "unsupportedRegTypes" => self
                .unsupported_reg_types
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            _ => None,
        }
    }
}
#[doc = r" Push the configured rules to the analyser"]
pub fn push_to_analyser_rules(
    rules: &Rules,
    metadata: &pgls_analyse::MetadataRegistry,
    analyser_rules: &mut pgls_analyser::LinterRules,
) {
    if let Some(rules) = rules.performance.as_ref() {
        for rule_name in Performance::GROUP_RULES {
            if let Some((_, Some(rule_options))) = rules.get_rule_configuration(rule_name) {
                if let Some(rule_key) = metadata.find_rule("performance", rule_name) {
                    analyser_rules.push_rule(rule_key, rule_options);
                }
            }
        }
    }
    if let Some(rules) = rules.security.as_ref() {
        for rule_name in Security::GROUP_RULES {
            if let Some((_, Some(rule_options))) = rules.get_rule_configuration(rule_name) {
                if let Some(rule_key) = metadata.find_rule("security", rule_name) {
                    analyser_rules.push_rule(rule_key, rule_options);
                }
            }
        }
    }
}
#[test]
fn test_order() {
    for items in Performance::GROUP_RULES.windows(2) {
        assert!(items[0] < items[1], "{} < {}", items[0], items[1]);
    }
    for items in Security::GROUP_RULES.windows(2) {
        assert!(items[0] < items[1], "{} < {}", items[0], items[1]);
    }
}
