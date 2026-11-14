//! Generated file, do not edit by hand, see `xtask/codegen`

use pgls_analyse::declare_lint_group;
pub mod ban_char_field;
pub mod creating_enum;
pub mod prefer_big_int;
pub mod prefer_identity;
pub mod prefer_jsonb;
pub mod prefer_text_field;
pub mod prefer_timestamptz;
pub mod require_column_qualifier;
pub mod require_enum_value_ordering;
pub mod require_function_schema;
pub mod require_operator_schema;
pub mod require_relation_schema;
pub mod require_table_schema;
pub mod require_type_schema;
declare_lint_group! { pub Style { name : "style" , rules : [self :: ban_char_field :: BanCharField , self :: creating_enum :: CreatingEnum , self :: prefer_big_int :: PreferBigInt , self :: prefer_identity :: PreferIdentity , self :: prefer_jsonb :: PreferJsonb , self :: prefer_text_field :: PreferTextField , self :: prefer_timestamptz :: PreferTimestamptz , self :: require_column_qualifier :: RequireColumnQualifier , self :: require_enum_value_ordering :: RequireEnumValueOrdering , self :: require_function_schema :: RequireFunctionSchema , self :: require_operator_schema :: RequireOperatorSchema , self :: require_relation_schema :: RequireRelationSchema , self :: require_table_schema :: RequireTableSchema , self :: require_type_schema :: RequireTypeSchema ,] } }
