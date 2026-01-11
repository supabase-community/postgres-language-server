//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
pub mod composite_primary_key_too_many_columns;
pub mod how_many_objects_with_uppercase;
pub mod how_many_redudant_index;
pub mod how_many_table_without_index_on_fk;
pub mod how_many_table_without_primary_key;
pub mod how_many_tables_never_selected;
pub mod how_many_tables_with_fk_mismatch;
pub mod how_many_tables_with_fk_outside_schema;
pub mod how_many_tables_with_reserved_keywords;
pub mod how_many_tables_with_same_trigger;
pub mod how_many_unused_index;
pub mod several_table_owner_in_schema;
::pgls_analyse::declare_lint_group! { pub Base { name : "base" , rules : [self :: composite_primary_key_too_many_columns :: CompositePrimaryKeyTooManyColumns , self :: how_many_objects_with_uppercase :: HowManyObjectsWithUppercase , self :: how_many_redudant_index :: HowManyRedudantIndex , self :: how_many_table_without_index_on_fk :: HowManyTableWithoutIndexOnFk , self :: how_many_table_without_primary_key :: HowManyTableWithoutPrimaryKey , self :: how_many_tables_never_selected :: HowManyTablesNeverSelected , self :: how_many_tables_with_fk_mismatch :: HowManyTablesWithFkMismatch , self :: how_many_tables_with_fk_outside_schema :: HowManyTablesWithFkOutsideSchema , self :: how_many_tables_with_reserved_keywords :: HowManyTablesWithReservedKeywords , self :: how_many_tables_with_same_trigger :: HowManyTablesWithSameTrigger , self :: how_many_unused_index :: HowManyUnusedIndex , self :: several_table_owner_in_schema :: SeveralTableOwnerInSchema ,] } }
