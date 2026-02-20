-- meta: name = sensitiveColumnsExposed
-- meta: title = Sensitive Columns Exposed
-- meta: severity = ERROR
-- meta: category = SECURITY
-- meta: description = Detects tables exposed via API that contain columns with potentially sensitive data (PII, credentials, financial info) without RLS protection.
-- meta: remediation = https://supabase.com/docs/guides/database/database-linter?lint=0023_sensitive_columns_exposed
-- meta: requires_supabase = true

(
with sensitive_patterns as (
    select unnest(array[
        -- Authentication & Credentials
        'password', 'passwd', 'pwd', 'passphrase',
        'secret', 'secret_key', 'private_key', 'api_key', 'apikey',
        'auth_key', 'token', 'jwt', 'access_token', 'refresh_token',
        'oauth_token', 'session_token', 'bearer_token', 'auth_code',
        'session_id', 'session_key', 'session_secret',
        'recovery_code', 'backup_code', 'verification_code',
        'otp', 'two_factor', '2fa_secret', '2fa_code',
        -- Personal Identifiers
        'ssn', 'social_security', 'social_security_number',
        'driver_license', 'drivers_license', 'license_number',
        'passport_number', 'passport_id', 'national_id', 'tax_id',
        -- Financial Information
        'credit_card', 'card_number', 'cvv', 'cvc', 'cvn',
        'bank_account', 'account_number', 'routing_number',
        'iban', 'swift_code', 'bic',
        -- Health & Medical
        'health_record', 'medical_record', 'patient_id',
        'insurance_number', 'health_insurance', 'medical_insurance',
        'treatment',
        -- Device Identifiers
        'mac_address', 'macaddr', 'imei', 'device_uuid',
        -- Digital Keys & Certificates
        'pgp_key', 'gpg_key', 'ssh_key', 'certificate',
        'license_key', 'activation_key',
        -- Biometric Data
        'facial_recognition'
    ]) as pattern
),
exposed_tables as (
    select
        n.nspname as schema_name,
        c.relname as table_name,
        c.oid as table_oid
    from
        pg_catalog.pg_class c
        join pg_catalog.pg_namespace n
            on c.relnamespace = n.oid
    where
        c.relkind = 'r' -- regular tables
        and (
            pg_catalog.has_table_privilege('anon', c.oid, 'SELECT')
            or pg_catalog.has_table_privilege('authenticated', c.oid, 'SELECT')
        )
        and n.nspname = any(array(select trim(unnest(string_to_array(current_setting('pgrst.db_schemas', 't'), ',')))))
        and n.nspname not in (
            '_timescaledb_cache', '_timescaledb_catalog', '_timescaledb_config', '_timescaledb_internal', 'auth', 'cron', 'extensions', 'graphql', 'graphql_public', 'information_schema', 'net', 'pgmq', 'pgroonga', 'pgsodium', 'pgsodium_masks', 'pgtle', 'pgbouncer', 'pg_catalog', 'pgtle', 'realtime', 'repack', 'storage', 'supabase_functions', 'supabase_migrations', 'tiger', 'topology', 'vault'
        )
        -- Only flag tables without RLS enabled
        and not c.relrowsecurity
),
sensitive_columns as (
    select
        et.schema_name,
        et.table_name,
        a.attname as column_name,
        sp.pattern as matched_pattern
    from
        exposed_tables et
        join pg_catalog.pg_attribute a
            on a.attrelid = et.table_oid
            and a.attnum > 0
            and not a.attisdropped
        cross join sensitive_patterns sp
    where
        -- Match column name against sensitive patterns (case insensitive), allowing '-'/'_' variants
        replace(lower(a.attname), '-', '_') = sp.pattern
)
select
    'sensitive_columns_exposed' as "name!",
    'Sensitive Columns Exposed' as "title!",
    'ERROR' as "level!",
    'EXTERNAL' as "facing!",
    array['SECURITY'] as "categories!",
    'Detects tables exposed via API that contain columns with potentially sensitive data (PII, credentials, financial info) without RLS protection.' as "description!",
    format(
        'Table `%s.%s` is exposed via API without RLS and contains potentially sensitive column(s): %s. This may lead to data exposure.',
        schema_name,
        table_name,
        string_agg(distinct column_name, ', ' order by column_name)
    ) as "detail!",
    'https://supabase.com/docs/guides/database/database-linter?lint=0023_sensitive_columns_exposed' as "remediation!",
    jsonb_build_object(
        'schema', schema_name,
        'name', table_name,
        'type', 'table',
        'sensitive_columns', array_agg(distinct column_name order by column_name),
        'matched_patterns', array_agg(distinct matched_pattern order by matched_pattern)
    ) as "metadata!",
    format(
        'sensitive_columns_exposed_%s_%s',
        schema_name,
        table_name
    ) as "cache_key!"
from
    sensitive_columns
group by
    schema_name,
    table_name
order by
    schema_name,
    table_name)
