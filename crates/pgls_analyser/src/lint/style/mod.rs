//! Generated file, do not edit by hand, see `xtask/codegen`

use pgls_analyse::declare_lint_group;
pub(crate) mod ban_char_field;
pub(crate) mod creating_enum;
pub(crate) mod prefer_big_int;
pub(crate) mod prefer_identity;
pub(crate) mod prefer_jsonb;
pub(crate) mod prefer_text_field;
pub(crate) mod prefer_timestamptz;
pub(crate) mod require_column_qualifier;
pub(crate) mod require_enum_value_ordering;
pub(crate) mod require_function_schema;
pub(crate) mod require_operator_schema;
pub(crate) mod require_relation_schema;
pub(crate) mod require_table_schema;
pub(crate) mod require_type_schema;
declare_lint_group! { pub Style { name : "style" , rules : [self :: ban_char_field :: BanCharField , self :: creating_enum :: CreatingEnum , self :: prefer_big_int :: PreferBigInt , self :: prefer_identity :: PreferIdentity , self :: prefer_jsonb :: PreferJsonb , self :: prefer_text_field :: PreferTextField , self :: prefer_timestamptz :: PreferTimestamptz , self :: require_column_qualifier :: RequireColumnQualifier , self :: require_enum_value_ordering :: RequireEnumValueOrdering , self :: require_function_schema :: RequireFunctionSchema , self :: require_operator_schema :: RequireOperatorSchema , self :: require_relation_schema :: RequireRelationSchema , self :: require_table_schema :: RequireTableSchema , self :: require_type_schema :: RequireTypeSchema ,] } }
