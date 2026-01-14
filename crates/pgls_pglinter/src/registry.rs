//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use pgls_analyse::RegistryVisitor;
use pgls_diagnostics::Category;
#[doc = r" Metadata for a pglinter rule"]
#[derive(Debug, Clone, Copy)]
pub struct RuleMetadata {
    #[doc = r#" Rule code (e.g., "B001")"#]
    pub code: &'static str,
    #[doc = r" Rule name in camelCase"]
    pub name: &'static str,
    #[doc = r" Rule scope (BASE, SCHEMA, CLUSTER)"]
    pub scope: &'static str,
    #[doc = r" Description of what the rule detects"]
    pub description: &'static str,
    #[doc = r" Suggested fixes"]
    pub fixes: &'static [&'static str],
}
#[doc = r" Visit all pglinter rules using the visitor pattern"]
pub fn visit_registry<V: RegistryVisitor>(registry: &mut V) {
    registry.record_category::<crate::rules::PgLinter>();
}
#[doc = r" Get the pglinter rule code from the camelCase name"]
pub fn get_rule_code(name: &str) -> Option<&'static str> {
    match name {
        "compositePrimaryKeyTooManyColumns" => Some("B012"),
        "howManyObjectsWithUppercase" => Some("B005"),
        "howManyRedudantIndex" => Some("B002"),
        "howManyTableWithoutIndexOnFk" => Some("B003"),
        "howManyTableWithoutPrimaryKey" => Some("B001"),
        "howManyTablesNeverSelected" => Some("B006"),
        "howManyTablesWithFkMismatch" => Some("B008"),
        "howManyTablesWithFkOutsideSchema" => Some("B007"),
        "howManyTablesWithReservedKeywords" => Some("B010"),
        "howManyTablesWithSameTrigger" => Some("B009"),
        "howManyUnusedIndex" => Some("B004"),
        "ownerSchemaIsInternalRole" => Some("S004"),
        "passwordEncryptionIsMd5" => Some("C003"),
        "pgHbaEntriesWithMethodTrustOrPasswordShouldNotExists" => Some("C002"),
        "pgHbaEntriesWithMethodTrustShouldNotExists" => Some("C001"),
        "schemaOwnerDoNotMatchTableOwner" => Some("S005"),
        "schemaPrefixedOrSuffixedWithEnvt" => Some("S002"),
        "schemaWithDefaultRoleNotGranted" => Some("S001"),
        "severalTableOwnerInSchema" => Some("B011"),
        "unsecuredPublicSchema" => Some("S003"),
        _ => None,
    }
}
#[doc = r" Get the diagnostic category for a rule code"]
pub fn get_rule_category(code: &str) -> Option<&'static Category> {
    match code {
        "B012" => Some(::pgls_diagnostics::category!(
            "pglinter/base/compositePrimaryKeyTooManyColumns"
        )),
        "B005" => Some(::pgls_diagnostics::category!(
            "pglinter/base/howManyObjectsWithUppercase"
        )),
        "B002" => Some(::pgls_diagnostics::category!(
            "pglinter/base/howManyRedudantIndex"
        )),
        "B003" => Some(::pgls_diagnostics::category!(
            "pglinter/base/howManyTableWithoutIndexOnFk"
        )),
        "B001" => Some(::pgls_diagnostics::category!(
            "pglinter/base/howManyTableWithoutPrimaryKey"
        )),
        "B006" => Some(::pgls_diagnostics::category!(
            "pglinter/base/howManyTablesNeverSelected"
        )),
        "B008" => Some(::pgls_diagnostics::category!(
            "pglinter/base/howManyTablesWithFkMismatch"
        )),
        "B007" => Some(::pgls_diagnostics::category!(
            "pglinter/base/howManyTablesWithFkOutsideSchema"
        )),
        "B010" => Some(::pgls_diagnostics::category!(
            "pglinter/base/howManyTablesWithReservedKeywords"
        )),
        "B009" => Some(::pgls_diagnostics::category!(
            "pglinter/base/howManyTablesWithSameTrigger"
        )),
        "B004" => Some(::pgls_diagnostics::category!(
            "pglinter/base/howManyUnusedIndex"
        )),
        "S004" => Some(::pgls_diagnostics::category!(
            "pglinter/schema/ownerSchemaIsInternalRole"
        )),
        "C003" => Some(::pgls_diagnostics::category!(
            "pglinter/cluster/passwordEncryptionIsMd5"
        )),
        "C002" => Some(::pgls_diagnostics::category!(
            "pglinter/cluster/pgHbaEntriesWithMethodTrustOrPasswordShouldNotExists"
        )),
        "C001" => Some(::pgls_diagnostics::category!(
            "pglinter/cluster/pgHbaEntriesWithMethodTrustShouldNotExists"
        )),
        "S005" => Some(::pgls_diagnostics::category!(
            "pglinter/schema/schemaOwnerDoNotMatchTableOwner"
        )),
        "S002" => Some(::pgls_diagnostics::category!(
            "pglinter/schema/schemaPrefixedOrSuffixedWithEnvt"
        )),
        "S001" => Some(::pgls_diagnostics::category!(
            "pglinter/schema/schemaWithDefaultRoleNotGranted"
        )),
        "B011" => Some(::pgls_diagnostics::category!(
            "pglinter/base/severalTableOwnerInSchema"
        )),
        "S003" => Some(::pgls_diagnostics::category!(
            "pglinter/schema/unsecuredPublicSchema"
        )),
        _ => None,
    }
}
#[doc = r" Get rule metadata by name (camelCase)"]
pub fn get_rule_metadata(name: &str) -> Option<RuleMetadata> {
    match name {
        "compositePrimaryKeyTooManyColumns" => Some(RuleMetadata {
            code: "B012",
            name: "compositePrimaryKeyTooManyColumns",
            scope: "BASE",
            description: "Detect tables with composite primary keys involving more than 4 columns",
            fixes: &[
                "Consider redesigning the table to avoid composite primary keys with more than 4 columns",
                "Use surrogate keys (e.g., serial, UUID) instead of composite primary keys, and establish unique constraints on necessary column combinations, to enforce uniqueness.",
            ],
        }),
        "howManyObjectsWithUppercase" => Some(RuleMetadata {
            code: "B005",
            name: "howManyObjectsWithUppercase",
            scope: "BASE",
            description: "Count number of objects with uppercase in name or in columns.",
            fixes: &["Do not use uppercase for any database objects"],
        }),
        "howManyRedudantIndex" => Some(RuleMetadata {
            code: "B002",
            name: "howManyRedudantIndex",
            scope: "BASE",
            description: "Count number of redundant index vs nb index.",
            fixes: &[
                "remove duplicated index or check if a constraint does not create a redundant index, or change warning/error threshold",
            ],
        }),
        "howManyTableWithoutIndexOnFk" => Some(RuleMetadata {
            code: "B003",
            name: "howManyTableWithoutIndexOnFk",
            scope: "BASE",
            description: "Count number of tables without index on foreign key.",
            fixes: &["create a index on foreign key or change warning/error threshold"],
        }),
        "howManyTableWithoutPrimaryKey" => Some(RuleMetadata {
            code: "B001",
            name: "howManyTableWithoutPrimaryKey",
            scope: "BASE",
            description: "Count number of tables without primary key.",
            fixes: &["create a primary key or change warning/error threshold"],
        }),
        "howManyTablesNeverSelected" => Some(RuleMetadata {
            code: "B006",
            name: "howManyTablesNeverSelected",
            scope: "BASE",
            description: "Count number of table(s) that has never been selected.",
            fixes: &[
                "Is it necessary to update/delete/insert rows in table(s) that are never selected ?",
            ],
        }),
        "howManyTablesWithFkMismatch" => Some(RuleMetadata {
            code: "B008",
            name: "howManyTablesWithFkMismatch",
            scope: "BASE",
            description: "Count number of tables with foreign keys that do not match the key reference type.",
            fixes: &[
                "Consider column type adjustments to ensure foreign key matches referenced key type",
                "ask a dba",
            ],
        }),
        "howManyTablesWithFkOutsideSchema" => Some(RuleMetadata {
            code: "B007",
            name: "howManyTablesWithFkOutsideSchema",
            scope: "BASE",
            description: "Count number of tables with foreign keys outside their schema.",
            fixes: &[
                "Consider restructuring schema design to keep related tables in same schema",
                "ask a dba",
            ],
        }),
        "howManyTablesWithReservedKeywords" => Some(RuleMetadata {
            code: "B010",
            name: "howManyTablesWithReservedKeywords",
            scope: "BASE",
            description: "Count number of database objects using reserved keywords in their names.",
            fixes: &[
                "Rename database objects to avoid using reserved keywords.",
                "Using reserved keywords can lead to SQL syntax errors and maintenance difficulties.",
            ],
        }),
        "howManyTablesWithSameTrigger" => Some(RuleMetadata {
            code: "B009",
            name: "howManyTablesWithSameTrigger",
            scope: "BASE",
            description: "Count number of tables using the same trigger vs nb table with their own triggers.",
            fixes: &[
                "For more readability and other considerations use one trigger function per table.",
                "Sharing the same trigger function add more complexity.",
            ],
        }),
        "howManyUnusedIndex" => Some(RuleMetadata {
            code: "B004",
            name: "howManyUnusedIndex",
            scope: "BASE",
            description: "Count number of unused index vs nb index (base on pg_stat_user_indexes, indexes associated to unique constraints are discard.)",
            fixes: &["remove unused index or change warning/error threshold"],
        }),
        "ownerSchemaIsInternalRole" => Some(RuleMetadata {
            code: "S004",
            name: "ownerSchemaIsInternalRole",
            scope: "SCHEMA",
            description: "Owner of schema should not be any internal pg roles, or owner is a superuser (not sure it is necesary).",
            fixes: &["change schema owner to a functional role"],
        }),
        "passwordEncryptionIsMd5" => Some(RuleMetadata {
            code: "C003",
            name: "passwordEncryptionIsMd5",
            scope: "CLUSTER",
            description: "This configuration is not secure anymore and will prevent an upgrade to Postgres 18. Warning, you will need to reset all passwords after this is changed to scram-sha-256.",
            fixes: &[
                "change password_encryption parameter to scram-sha-256 (ALTER SYSTEM SET password_encryption = ",
                "scram-sha-256",
                " ). Warning, you will need to reset all passwords after this parameter is updated.",
            ],
        }),
        "pgHbaEntriesWithMethodTrustOrPasswordShouldNotExists" => Some(RuleMetadata {
            code: "C002",
            name: "pgHbaEntriesWithMethodTrustOrPasswordShouldNotExists",
            scope: "CLUSTER",
            description: "This configuration is extremely insecure and should only be used in a controlled, non-production environment for testing purposes. In a production environment, you should use more secure authentication methods such as md5, scram-sha-256, or cert, and restrict access to trusted IP addresses only.",
            fixes: &["change trust or password method in pg_hba.conf"],
        }),
        "pgHbaEntriesWithMethodTrustShouldNotExists" => Some(RuleMetadata {
            code: "C001",
            name: "pgHbaEntriesWithMethodTrustShouldNotExists",
            scope: "CLUSTER",
            description: "This configuration is extremely insecure and should only be used in a controlled, non-production environment for testing purposes. In a production environment, you should use more secure authentication methods such as md5, scram-sha-256, or cert, and restrict access to trusted IP addresses only.",
            fixes: &["change trust method in pg_hba.conf"],
        }),
        "schemaOwnerDoNotMatchTableOwner" => Some(RuleMetadata {
            code: "S005",
            name: "schemaOwnerDoNotMatchTableOwner",
            scope: "SCHEMA",
            description: "The schema owner and tables in the schema do not match.",
            fixes: &["For maintenance facilities, schema and tables owners should be the same."],
        }),
        "schemaPrefixedOrSuffixedWithEnvt" => Some(RuleMetadata {
            code: "S002",
            name: "schemaPrefixedOrSuffixedWithEnvt",
            scope: "SCHEMA",
            description: "The schema is prefixed with one of staging,stg,preprod,prod,sandbox,sbox string. Means that when you refresh your preprod, staging environments from production, you have to rename the target schema from prod_ to stg_ or something like. It is possible, but it is never easy.",
            fixes: &[
                "Keep the same schema name across environments. Prefer prefix or suffix the database name",
            ],
        }),
        "schemaWithDefaultRoleNotGranted" => Some(RuleMetadata {
            code: "S001",
            name: "schemaWithDefaultRoleNotGranted",
            scope: "SCHEMA",
            description: "The schema has no default role. Means that futur table will not be granted through a role. So you will have to re-execute grants on it.",
            fixes: &[
                "add a default privilege=> ALTER DEFAULT PRIVILEGES IN SCHEMA <schema> for user <schema",
                "s owner>",
            ],
        }),
        "severalTableOwnerInSchema" => Some(RuleMetadata {
            code: "B011",
            name: "severalTableOwnerInSchema",
            scope: "BASE",
            description: "In a schema there are several tables owned by different owners.",
            fixes: &["change table owners to the same functional role"],
        }),
        "unsecuredPublicSchema" => Some(RuleMetadata {
            code: "S003",
            name: "unsecuredPublicSchema",
            scope: "SCHEMA",
            description: "Only authorized users should be allowed to create objects.",
            fixes: &["REVOKE CREATE ON SCHEMA <schema_name> FROM PUBLIC"],
        }),
        _ => None,
    }
}
#[doc = r#" Get rule metadata by code (e.g., "B001", "S001", "C001")"#]
pub fn get_rule_metadata_by_code(code: &str) -> Option<RuleMetadata> {
    match code {
        "B012" => Some(RuleMetadata {
            code: "B012",
            name: "compositePrimaryKeyTooManyColumns",
            scope: "BASE",
            description: "Detect tables with composite primary keys involving more than 4 columns",
            fixes: &[
                "Consider redesigning the table to avoid composite primary keys with more than 4 columns",
                "Use surrogate keys (e.g., serial, UUID) instead of composite primary keys, and establish unique constraints on necessary column combinations, to enforce uniqueness.",
            ],
        }),
        "B005" => Some(RuleMetadata {
            code: "B005",
            name: "howManyObjectsWithUppercase",
            scope: "BASE",
            description: "Count number of objects with uppercase in name or in columns.",
            fixes: &["Do not use uppercase for any database objects"],
        }),
        "B002" => Some(RuleMetadata {
            code: "B002",
            name: "howManyRedudantIndex",
            scope: "BASE",
            description: "Count number of redundant index vs nb index.",
            fixes: &[
                "remove duplicated index or check if a constraint does not create a redundant index, or change warning/error threshold",
            ],
        }),
        "B003" => Some(RuleMetadata {
            code: "B003",
            name: "howManyTableWithoutIndexOnFk",
            scope: "BASE",
            description: "Count number of tables without index on foreign key.",
            fixes: &["create a index on foreign key or change warning/error threshold"],
        }),
        "B001" => Some(RuleMetadata {
            code: "B001",
            name: "howManyTableWithoutPrimaryKey",
            scope: "BASE",
            description: "Count number of tables without primary key.",
            fixes: &["create a primary key or change warning/error threshold"],
        }),
        "B006" => Some(RuleMetadata {
            code: "B006",
            name: "howManyTablesNeverSelected",
            scope: "BASE",
            description: "Count number of table(s) that has never been selected.",
            fixes: &[
                "Is it necessary to update/delete/insert rows in table(s) that are never selected ?",
            ],
        }),
        "B008" => Some(RuleMetadata {
            code: "B008",
            name: "howManyTablesWithFkMismatch",
            scope: "BASE",
            description: "Count number of tables with foreign keys that do not match the key reference type.",
            fixes: &[
                "Consider column type adjustments to ensure foreign key matches referenced key type",
                "ask a dba",
            ],
        }),
        "B007" => Some(RuleMetadata {
            code: "B007",
            name: "howManyTablesWithFkOutsideSchema",
            scope: "BASE",
            description: "Count number of tables with foreign keys outside their schema.",
            fixes: &[
                "Consider restructuring schema design to keep related tables in same schema",
                "ask a dba",
            ],
        }),
        "B010" => Some(RuleMetadata {
            code: "B010",
            name: "howManyTablesWithReservedKeywords",
            scope: "BASE",
            description: "Count number of database objects using reserved keywords in their names.",
            fixes: &[
                "Rename database objects to avoid using reserved keywords.",
                "Using reserved keywords can lead to SQL syntax errors and maintenance difficulties.",
            ],
        }),
        "B009" => Some(RuleMetadata {
            code: "B009",
            name: "howManyTablesWithSameTrigger",
            scope: "BASE",
            description: "Count number of tables using the same trigger vs nb table with their own triggers.",
            fixes: &[
                "For more readability and other considerations use one trigger function per table.",
                "Sharing the same trigger function add more complexity.",
            ],
        }),
        "B004" => Some(RuleMetadata {
            code: "B004",
            name: "howManyUnusedIndex",
            scope: "BASE",
            description: "Count number of unused index vs nb index (base on pg_stat_user_indexes, indexes associated to unique constraints are discard.)",
            fixes: &["remove unused index or change warning/error threshold"],
        }),
        "S004" => Some(RuleMetadata {
            code: "S004",
            name: "ownerSchemaIsInternalRole",
            scope: "SCHEMA",
            description: "Owner of schema should not be any internal pg roles, or owner is a superuser (not sure it is necesary).",
            fixes: &["change schema owner to a functional role"],
        }),
        "C003" => Some(RuleMetadata {
            code: "C003",
            name: "passwordEncryptionIsMd5",
            scope: "CLUSTER",
            description: "This configuration is not secure anymore and will prevent an upgrade to Postgres 18. Warning, you will need to reset all passwords after this is changed to scram-sha-256.",
            fixes: &[
                "change password_encryption parameter to scram-sha-256 (ALTER SYSTEM SET password_encryption = ",
                "scram-sha-256",
                " ). Warning, you will need to reset all passwords after this parameter is updated.",
            ],
        }),
        "C002" => Some(RuleMetadata {
            code: "C002",
            name: "pgHbaEntriesWithMethodTrustOrPasswordShouldNotExists",
            scope: "CLUSTER",
            description: "This configuration is extremely insecure and should only be used in a controlled, non-production environment for testing purposes. In a production environment, you should use more secure authentication methods such as md5, scram-sha-256, or cert, and restrict access to trusted IP addresses only.",
            fixes: &["change trust or password method in pg_hba.conf"],
        }),
        "C001" => Some(RuleMetadata {
            code: "C001",
            name: "pgHbaEntriesWithMethodTrustShouldNotExists",
            scope: "CLUSTER",
            description: "This configuration is extremely insecure and should only be used in a controlled, non-production environment for testing purposes. In a production environment, you should use more secure authentication methods such as md5, scram-sha-256, or cert, and restrict access to trusted IP addresses only.",
            fixes: &["change trust method in pg_hba.conf"],
        }),
        "S005" => Some(RuleMetadata {
            code: "S005",
            name: "schemaOwnerDoNotMatchTableOwner",
            scope: "SCHEMA",
            description: "The schema owner and tables in the schema do not match.",
            fixes: &["For maintenance facilities, schema and tables owners should be the same."],
        }),
        "S002" => Some(RuleMetadata {
            code: "S002",
            name: "schemaPrefixedOrSuffixedWithEnvt",
            scope: "SCHEMA",
            description: "The schema is prefixed with one of staging,stg,preprod,prod,sandbox,sbox string. Means that when you refresh your preprod, staging environments from production, you have to rename the target schema from prod_ to stg_ or something like. It is possible, but it is never easy.",
            fixes: &[
                "Keep the same schema name across environments. Prefer prefix or suffix the database name",
            ],
        }),
        "S001" => Some(RuleMetadata {
            code: "S001",
            name: "schemaWithDefaultRoleNotGranted",
            scope: "SCHEMA",
            description: "The schema has no default role. Means that futur table will not be granted through a role. So you will have to re-execute grants on it.",
            fixes: &[
                "add a default privilege=> ALTER DEFAULT PRIVILEGES IN SCHEMA <schema> for user <schema",
                "s owner>",
            ],
        }),
        "B011" => Some(RuleMetadata {
            code: "B011",
            name: "severalTableOwnerInSchema",
            scope: "BASE",
            description: "In a schema there are several tables owned by different owners.",
            fixes: &["change table owners to the same functional role"],
        }),
        "S003" => Some(RuleMetadata {
            code: "S003",
            name: "unsecuredPublicSchema",
            scope: "SCHEMA",
            description: "Only authorized users should be allowed to create objects.",
            fixes: &["REVOKE CREATE ON SCHEMA <schema_name> FROM PUBLIC"],
        }),
        _ => None,
    }
}
