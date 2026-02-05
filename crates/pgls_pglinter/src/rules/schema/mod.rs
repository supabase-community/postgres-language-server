//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
pub mod owner_schema_is_internal_role;
pub mod schema_owner_do_not_match_table_owner;
pub mod schema_prefixed_or_suffixed_with_envt;
pub mod schema_with_default_role_not_granted;
pub mod unsecured_public_schema;
::pgls_analyse::declare_lint_group! { pub Schema { name : "schema" , rules : [self :: owner_schema_is_internal_role :: OwnerSchemaIsInternalRole , self :: schema_owner_do_not_match_table_owner :: SchemaOwnerDoNotMatchTableOwner , self :: schema_prefixed_or_suffixed_with_envt :: SchemaPrefixedOrSuffixedWithEnvt , self :: schema_with_default_role_not_granted :: SchemaWithDefaultRoleNotGranted , self :: unsecured_public_schema :: UnsecuredPublicSchema ,] } }
