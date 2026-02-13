-- =============================================================================
-- pglinter Rules Configuration
-- =============================================================================
--
-- This file defines the comprehensive rule set for the pglinter PostgreSQL
-- extension. It creates the rules table that stores metadata for all
-- database analysis rules.
--
-- Rule Categories:
--   B-series: Base Database Rules (tables, indexes, primary keys, etc.)
--   C-series: Cluster Rules (configuration, security, performance)
--   S-series: Schema Rules (permissions, ownership, security)
--
-- Each rule includes:
--   - Rule code (e.g., B001, T003)
--   - Configurable warning/error thresholds
--   - Scope (BASE, CLUSTER, SCHEMA, TABLE)
--   - Descriptive metadata and fix suggestions
--   - SQL queries for analysis (q1/q2 fields)
--
-- Usage:
--   This file is automatically executed during extension installation
--   via pgrx's extension_sql_file! macro.
--
-- =============================================================================

CREATE TABLE IF NOT EXISTS pglinter.rules (
    id SERIAL PRIMARY KEY,
    name TEXT,
    code TEXT,
    enable BOOL DEFAULT TRUE,
    warning_level INT,
    error_level INT,
    scope TEXT,
    description TEXT,
    message TEXT,
    fixes TEXT [],
    q1 TEXT,
    q2 TEXT,
    q3 TEXT
);

-- Clear existing data and insert comprehensive rules
DELETE FROM pglinter.rules;


INSERT INTO pglinter.rules (
    name,
    code,
    warning_level,
    error_level,
    scope,
    description,
    message,
    fixes
) VALUES
-- Base Database Rules (B series)
(
    'HowManyTableWithoutPrimaryKey', 'B001', 1, 80, 'BASE',
    'Count number of tables without primary key.',
    '{0}/{1} table(s) without primary key exceed the {2} threshold: {3}%. Object list:\n{4}',
    ARRAY['create a primary key or change warning/error threshold']
),
(
    'HowManyRedudantIndex', 'B002', 1, 80, 'BASE',
    'Count number of redundant index vs nb index.',
    '{0}/{1} redundant(s) index exceed the {2} threshold: {3}%. Object list:\n{4}',
    ARRAY[
        'remove duplicated index or check if a constraint does not create a redundant index, or change warning/error threshold'
    ]
),
(
    'HowManyTableWithoutIndexOnFk', 'B003', 1, 80, 'BASE',
    'Count number of tables without index on foreign key.',
    '{0}/{1} table(s) without index on foreign key exceed the {2} threshold: {3}%. Object list:\n{4}',
    ARRAY['create a index on foreign key or change warning/error threshold']
),
(
    'HowManyUnusedIndex', 'B004', 20, 80, 'BASE',
    'Count number of unused index vs nb index (base on pg_stat_user_indexes, indexes associated to unique constraints are discard.)',
    '{0}/{1} unused index exceed the {2} threshold: {3}%. Object list:\n{4}',
    ARRAY['remove unused index or change warning/error threshold']
),
(
    'HowManyObjectsWithUppercase', 'B005', 20, 80, 'BASE',
    'Count number of objects with uppercase in name or in columns.',
    '{0}/{1} object(s) using uppercase for name or columns exceed the {2} threshold: {3}%. Object list:\n{4}',
    ARRAY['Do not use uppercase for any database objects']
),
(
    'HowManyTablesNeverSelected', 'B006', 1, 80, 'BASE',
    'Count number of table(s) that has never been selected.',
    '{0}/{1} table(s) are never selected the {2} threshold: {3}%. Object list:\n{4}',
    ARRAY[
        'Is it necessary to update/delete/insert rows in table(s) that are never selected ?'
    ]
),
(
    'HowManyTablesWithFkOutsideSchema', 'B007', 20, 80, 'BASE',
    'Count number of tables with foreign keys outside their schema.',
    '{0}/{1} table(s) with foreign keys outside schema exceed the {2} threshold: {3}%. Object list:\n{4}',
    ARRAY[
        'Consider restructuring schema design to keep related tables in same schema',
        'ask a dba'
    ]
),
(
    'HowManyTablesWithFkMismatch', 'B008', 1, 80, 'BASE',
    'Count number of tables with foreign keys that do not match the key reference type.',
    '{0}/{1} table(s) with foreign key mismatch exceed the {2} threshold: {3}%. Object list:\n{4}',
    ARRAY[
        'Consider column type adjustments to ensure foreign key matches referenced key type',
        'ask a dba'
    ]
),
(
    'HowManyTablesWithSameTrigger', 'B009', 20, 80, 'BASE',
    'Count number of tables using the same trigger vs nb table with their own triggers.',
    '{0}/{1} table(s) using the same trigger function exceed the {2} threshold: {3}%. Object list:\n{4}',
    ARRAY[
        'For more readability and other considerations use one trigger function per table.',
        'Sharing the same trigger function add more complexity.'
    ]
),
(
    'HowManyTablesWithReservedKeywords', 'B010', 20, 80, 'BASE',
    'Count number of database objects using reserved keywords in their names.',
    '{0}/{1} object(s) using reserved keywords exceed the {2} threshold: {3}%. Object list:\n{4}',
    ARRAY[
        'Rename database objects to avoid using reserved keywords.',
        'Using reserved keywords can lead to SQL syntax errors and maintenance difficulties.'
    ]
),
(
    'SeveralTableOwnerInSchema', 'B011', 1, 80, 'BASE',
    'In a schema there are several tables owned by different owners.',
    '{0}/{1} schemas have tables owned by different owners. Exceed the {2} threshold: {3}%. Object list:\n{4}',
    ARRAY['change table owners to the same functional role']
),
(
    'CompositePrimaryKeyTooManyColumns', 'B012', 1, 80, 'BASE',
    'Detect tables with composite primary keys involving more than 4 columns',
    '{0} table(s) have composite primary keys with more than 4 columns. Object list:\n{4}',
    ARRAY[
      'Consider redesigning the table to avoid composite primary keys with more than 4 columns',
      'Use surrogate keys (e.g., serial, UUID) instead of composite primary keys, and establish unique constraints on necessary column combinations, to enforce uniqueness.'
    ]
),
(
    'SchemaWithDefaultRoleNotGranted', 'S001', 1, 1, 'SCHEMA',
    'The schema has no default role. Means that futur table will not be granted through a role. So you will have to re-execute grants on it.',
    'No default role grantee on schema {0}.{1}. It means that each time a table is created, you must grant it to roles. Object list:\n{4}',
    ARRAY[
        'add a default privilege=> ALTER DEFAULT PRIVILEGES IN SCHEMA <schema> for user <schema''s owner>'
    ]
),

(
    'SchemaPrefixedOrSuffixedWithEnvt', 'S002', 1, 1, 'SCHEMA',
    'The schema is prefixed with one of staging,stg,preprod,prod,sandbox,sbox string. Means that when you refresh your preprod, staging environments from production, you have to rename the target schema from prod_ to stg_ or something like. It is possible, but it is never easy.',
    '{0}/{1} schemas are prefixed or suffixed with environment names. It exceed the {2} threshold: {3}%. Prefer prefix or suffix the database name instead. Object list:\n{4}',
    ARRAY[
        'Keep the same schema name across environments. Prefer prefix or suffix the database name'
    ]
),
(
    'UnsecuredPublicSchema', 'S003', 1, 80, 'SCHEMA',
    'Only authorized users should be allowed to create objects.',
    '{0}/{1} schemas are unsecured, schemas where all users can create objects in, exceed the {2} threshold: {3}%. Object list:\n{4}',
    ARRAY['REVOKE CREATE ON SCHEMA <schema_name> FROM PUBLIC']
),
(
    'OwnerSchemaIsInternalRole', 'S004', 20, 80, 'SCHEMA',
    'Owner of schema should not be any internal pg roles, or owner is a superuser (not sure it is necesary).',
    '{0}/{1} schemas are owned by internal roles or superuser. Exceed the {2} threshold: {3}%. Object list:\n{4}',
    ARRAY['change schema owner to a functional role']
),
(
    'SchemaOwnerDoNotMatchTableOwner', 'S005', 20, 80, 'SCHEMA',
    'The schema owner and tables in the schema do not match.',
    '{0}/{1} in the same schema, tables have different owners. They should be the same. Exceed the {2} threshold: {3}%. Object list:\n{4}',
    ARRAY['For maintenance facilities, schema and tables owners should be the same.']
),
(
    'PgHbaEntriesWithMethodTrustShouldNotExists',
    'C001',
    20,
    80,
    'CLUSTER',
    'This configuration is extremely insecure and should only be used in a controlled, non-production environment for testing purposes. In a production environment, you should use more secure authentication methods such as md5, scram-sha-256, or cert, and restrict access to trusted IP addresses only.',
    '{0} entries in pg_hba.conf with trust authentication method exceed the warning threshold: {1}.',
    ARRAY['change trust method in pg_hba.conf']
),
(
    'PgHbaEntriesWithMethodTrustOrPasswordShouldNotExists',
    'C002',
    20,
    80,
    'CLUSTER',
    'This configuration is extremely insecure and should only be used in a controlled, non-production environment for testing purposes. In a production environment, you should use more secure authentication methods such as md5, scram-sha-256, or cert, and restrict access to trusted IP addresses only.',
    '{0} entries in pg_hba.conf with trust or password authentication method exceed the warning threshold: {1}.',
    ARRAY['change trust or password method in pg_hba.conf']
),
(
    'PasswordEncryptionIsMd5',
    'C003',
    20,
    80,
    'CLUSTER',
    'This configuration is not secure anymore and will prevent an upgrade to Postgres 18. Warning, you will need to reset all passwords after this is changed to scram-sha-256.',
    'Encrypted passwords with MD5.',
    ARRAY['change password_encryption parameter to scram-sha-256 (ALTER SYSTEM SET password_encryption = ''scram-sha-256'' ). Warning, you will need to reset all passwords after this parameter is updated.']
);


-- =============================================================================
-- RULE QUERY UPDATES - Auto-generated from individual SQL files
-- =============================================================================
-- The following UPDATE statements populate the q1 and q2 columns
-- with SQL queries extracted from individual *q*.sql files.
-- These queries are used by the pglinter engine to execute rule checks.
-- =============================================================================

-- B001 - Tables Without Primary Key
UPDATE pglinter.rules
SET
    q1 = $$
SELECT count(*)::BIGINT AS total_tables
FROM pg_catalog.pg_tables
WHERE
    schemaname NOT IN (
        'pg_toast', 'pg_catalog', 'information_schema', 'pglinter','_timescaledb', 'timescaledb'
    )
$$,
    q2 = $$
SELECT
count(1)::BIGINT AS tables_without_primary_key
FROM
    pg_class c
JOIN
    pg_namespace n ON n.oid = c.relnamespace
LEFT JOIN
    pg_index i ON i.indrelid = c.oid AND i.indisprimary
WHERE
    n.nspname NOT IN ('pg_catalog', 'information_schema', 'gp_toolkit','_timescaledb', 'timescaledb') -- Exclude system schemas
    AND c.relkind = 'r' -- Only include regular tables
    AND i.indrelid IS NULL
$$,
    q3 = $$
SELECT pt.schemaname::text,pt.tablename::text
FROM pg_tables AS pt
WHERE
    pt.schemaname NOT IN (
        'pg_toast', 'pg_catalog', 'information_schema', 'pglinter','_timescaledb', 'timescaledb'
    )
    AND NOT EXISTS (
        SELECT 1
        FROM pg_constraint AS pc
        WHERE
            pc.conrelid = (
                SELECT pg_class.oid
                FROM pg_class
                JOIN pg_namespace ON pg_class.relnamespace = pg_namespace.oid
                WHERE
                    pg_class.relname = pt.tablename
                    AND pg_namespace.nspname = pt.schemaname
            )
            AND pc.contype = 'p'
    )
ORDER BY 1
$$
WHERE code = 'B001';


-- =============================================================================
-- B002 - Redundant Indexes (Total Index Count Query)
-- =============================================================================
UPDATE pglinter.rules
SET
    q1 = $$
SELECT COUNT(*) AS total_indexes
FROM pg_indexes
WHERE
    schemaname NOT IN (
        'pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb'
    )
$$,
    q2 = $$
SELECT COUNT(*) AS redundant_indexes
FROM (
    SELECT DISTINCT i1.indexrelid
    FROM pg_index i1, pg_index i2
    WHERE
        i1.indrelid = i2.indrelid
        AND i1.indexrelid != i2.indexrelid
        AND i1.indkey = i2.indkey
        AND EXISTS (
            SELECT 1 FROM pg_indexes pi1
            WHERE
                pi1.indexname
                = (
                    SELECT relname FROM pg_class
                    WHERE oid = i1.indexrelid
                )
                AND pi1.schemaname NOT IN (
                    'pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb'
                )
        )
) redundant
$$,
    q3 = $$
WITH index_info AS (
    -- This CTE gets the column info, plus the boolean flag for Primary Key (indisprimary).
    SELECT
        ind.indrelid AS table_oid,
        ind.indexrelid AS index_oid,
        att.attname AS column_name,
        array_position(ind.indkey, att.attnum) AS column_order,
        ind.indisprimary -- Added Primary Key flag
    FROM pg_index ind
    JOIN pg_attribute att ON att.attrelid = ind.indrelid AND att.attnum = ANY(ind.indkey)
    WHERE NOT ind.indisexclusion
),
indexed_columns AS (
    -- Aggregates columns for each index and propagates PK flag.
    SELECT
        table_oid,
        index_oid,
        string_agg(column_name, ',' ORDER BY column_order) AS indexed_columns_string,
        MAX(indisprimary::int)::bool AS is_primary_key
    FROM index_info
    GROUP BY table_oid, index_oid
),
table_info AS (
    -- Joins to pg_class and pg_namespace to get table names and schema names.
    SELECT
        oid AS table_oid,
        relname AS tablename,
        relnamespace
    FROM pg_class
)
SELECT
    pg_namespace.nspname::TEXT AS schema_name,
    table_info.tablename::TEXT AS table_name,
    redundant_index.relname::TEXT ||'('|| i1.indexed_columns_string || ') is redundant with '|| superset_index.relname||'('|| i2.indexed_columns_string ||')' AS problematic_object
FROM indexed_columns AS i1 -- The smaller/redundant index
JOIN indexed_columns AS i2 ON i1.table_oid = i2.table_oid -- The larger/superset index
JOIN pg_class redundant_index ON i1.index_oid = redundant_index.oid
JOIN pg_class superset_index ON i2.index_oid = superset_index.oid
JOIN table_info ON i1.table_oid = table_info.table_oid
JOIN pg_namespace ON table_info.relnamespace = pg_namespace.oid
WHERE
    pg_namespace.nspname NOT IN ('pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb')
    AND i1.index_oid <> i2.index_oid -- Ensure the indexes are not the same
    -- Checks if the smaller index's column string is a prefix of the larger index's string.
    AND i2.indexed_columns_string LIKE i1.indexed_columns_string || '%'

ORDER BY 1, 2
$$
WHERE code = 'B002';

-- =============================================================================
-- B003 - Foreign Key without Index
-- =============================================================================
UPDATE pglinter.rules
SET
    q1 = $$
SELECT count(DISTINCT tc.table_name)::BIGINT AS total_tables
FROM
    information_schema.table_constraints AS tc
WHERE
    tc.constraint_type = 'FOREIGN KEY'
    AND tc.table_schema NOT IN (
        'pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb'
    )
$$,
    q2 = $$
SELECT COUNT(DISTINCT c.relname)::INT AS tables_with_unindexed_foreign_keys
FROM pg_constraint con
JOIN pg_class c ON c.oid = con.conrelid
JOIN pg_namespace n ON n.oid = c.relnamespace
LEFT JOIN
    pg_index i
    ON i.indrelid = c.oid AND con.conkey::smallint [] <@ i.indkey::smallint []
WHERE
    con.contype = 'f'
    AND c.relkind = 'r'
    AND i.indexrelid IS NULL
    AND n.nspname NOT IN ('pg_catalog', 'pg_toast', 'information_schema', 'pglinter','_timescaledb', 'timescaledb')
$$,
    q3 = $$
SELECT DISTINCT
    tc.table_schema::text,
    tc.table_name::text,
    tc.constraint_name::text AS problematic_object
FROM information_schema.table_constraints AS tc
INNER JOIN information_schema.key_column_usage AS kcu
    ON
        tc.constraint_name = kcu.constraint_name
        AND tc.table_schema = kcu.table_schema
WHERE
    tc.constraint_type = 'FOREIGN KEY'
    AND tc.table_schema NOT IN (
        'pg_toast', 'pg_catalog', 'information_schema', 'pglinter','_timescaledb', 'timescaledb'
    )
    AND NOT EXISTS (
        SELECT 1 FROM pg_indexes AS pi
        WHERE
            pi.schemaname = tc.table_schema
            AND pi.tablename = tc.table_name
            AND pi.indexdef LIKE '%' || kcu.column_name || '%'
    )
ORDER BY 1
$$
WHERE code = 'B003';

-- =============================================================================
-- B004 - Manual Index Usage (Total)
-- =============================================================================
UPDATE pglinter.rules
SET
    q1 = $$
SELECT COUNT(*) AS total_manual_indexes
FROM pg_stat_user_indexes AS psu
JOIN pg_index AS pgi ON psu.indexrelid = pgi.indexrelid
WHERE
    pgi.indisprimary = FALSE -- Excludes indexes created for a PRIMARY KEY
    -- Excludes indexes created for a UNIQUE constraint
    AND pgi.indisunique = FALSE
    AND psu.schemaname NOT IN (
        'pg_toast', 'pg_catalog', 'information_schema', 'pglinter','_timescaledb', 'timescaledb'
    )
$$,
    q2 = $$
SELECT COUNT(*) AS unused_manual_indexes
FROM pg_stat_user_indexes AS psu
JOIN pg_index AS pgi ON psu.indexrelid = pgi.indexrelid
WHERE
    psu.idx_scan = 0
    AND pgi.indisprimary = FALSE -- Excludes indexes created for a PRIMARY KEY
    -- Excludes indexes created for a UNIQUE constraint
    AND pgi.indisunique = FALSE
    AND psu.schemaname NOT IN (
        'pg_toast', 'pg_catalog', 'information_schema', 'pglinter','_timescaledb', 'timescaledb'
    )
$$,
    q3 = $$
SELECT
    schemaname::text,
    relname::text || 'has' ||
    LEAST(
        ROUND(
            (
                seq_tup_read::numeric
                / NULLIF((seq_tup_read + idx_tup_fetch)::numeric, 0)
            ) * 100, 0
        ),
        100
    )::text ||' % of seq scan.' AS problematic_object
FROM pg_stat_user_tables
WHERE
    schemaname NOT IN (
        'pg_toast', 'pg_catalog', 'information_schema', 'pglinter','_timescaledb', 'timescaledb'
    )
ORDER BY 1, 2
$$
WHERE code = 'B004';


-- =============================================================================
-- B005 - Objects With Uppercase (Total)
-- =============================================================================
UPDATE pglinter.rules
SET
    q1 = $$
SELECT COUNT(*) AS total_objects
FROM (
    -- All tables
    SELECT
        'table' AS object_type,
        table_schema AS schema_name,
        table_name AS object_name
    FROM information_schema.tables
    WHERE
        table_schema NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter','_timescaledb', 'timescaledb'
        )

    UNION

    -- All columns
    SELECT
        'column' AS object_type,
        table_schema AS schema_name,
        table_name || '.' || column_name AS object_name
    FROM information_schema.columns
    WHERE
        table_schema NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter','_timescaledb', 'timescaledb'
        )

    UNION

    -- All indexes
    SELECT
        'index' AS object_type,
        schemaname AS schema_name,
        indexname AS object_name
    FROM pg_indexes
    WHERE
        schemaname NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter','_timescaledb', 'timescaledb'
        )

    UNION

    -- All sequences
    SELECT
        'sequence' AS object_type,
        sequence_schema AS schema_name,
        sequence_name AS object_name
    FROM information_schema.sequences
    WHERE
        sequence_schema NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter','_timescaledb', 'timescaledb'
        )

    UNION

    -- All views
    SELECT
        'view' AS object_type,
        table_schema AS schema_name,
        table_name AS object_name
    FROM information_schema.views
    WHERE
        table_schema NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter','_timescaledb', 'timescaledb'
        )

    UNION

    -- All functions
    SELECT
        'function' AS object_type,
        routine_schema AS schema_name,
        routine_name AS object_name
    FROM information_schema.routines
    WHERE
        routine_schema NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter','_timescaledb', 'timescaledb'
        )
        AND routine_type = 'FUNCTION'

    UNION

    -- All triggers
    SELECT
        'trigger' AS object_type,
        trigger_schema AS schema_name,
        trigger_name AS object_name
    FROM information_schema.triggers
    WHERE
        trigger_schema NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter','_timescaledb', 'timescaledb'
        )

    UNION

    -- All schemas
    SELECT
        'schema' AS object_type,
        schema_name AS schema_name,
        schema_name AS object_name
    FROM information_schema.schemata
    WHERE
        schema_name NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter','_timescaledb', 'timescaledb'
        )
) all_objects
$$,
    q2 = $$
SELECT COUNT(*) AS uppercase_objects
FROM (
    -- Tables with uppercase names
    SELECT
        'table' AS object_type,
        table_schema AS schema_name,
        table_name AS object_name
    FROM information_schema.tables
    WHERE
        table_schema NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter','_timescaledb', 'timescaledb'
        )
        AND table_name != LOWER(table_name)

    UNION

    -- Columns with uppercase names
    SELECT
        'column' AS object_type,
        table_schema AS schema_name,
        table_name || '.' || column_name AS object_name
    FROM information_schema.columns
    WHERE
        table_schema NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter','_timescaledb', 'timescaledb'
        )
        AND column_name != LOWER(column_name)

    UNION

    -- Indexes with uppercase names
    SELECT
        'index' AS object_type,
        schemaname AS schema_name,
        indexname AS object_name
    FROM pg_indexes
    WHERE
        schemaname NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter','_timescaledb', 'timescaledb'
        )
        AND indexname != LOWER(indexname)

    UNION

    -- Sequences with uppercase names
    SELECT
        'sequence' AS object_type,
        sequence_schema AS schema_name,
        sequence_name AS object_name
    FROM information_schema.sequences
    WHERE
        sequence_schema NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter','_timescaledb', 'timescaledb'
        )
        AND sequence_name != LOWER(sequence_name)

    UNION

    -- Views with uppercase names
    SELECT
        'view' AS object_type,
        table_schema AS schema_name,
        table_name AS object_name
    FROM information_schema.views
    WHERE
        table_schema NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter','_timescaledb', 'timescaledb'
        )
        AND table_name != LOWER(table_name)

    UNION

    -- Functions with uppercase names
    SELECT
        'function' AS object_type,
        routine_schema AS schema_name,
        routine_name AS object_name
    FROM information_schema.routines
    WHERE
        routine_schema NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter','_timescaledb', 'timescaledb'
        )
        AND routine_type = 'FUNCTION'
        AND routine_name != LOWER(routine_name)

    UNION

    -- Triggers with uppercase names
    SELECT
        'trigger' AS object_type,
        trigger_schema AS schema_name,
        trigger_name AS object_name
    FROM information_schema.triggers
    WHERE
        trigger_schema NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter','_timescaledb', 'timescaledb'
        )
        AND trigger_name != LOWER(trigger_name)

    UNION

    -- Schemas with uppercase names
    SELECT
        'schema' AS object_type,
        schema_name AS schema_name,
        schema_name AS object_name
    FROM information_schema.schemata
    WHERE
        schema_name NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter','_timescaledb', 'timescaledb'
        )
        AND schema_name != LOWER(schema_name)
) uppercase_objects
$$,
    q3 = $$
SELECT
    object_type::TEXT ||' '||schema_name::TEXT,
    object_name::TEXT AS problematic_object
FROM (
    -- Tables with uppercase names
    SELECT
        'table' AS object_type,
        table_schema AS schema_name,
        table_name AS object_name
    FROM information_schema.tables
    WHERE
        table_schema NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter','_timescaledb', 'timescaledb'
        )
        AND table_name != LOWER(table_name)

    UNION ALL

    -- Columns with uppercase names
    SELECT
        'column' AS object_type,
        table_schema AS schema_name,
        table_name || '.' || column_name AS object_name
    FROM information_schema.columns
    WHERE
        table_schema NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter','_timescaledb', 'timescaledb'
        )
        AND column_name != LOWER(column_name)

    UNION ALL

    -- Indexes with uppercase names
    SELECT
        'index' AS object_type,
        schemaname AS schema_name,
        indexname AS object_name
    FROM pg_indexes
    WHERE
        schemaname NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter','_timescaledb', 'timescaledb'
        )
        AND indexname != LOWER(indexname)

    UNION ALL

    -- Sequences with uppercase names
    SELECT
        'sequence' AS object_type,
        sequence_schema AS schema_name,
        sequence_name AS object_name
    FROM information_schema.sequences
    WHERE
        sequence_schema NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter','_timescaledb', 'timescaledb'
        )
        AND sequence_name != LOWER(sequence_name)

    UNION ALL

    -- Views with uppercase names
    SELECT
        'view' AS object_type,
        table_schema AS schema_name,
        table_name AS object_name
    FROM information_schema.views
    WHERE
        table_schema NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter','_timescaledb', 'timescaledb'
        )
        AND table_name != LOWER(table_name)

    UNION ALL

    -- Functions with uppercase names
    SELECT
        'function' AS object_type,
        routine_schema AS schema_name,
        routine_name AS object_name
    FROM information_schema.routines
    WHERE
        routine_schema NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter','_timescaledb', 'timescaledb'
        )
        AND routine_type = 'FUNCTION'
        AND routine_name != LOWER(routine_name)

    UNION ALL

    -- Triggers with uppercase names
    SELECT
        'trigger' AS object_type,
        trigger_schema AS schema_name,
        trigger_name AS object_name
    FROM information_schema.triggers
    WHERE
        trigger_schema NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter','_timescaledb', 'timescaledb'
        )
        AND trigger_name != LOWER(trigger_name)

    UNION ALL

    -- Schemas with uppercase names
    SELECT
        'schema' AS object_type,
        schema_name AS schema_name,
        schema_name AS object_name
    FROM information_schema.schemata
    WHERE
        schema_name NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter','_timescaledb', 'timescaledb'
        )
        AND schema_name != LOWER(schema_name)
) AS uppercase_objects
ORDER BY
    object_type,
    schema_name,
    object_name
$$
WHERE code = 'B005';

-- =============================================================================
-- B006 - Tables Never Selected (Total)
-- =============================================================================
UPDATE pglinter.rules
SET
  q1 = $$
SELECT count(*)::BIGINT
FROM pg_catalog.pg_tables pt
WHERE
    schemaname NOT IN (
        'pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb'
    )
$$,
  q2 = $$
SELECT COUNT(*) AS unselected_tables
FROM pg_stat_user_tables AS psu
WHERE
    (psu.idx_scan = 0 OR psu.idx_scan IS NULL)
    AND (psu.seq_scan = 0 OR psu.seq_scan IS NULL)
    AND n_tup_ins > 0
    AND (n_tup_upd = 0 OR n_tup_upd IS NULL)
    AND (n_tup_del = 0 OR n_tup_del IS NULL)
    AND psu.schemaname NOT IN (
        'pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb'
    )
$$,
  q3 = $$
SELECT psu.schemaname::text, psu.relname::text
FROM pg_stat_user_tables AS psu
WHERE
    (psu.idx_scan = 0 OR psu.idx_scan IS NULL)
    AND (psu.seq_scan = 0 OR psu.seq_scan IS NULL)
    AND n_tup_ins > 0
    AND (n_tup_upd = 0 OR n_tup_upd IS NULL)
    AND (n_tup_del = 0 OR n_tup_del IS NULL)
    AND psu.schemaname NOT IN (
        'pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb'
    )
$$
WHERE code = 'B006';


-- =============================================================================
-- B007 - Tables With FK Outside Schema (Total)
-- =============================================================================
UPDATE pglinter.rules
SET
    q1 = $$
SELECT
    COUNT(DISTINCT conrelid::regclass) AS tables_with_foreign_keys
FROM
    pg_constraint c
JOIN
    pg_class r ON r.oid = c.conrelid
JOIN
    pg_namespace n ON n.oid = r.relnamespace
WHERE
    c.contype = 'f' -- Filter for Foreign Key constraints
    AND n.nspname NOT IN (
        'pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb'
    )
$$,
    q2 = $$
SELECT
    COUNT(
        DISTINCT tc.table_schema || '.' || tc.table_name
    ) AS tables_with_fk_outside_schema
FROM information_schema.table_constraints AS tc
INNER JOIN information_schema.constraint_column_usage AS ccu
    ON tc.constraint_name = ccu.constraint_name
WHERE
    tc.constraint_type = 'FOREIGN KEY'
    AND tc.table_schema != ccu.table_schema
    AND tc.table_schema NOT IN (
        'pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb'
    )
$$,
    q3 = $$
SELECT
    tc.table_schema::TEXT,tc.table_name::TEXT,
    'has foreign key '||tc.constraint_name::TEXT||' referencing '||
    ccu.table_schema::TEXT||'.'||ccu.table_name::TEXT AS problematic_object
FROM information_schema.table_constraints AS tc
INNER JOIN information_schema.constraint_column_usage AS ccu
    ON tc.constraint_name = ccu.constraint_name
WHERE
    tc.constraint_type = 'FOREIGN KEY'
    AND tc.table_schema != ccu.table_schema
    AND tc.table_schema NOT IN (
        'pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb'
    )
$$
WHERE code = 'B007';

-- =============================================================================
-- B008 - Tables With FK mismatch
-- =============================================================================
UPDATE pglinter.rules
SET
    q1 = $$
SELECT
    COUNT(DISTINCT conrelid::regclass) AS tables_with_foreign_keys
FROM
    pg_constraint c
JOIN
    pg_class r ON r.oid = c.conrelid
JOIN
    pg_namespace n ON n.oid = r.relnamespace
WHERE
    c.contype = 'f' -- Filter for Foreign Key constraints
    AND n.nspname NOT IN (
        'pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb'
    )
$$,
    q2 = $$
SELECT
    count(1)::BIGINT AS fk_type_mismatches
FROM information_schema.table_constraints AS tc
INNER JOIN information_schema.key_column_usage AS kcu
    ON
        tc.constraint_name = kcu.constraint_name
        AND tc.table_schema = kcu.table_schema
INNER JOIN information_schema.constraint_column_usage AS ccu
    ON tc.constraint_name = ccu.constraint_name
INNER JOIN information_schema.columns AS col1
    ON
        kcu.table_schema = col1.table_schema
        AND kcu.table_name = col1.table_name
        AND kcu.column_name = col1.column_name
INNER JOIN information_schema.columns AS col2
    ON
        ccu.table_schema = col2.table_schema
        AND ccu.table_name = col2.table_name
        AND ccu.column_name = col2.column_name
WHERE
    tc.constraint_type = 'FOREIGN KEY'
    AND tc.table_schema NOT IN (
        'pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb'
    )
    AND col1.data_type != col2.data_type
$$,
    q3 = $$
SELECT
    tc.table_schema::text || '.'
    || tc.table_name::text || ' constraint '
    || tc.constraint_name::text || ' column '
    || kcu.column_name::text || ' type is '
    || col1.data_type::text || ' but '
    || ccu.table_name::text || '.'
    || ccu.column_name::text || ' type is '
    || col2.data_type::text AS problematic_object
FROM information_schema.table_constraints AS tc
INNER JOIN information_schema.key_column_usage AS kcu
    ON
        tc.constraint_name = kcu.constraint_name
        AND tc.table_schema = kcu.table_schema
INNER JOIN information_schema.constraint_column_usage AS ccu
    ON tc.constraint_name = ccu.constraint_name
INNER JOIN information_schema.columns AS col1
    ON
        kcu.table_schema = col1.table_schema
        AND kcu.table_name = col1.table_name
        AND kcu.column_name = col1.column_name
INNER JOIN information_schema.columns AS col2
    ON
        ccu.table_schema = col2.table_schema
        AND ccu.table_name = col2.table_name
        AND ccu.column_name = col2.column_name
WHERE
    tc.constraint_type = 'FOREIGN KEY'
    AND tc.table_schema NOT IN (
        'pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb'
    )
    AND col1.data_type != col2.data_type
$$
WHERE code = 'B008';

-- =============================================================================
-- B009 - Tables With same trigger
-- =============================================================================
UPDATE pglinter.rules
SET q1 = $$
SELECT
    COALESCE(COUNT(DISTINCT event_object_table), 0)::BIGINT as table_using_trigger
FROM
    information_schema.triggers t
WHERE
    t.trigger_schema NOT IN (
    'pg_toast', 'pg_catalog', 'information_schema', 'pglinter'
)
$$,
  q2 = $$
SELECT
    COALESCE(SUM(shared_table_count), 0)::BIGINT AS table_using_same_trigger
FROM (
    SELECT
        COUNT(DISTINCT t.event_object_table) AS shared_table_count
    FROM (
        SELECT
            t.event_object_table,
            -- Extracts the function name from the action_statement (e.g., 'public.my_func()')
            SUBSTRING(t.action_statement FROM 'EXECUTE FUNCTION ([^()]+)') AS trigger_function_name
        FROM
            information_schema.triggers t
        WHERE
            t.trigger_schema NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter'
        )
    ) t
    GROUP BY
        t.trigger_function_name
    HAVING
        COUNT(DISTINCT t.event_object_table) > 1
) shared_triggers
$$,
  q3 = $$
WITH SharedFunctions AS (
    -- 1. Identify all trigger functions that are used by more than one table
    SELECT
        SUBSTRING(t.action_statement FROM 'EXECUTE FUNCTION ([^()]+)') AS trigger_function_name
    FROM
        information_schema.triggers t
    WHERE
        t.trigger_schema NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb'
        )
    GROUP BY
        1
    HAVING
        COUNT(DISTINCT t.event_object_table) > 1
)
SELECT
    t.event_object_table::TEXT AS table_name,
    t.trigger_name::TEXT || ' uses the same trigger function ' ||
    t.trigger_schema::TEXT,
    s.trigger_function_name::TEXT
FROM
    information_schema.triggers t
JOIN
    SharedFunctions s ON s.trigger_function_name = SUBSTRING(t.action_statement FROM 'EXECUTE FUNCTION ([^()]+)')
WHERE
    t.trigger_schema NOT IN (
        'pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb'
    )
ORDER BY
    s.trigger_function_name,
    t.trigger_schema,
    t.event_object_table
$$
WHERE code = 'B009';


-- =============================================================================
-- B010 - Tables With Reserved Keywords
-- =============================================================================
UPDATE pglinter.rules
  SET q1 = $$
SELECT count(*)::BIGINT AS total_tables
FROM pg_catalog.pg_tables
WHERE
    schemaname NOT IN (
        'pg_toast', 'pg_catalog', 'information_schema', 'pglinter','_timescaledb', 'timescaledb'
    )
$$,
  q2 = $$
WITH reserved_keywords AS (
    SELECT UNNEST(ARRAY[
        'ALL', 'ANALYSE', 'ANALYZE', 'AND', 'ANY', 'ARRAY', 'AS', 'ASC',
        'ASYMMETRIC', 'AUTHORIZATION', 'BINARY', 'BOTH', 'CASE', 'CAST',
        'CHECK', 'COLLATE', 'COLLATION', 'COLUMN', 'CONCURRENTLY',
        'CONSTRAINT', 'CREATE', 'CROSS', 'CURRENT_CATALOG', 'CURRENT_DATE',
        'CURRENT_ROLE', 'CURRENT_SCHEMA', 'CURRENT_TIME', 'CURRENT_TIMESTAMP',
        'CURRENT_USER', 'DEFAULT', 'DEFERRABLE', 'DESC', 'DISTINCT', 'DO',
        'ELSE', 'END', 'EXCEPT', 'FALSE', 'FETCH', 'FOR', 'FOREIGN', 'FROM',
        'FULL', 'GRANT', 'GROUP', 'HAVING', 'IN', 'INITIALLY', 'INNER',
        'INTERSECT', 'INTO', 'IS', 'ISNULL', 'JOIN', 'LATERAL', 'LEADING',
        'LEFT', 'LIKE', 'LIMIT', 'LOCALTIME', 'LOCALTIMESTAMP', 'NATURAL',
        'NOT', 'NOTNULL', 'NULL', 'OFFSET', 'ON', 'ONLY', 'OR', 'ORDER',
        'OUTER', 'OVERLAPS', 'PLACING', 'PRIMARY', 'REFERENCES', 'RETURNING',
        'RIGHT', 'SELECT', 'SESSION_USER', 'SIMILAR', 'SOME', 'SYMMETRIC',
        'TABLE', 'TABLESAMPLE', 'THEN', 'TO', 'TRAILING', 'TRUE', 'UNION',
        'UNIQUE', 'USER', 'USING', 'VARIADIC', 'VERBOSE', 'WHEN', 'WHERE',
        'WINDOW', 'WITH'
    ]) AS keyword
)
SELECT
    COUNT(1) AS total_reserved_keyword_objects
FROM (
    -- Tables using reserved keywords
    SELECT
        'table' AS object_type,
        table_schema AS schema_name,
        table_name AS object_name
    FROM information_schema.tables
    CROSS JOIN reserved_keywords
    WHERE
        table_schema NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb'
        )
        AND UPPER(table_name) = keyword

    UNION ALL -- Use UNION ALL for counting to avoid redundant DISTINCT check

    -- Columns using reserved keywords
    SELECT
        'column' AS object_type,
        table_schema AS schema_name,
        table_name || '.' || column_name AS object_name
    FROM information_schema.columns
    CROSS JOIN reserved_keywords
    WHERE
        table_schema NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb'
        )
        AND UPPER(column_name) = keyword

    UNION ALL

    -- Indexes using reserved keywords
    SELECT
        'index' AS object_type,
        schemaname AS schema_name,
        indexname AS object_name
    FROM pg_indexes
    CROSS JOIN reserved_keywords
    WHERE
        schemaname NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb'
        )
        AND UPPER(indexname) = keyword
) reserved_objects
$$,
  q3 = $$
WITH reserved_keywords AS (
    SELECT UNNEST(ARRAY[
        'ALL', 'ANALYSE', 'ANALYZE', 'AND', 'ANY', 'ARRAY', 'AS', 'ASC',
        'ASYMMETRIC', 'AUTHORIZATION', 'BINARY', 'BOTH', 'CASE', 'CAST',
        'CHECK', 'COLLATE', 'COLLATION', 'COLUMN', 'CONCURRENTLY',
        'CONSTRAINT', 'CREATE', 'CROSS', 'CURRENT_CATALOG', 'CURRENT_DATE',
        'CURRENT_ROLE', 'CURRENT_SCHEMA', 'CURRENT_TIME', 'CURRENT_TIMESTAMP',
        'CURRENT_USER', 'DEFAULT', 'DEFERRABLE', 'DESC', 'DISTINCT', 'DO',
        'ELSE', 'END', 'EXCEPT', 'FALSE', 'FETCH', 'FOR', 'FOREIGN', 'FROM',
        'FULL', 'GRANT', 'GROUP', 'HAVING', 'IN', 'INITIALLY', 'INNER',
        'INTERSECT', 'INTO', 'IS', 'ISNULL', 'JOIN', 'LATERAL', 'LEADING',
        'LEFT', 'LIKE', 'LIMIT', 'LOCALTIME', 'LOCALTIMESTAMP', 'NATURAL',
        'NOT', 'NOTNULL', 'NULL', 'OFFSET', 'ON', 'ONLY', 'OR', 'ORDER',
        'OUTER', 'OVERLAPS', 'PLACING', 'PRIMARY', 'REFERENCES', 'RETURNING',
        'RIGHT', 'SELECT', 'SESSION_USER', 'SIMILAR', 'SOME', 'SYMMETRIC',
        'TABLE', 'TABLESAMPLE', 'THEN', 'TO', 'TRAILING', 'TRUE', 'UNION',
        'UNIQUE', 'USER', 'USING', 'VARIADIC', 'VERBOSE', 'WHEN', 'WHERE',
        'WINDOW', 'WITH'
    ]) AS keyword
)
SELECT
    object_type || ' in ' ||
    schema_name,
    object_name || ' is a reserved keyword: ' ||
    keyword AS reserved_keyword_match
FROM (
    -- Tables using reserved keywords
    SELECT
        'table' AS object_type,
        table_schema AS schema_name,
        table_name AS object_name,
        keyword
    FROM information_schema.tables
    CROSS JOIN reserved_keywords
    WHERE
        table_schema NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb'
        )
        AND UPPER(table_name) = keyword

    UNION ALL

    -- Columns using reserved keywords
    SELECT
        'column' AS object_type,
        table_schema AS schema_name,
        table_name || '.' || column_name AS object_name,
        keyword
    FROM information_schema.columns
    CROSS JOIN reserved_keywords
    WHERE
        table_schema NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb'
        )
        AND UPPER(column_name) = keyword

    UNION ALL

    -- Indexes using reserved keywords
    SELECT
        'index' AS object_type,
        schemaname AS schema_name,
        indexname AS object_name,
        keyword
    FROM pg_indexes
    CROSS JOIN reserved_keywords
    WHERE
        schemaname NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb'
        )
        AND UPPER(indexname) = keyword
) AS reserved_objects
ORDER BY
    object_type,
    schema_name,
    object_name
$$
WHERE code = 'B010';

-- =============================================================================
-- B011 - Several tables in schema have different owners
-- =============================================================================
UPDATE pglinter.rules
SET
    q1 = $$
SELECT
    COUNT(*)::BIGINT AS total_schema_count
FROM
    pg_namespace n
WHERE
    n.nspname NOT IN ( 'pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb')
$$,
    q2 = $$
WITH C1 AS (
SELECT coalesce(count(DISTINCT tableowner)::BIGINT, 0) AS diff_owners
FROM pg_tables
WHERE
    schemaname NOT IN ('pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb')
GROUP BY schemaname)
SELECT COUNT(1) from C1 where diff_owners > 1
$$,
    q3 = $$
WITH SchemaOwnerTable AS (
    -- Step 1: Find all distinct combinations of (schemaname, tableowner)
    SELECT DISTINCT
        schemaname::TEXT AS schemaname,
        tableowner::TEXT AS tableowner
    FROM
        pg_tables
    WHERE
        schemaname NOT IN (
            'pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb'
        )
),
OwnerCounts AS (
    -- Step 2: Count the number of distinct owners for each schema
    SELECT
        schemaname,
        COUNT(tableowner) AS distinct_owner_count
    FROM
        SchemaOwnerTable
    GROUP BY
        schemaname
    HAVING
        -- Only keep schemas that have more than one distinct owner
        COUNT(tableowner) > 1
)
SELECT
    t.schemaname::TEXT,
    t.tablename || ' owner is ' || t.tableowner::TEXT AS table_and_owner
FROM
    pg_tables t
JOIN
    OwnerCounts oc ON t.schemaname = oc.schemaname
WHERE
    t.schemaname NOT IN (
        'pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb'
    )
ORDER BY
    1, 2
$$
WHERE code = 'B011';

-- =============================================================================
-- B012 - Composite primary keys with more than 4 columns
-- =============================================================================

UPDATE pglinter.rules
SET
    q1 = $$
SELECT COUNT(*)::BIGINT AS total_composite_pk_tables
FROM (
    SELECT tc.table_schema, tc.table_name, COUNT(kcu.column_name) AS pk_col_count
    FROM information_schema.table_constraints tc
    JOIN information_schema.key_column_usage kcu
      ON tc.constraint_name = kcu.constraint_name
     AND tc.table_schema = kcu.table_schema
     AND tc.table_name = kcu.table_name
    WHERE tc.constraint_type = 'PRIMARY KEY'
      AND tc.table_schema NOT IN ('pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb')
    GROUP BY tc.table_schema, tc.table_name, tc.constraint_name
) sub
$$,
    q2 = $$
SELECT COUNT(*)::BIGINT AS total_composite_pk_tables
FROM (
    SELECT tc.table_schema, tc.table_name, COUNT(kcu.column_name) AS pk_col_count
    FROM information_schema.table_constraints tc
    JOIN information_schema.key_column_usage kcu
      ON tc.constraint_name = kcu.constraint_name
     AND tc.table_schema = kcu.table_schema
     AND tc.table_name = kcu.table_name
    WHERE tc.constraint_type = 'PRIMARY KEY'
      AND tc.table_schema NOT IN ('pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb')
    GROUP BY tc.table_schema, tc.table_name, tc.constraint_name
    HAVING COUNT(kcu.column_name) > 4
) sub
$$,
    q3 = $$
SELECT
    sub.table_schema || '.' || sub.table_name ||'('||string_agg(sub.column_name, ', ')||')' AS pk_columns
FROM (
    SELECT
        tc.table_schema,
        tc.table_name,
        kcu.column_name
    FROM information_schema.table_constraints tc
    JOIN information_schema.key_column_usage kcu
      ON tc.constraint_name = kcu.constraint_name
     AND tc.table_schema = kcu.table_schema
     AND tc.table_name = kcu.table_name
    WHERE tc.constraint_type = 'PRIMARY KEY'
      AND tc.table_schema NOT IN ('pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb')
) sub
GROUP BY sub.table_schema, sub.table_name
HAVING COUNT(sub.column_name) > 4
$$
WHERE code = 'B012';

-- =============================================================================
-- S001 - Schema Permission Analysis
-- =============================================================================
UPDATE pglinter.rules
SET
    q1 = $$
SELECT
    COUNT(*) AS total_schema_count
FROM
    pg_namespace n
WHERE
    n.nspname NOT IN ('pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb')
    AND n.nspname NOT LIKE 'pg_%'
$$,
    q2 = $$
SELECT count(DISTINCT n.nspname::text)::BIGINT AS nb_schema
FROM pg_namespace n
WHERE
    n.nspname NOT IN ('pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb')
    AND n.nspname NOT LIKE 'pg_%'
    AND NOT EXISTS (
        SELECT 1
        FROM pg_default_acl da
        WHERE
            da.defaclnamespace = n.oid
            AND da.defaclrole != n.nspowner
    )
ORDER BY 1
$$,
    q3 = $$
SELECT DISTINCT n.nspname::text AS schema_name
FROM pg_namespace n
WHERE
    n.nspname NOT IN ('pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb')
    AND n.nspname NOT LIKE 'pg_%'
    AND NOT EXISTS (
        SELECT 1
        FROM pg_default_acl da
        WHERE
            da.defaclnamespace = n.oid
            AND da.defaclrole != n.nspowner
    )
ORDER BY 1
$$
WHERE code = 'S001';


-- =============================================================================
-- S002 - Environment-Named Schemas
-- =============================================================================
UPDATE pglinter.rules
SET
    q1 = $$
SELECT
    COUNT(*)::BIGINT AS total_schema_count
FROM
    pg_namespace n
WHERE
    n.nspname NOT IN ('pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb')
    AND n.nspname NOT LIKE 'pg_%'
$$,
    q2 = $$
SELECT count(n.nspname::text)::BIGINT AS nb_schema_name
FROM pg_namespace n
WHERE
    n.nspname NOT IN ('pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb')
    AND n.nspname NOT LIKE 'pg_%'
    AND (
        n.nspname ILIKE 'staging_%' OR n.nspname ILIKE '%_staging'
        OR n.nspname ILIKE 'stg_%' OR n.nspname ILIKE '%_stg'
        OR n.nspname ILIKE 'preprod_%' OR n.nspname ILIKE '%_preprod'
        OR n.nspname ILIKE 'prod_%' OR n.nspname ILIKE '%_prod'
        OR n.nspname ILIKE 'production_%' OR n.nspname ILIKE '%_production'
        OR n.nspname ILIKE 'dev_%' OR n.nspname ILIKE '%_dev'
        OR n.nspname ILIKE 'development_%' OR n.nspname ILIKE '%_development'
        OR n.nspname ILIKE 'sandbox_%' OR n.nspname ILIKE '%_sandbox'
        OR n.nspname ILIKE 'sbox_%' OR n.nspname ILIKE '%_sbox'
    )
$$,
    q3 = $$
SELECT n.nspname::text AS nb_schema_name
FROM pg_namespace n
WHERE
    n.nspname NOT IN ('pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb')
    AND n.nspname NOT LIKE 'pg_%'
    AND (
        n.nspname ILIKE 'staging_%' OR n.nspname ILIKE '%_staging'
        OR n.nspname ILIKE 'stg_%' OR n.nspname ILIKE '%_stg'
        OR n.nspname ILIKE 'preprod_%' OR n.nspname ILIKE '%_preprod'
        OR n.nspname ILIKE 'prod_%' OR n.nspname ILIKE '%_prod'
        OR n.nspname ILIKE 'production_%' OR n.nspname ILIKE '%_production'
        OR n.nspname ILIKE 'dev_%' OR n.nspname ILIKE '%_dev'
        OR n.nspname ILIKE 'development_%' OR n.nspname ILIKE '%_development'
        OR n.nspname ILIKE 'sandbox_%' OR n.nspname ILIKE '%_sandbox'
        OR n.nspname ILIKE 'sbox_%' OR n.nspname ILIKE '%_sbox'
    )
ORDER BY 1
$$
WHERE code = 'S002';

-- =============================================================================
-- S003 - Schema Public Access (Problems)
-- =============================================================================
UPDATE pglinter.rules
SET
    q1 = $$
SELECT
    COUNT(*)::BIGINT AS total_schema_count
FROM
    pg_namespace n
WHERE
    n.nspname NOT IN ( 'pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb')
    AND n.nspname NOT LIKE 'pg_%'
$$,
    q2 = $$
SELECT COUNT(*) AS total_schemas
FROM pg_namespace n
WHERE
    n.nspname NOT IN ('pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb')
    AND HAS_SCHEMA_PRIVILEGE('public', n.nspname, 'CREATE')
$$,
    q3 = $$
SELECT n.nspname::text AS schemas
FROM pg_namespace n
WHERE
    n.nspname NOT IN ('pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb')
    AND HAS_SCHEMA_PRIVILEGE('public', n.nspname, 'CREATE')
ORDER BY 1
$$
WHERE code = 'S003';

-- =============================================================================
-- S004 - Schema Owner is Internal Role
-- =============================================================================
UPDATE pglinter.rules
SET
    q1 = $$
SELECT
    COUNT(*)::BIGINT AS total_schema_count
FROM
    pg_namespace n
WHERE
    n.nspname NOT IN ( 'pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb')
    AND n.nspname NOT LIKE 'pg_%'
$$,
    q2 = $$
SELECT COUNT(*)::BIGINT AS total_schemas
FROM pg_namespace n
WHERE
    n.nspname NOT IN ('pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb')
    AND HAS_SCHEMA_PRIVILEGE('public', n.nspname, 'CREATE')
$$,
    q3 = $$
SELECT
    r.rolname::TEXT || ' is the owner of the schema ' || n.nspname::TEXT AS owner_info
FROM
    pg_namespace n
JOIN
    pg_roles r ON n.nspowner = r.oid
WHERE
    n.nspname NOT IN ('pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb')
    AND (
        r.rolsuper IS TRUE -- Owned by a Superuser (e.g., 'postgres')
        OR r.rolname LIKE 'pg_%' -- Owned by a reserved PostgreSQL system role
        OR r.rolname = 'postgres' -- Explicitly include the default administrative account
    )
ORDER BY
    1
$$
WHERE code = 'S004';

-- =============================================================================
-- S005 - Schema and table owners differ.
-- =============================================================================
UPDATE pglinter.rules
SET
    q1 = $$
SELECT
    COUNT(*)::BIGINT AS total_schema_count
FROM
    pg_namespace n
WHERE
    n.nspname NOT IN ('pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb')
    AND n.nspname NOT LIKE 'pg_%'
$$,
    q2 = $$
SELECT
    COUNT(DISTINCT n.nspname) AS schemas_with_mixed_ownership
FROM
    pg_namespace n
JOIN
    pg_class c ON c.relnamespace = n.oid -- Link schema to its relations (tables)
WHERE
    n.nspname NOT IN ( -- Exclude system/technical schemas
        'pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb'
    )
    AND n.nspname NOT LIKE 'pg_temp%' -- Exclude temp schemas
    AND c.relkind = 'r'               -- Only count regular tables ('r')
    AND n.nspowner <> c.relowner      -- Schema owner does NOT equal Table owner
$$,
    q3 = $$
SELECT
    'Owner of schema ' || n.nspname::TEXT || ' is ' || r_schema.rolname::TEXT ||' but owner of table '||n.nspname::TEXT ||'.'|| c.relname::TEXT || ' is ' || r_table.rolname::TEXT AS ownership_info
FROM
    pg_namespace n
JOIN
    pg_class c ON c.relnamespace = n.oid -- Link schema to its relations (tables)
JOIN
    pg_roles r_schema ON n.nspowner = r_schema.oid -- Get schema owner name
JOIN
    pg_roles r_table ON c.relowner = r_table.oid    -- Get table owner name
WHERE
    n.nspname NOT IN (
        'pg_toast', 'pg_catalog', 'information_schema', 'pglinter', '_timescaledb', 'timescaledb'
    )
    AND n.nspname NOT LIKE 'pg_temp%'
    AND c.relkind = 'r'               -- Only count regular tables
    AND n.nspowner <> c.relowner      -- The core condition: Owners are different
ORDER BY 1
$$
WHERE code = 'S005';


-- =============================================================================
-- C001 - Memory Configuration Analysis
-- =============================================================================
UPDATE pglinter.rules
SET q1 = $$
SELECT
    current_setting('max_connections')::int AS max_connections,
    current_setting('work_mem') AS work_mem_setting
$$
WHERE code = 'C001';

-- =============================================================================
-- C002 - Authentication Security (Total)
-- =============================================================================
UPDATE pglinter.rules
SET q1 = $$
SELECT count(*)::BIGINT FROM pg_catalog.pg_hba_file_rules
$$
WHERE code = 'C002';

-- =============================================================================
-- C002 - Authentication Security (Problems)
-- =============================================================================
UPDATE pglinter.rules
SET q2 = $$
SELECT count(*)::BIGINT
FROM pg_catalog.pg_hba_file_rules
WHERE auth_method IN ('trust', 'password')
$$
WHERE code = 'C002';

-- =============================================================================
-- C003 - MD5 encrypted Passwords (Problems)
-- =============================================================================
UPDATE pglinter.rules
SET q1 = $$
SELECT 'password_encryption is ' || setting FROM
pg_catalog.pg_settings
WHERE name='password_encryption' AND setting='md5'
$$
WHERE code = 'C003';
