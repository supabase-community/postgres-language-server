macro_rules! assert_node_variant {
    ($variant:ident, $expr:expr) => {
        match $expr.node.as_ref() {
            Some(pgt_query::NodeEnum::$variant(inner)) => inner,
            other => panic!("Expected {}, got {:?}", stringify!($variant), other),
        }
    };
}

mod a_array_expr;
mod a_const;
mod a_expr;
mod a_indices;
mod a_indirection;
mod a_star;
mod access_priv;
mod aggref;
mod alias;
mod alter_collation_stmt;
mod alter_database_refresh_coll_stmt;
mod alter_database_set_stmt;
mod alter_database_stmt;
mod alter_default_privileges_stmt;
mod alter_domain_stmt;
mod alter_enum_stmt;
mod alter_event_trig_stmt;
mod alter_extension_contents_stmt;
mod alter_extension_stmt;
mod alter_fdw_stmt;
mod alter_foreign_server_stmt;
mod alter_function_stmt;
mod alter_object_depends_stmt;
mod alter_object_schema_stmt;
mod alter_op_family_stmt;
mod alter_operator_stmt;
mod alter_owner_stmt;
mod alter_policy_stmt;
mod alter_publication_stmt;
mod alter_role_set_stmt;
mod alter_role_stmt;
mod alter_seq_stmt;
mod alter_stats_stmt;
mod alter_subscription_stmt;
mod alter_system_stmt;
mod alter_table_move_all_stmt;
mod alter_table_stmt;
mod alter_tablespace_options_stmt;
mod alter_ts_configuration_stmt;
mod alter_ts_dictionary_stmt;
mod alter_user_mapping_stmt;
mod array_coerce_expr;
mod bitstring;
mod bool_expr;
mod boolean;
mod boolean_test;
mod call_stmt;
mod case_expr;
mod case_when;
mod checkpoint_stmt;
mod close_portal_stmt;
mod cluster_stmt;
mod coalesce_expr;
mod coerce_to_domain;
mod coerce_to_domain_value;
mod coerce_via_io;
mod collate_clause;
mod column_def;
mod column_ref;
mod comment_stmt;
mod common_table_expr;
mod composite_type_stmt;
mod constraint;
mod constraints_set_stmt;
mod convert_rowtype_expr;
mod copy_stmt;
mod create_am_stmt;
mod create_cast_stmt;
mod create_conversion_stmt;
mod create_domain_stmt;
mod create_enum_stmt;
mod create_event_trig_stmt;
mod create_extension_stmt;
mod create_fdw_stmt;
mod create_foreign_server_stmt;
mod create_foreign_table_stmt;
mod create_function_stmt;
mod create_op_class_item;
mod create_op_class_stmt;
mod create_op_family_stmt;
mod create_plang_stmt;
mod create_policy_stmt;
mod create_publication_stmt;
mod create_range_stmt;
mod create_role_stmt;
mod create_schema_stmt;
mod create_seq_stmt;
mod create_stats_stmt;
mod create_stmt;
mod create_subscription_stmt;
mod create_table_as_stmt;
mod create_table_space_stmt;
mod create_transform_stmt;
mod create_trig_stmt;
mod create_user_mapping_stmt;
mod createdb_stmt;
mod current_of_expr;
mod deallocate_stmt;
mod declare_cursor_stmt;
mod def_elem;
mod define_stmt;
mod delete_stmt;
mod discard_stmt;
mod do_stmt;
mod drop_owned_stmt;
mod drop_role_stmt;
mod drop_stmt;
mod drop_subscription_stmt;
mod drop_table_space_stmt;
mod drop_user_mapping_stmt;
mod dropdb_stmt;
mod execute_stmt;
mod explain_stmt;
mod fetch_stmt;
mod field_select;
mod field_store;
mod float;
mod func_call;
mod func_expr;
mod grant_role_stmt;
mod grant_stmt;
mod grouping_func;
mod grouping_set;
mod import_foreign_schema_stmt;
mod index_elem;
mod index_stmt;
mod infer_clause;
mod insert_stmt;
mod integer;
mod join_expr;
mod json_agg_constructor;
mod json_array_constructor;
mod json_func_expr;
mod json_is_predicate;
mod json_key_value;
mod json_object_constructor;
mod json_parse_expr;
mod json_scalar_expr;
mod json_serialize_expr;
mod json_table;
mod json_value_expr;
mod list;
mod listen_stmt;
mod load_stmt;
mod lock_stmt;
mod locking_clause;
mod merge_stmt;
mod min_max_expr;
mod named_arg_expr;
mod node_list;
mod notify_stmt;
mod null_test;
mod object_with_args;
mod on_conflict_clause;
mod op_expr;
mod param_ref;
mod partition_bound_spec;
mod partition_elem;
mod partition_spec;
mod prepare_stmt;
mod publication_obj_spec;
mod range_function;
mod range_subselect;
mod range_table_func;
mod range_table_sample;
mod range_var;
mod reassign_owned_stmt;
mod refresh_matview_stmt;
mod reindex_stmt;
mod relabel_type;
mod rename_stmt;
mod replica_identity_stmt;
mod res_target;
mod role_spec;
mod row_compare_expr;
mod row_expr;
mod rule_stmt;
mod scalar_array_op_expr;
mod sec_label_stmt;
mod select_stmt;
mod set_operation_stmt;
mod set_to_default;
mod sort_by;
mod sql_value_function;
mod string;
mod sub_link;
mod sub_plan;
mod table_like_clause;
mod transaction_stmt;
mod truncate_stmt;
mod type_cast;
mod type_name;
mod unlisten_stmt;
mod update_stmt;
mod vacuum_relation;
mod vacuum_stmt;
mod variable_set_stmt;
mod variable_show_stmt;
mod view_stmt;
mod window_clause;
mod window_def;
mod window_func;
mod with_check_option;
mod with_clause;
mod xml_expr;
mod xml_serialize;

use a_array_expr::emit_a_array_expr;
use a_const::emit_a_const;
use a_expr::emit_a_expr;
use a_indices::emit_a_indices;
use a_indirection::emit_a_indirection;
use a_star::emit_a_star;
use access_priv::emit_access_priv;
use aggref::emit_aggref;
use alias::emit_alias;
use alter_collation_stmt::emit_alter_collation_stmt;
use alter_database_refresh_coll_stmt::emit_alter_database_refresh_coll_stmt;
use alter_database_set_stmt::emit_alter_database_set_stmt;
use alter_database_stmt::emit_alter_database_stmt;
use alter_default_privileges_stmt::emit_alter_default_privileges_stmt;
use alter_domain_stmt::emit_alter_domain_stmt;
use alter_enum_stmt::emit_alter_enum_stmt;
use alter_event_trig_stmt::emit_alter_event_trig_stmt;
use alter_extension_contents_stmt::emit_alter_extension_contents_stmt;
use alter_extension_stmt::emit_alter_extension_stmt;
use alter_fdw_stmt::emit_alter_fdw_stmt;
use alter_foreign_server_stmt::emit_alter_foreign_server_stmt;
use alter_function_stmt::emit_alter_function_stmt;
use alter_object_depends_stmt::emit_alter_object_depends_stmt;
use alter_object_schema_stmt::emit_alter_object_schema_stmt;
use alter_op_family_stmt::emit_alter_op_family_stmt;
use alter_operator_stmt::emit_alter_operator_stmt;
use alter_owner_stmt::emit_alter_owner_stmt;
use alter_policy_stmt::emit_alter_policy_stmt;
use alter_publication_stmt::emit_alter_publication_stmt;
use alter_role_set_stmt::emit_alter_role_set_stmt;
use alter_role_stmt::emit_alter_role_stmt;
use alter_seq_stmt::emit_alter_seq_stmt;
use alter_stats_stmt::emit_alter_stats_stmt;
use alter_subscription_stmt::emit_alter_subscription_stmt;
use alter_system_stmt::emit_alter_system_stmt;
use alter_table_move_all_stmt::emit_alter_table_move_all_stmt;
use alter_table_stmt::{emit_alter_table_cmd, emit_alter_table_stmt};
use alter_tablespace_options_stmt::emit_alter_tablespace_options_stmt;
use alter_ts_configuration_stmt::emit_alter_ts_configuration_stmt;
use alter_ts_dictionary_stmt::emit_alter_ts_dictionary_stmt;
use alter_user_mapping_stmt::emit_alter_user_mapping_stmt;
use array_coerce_expr::emit_array_coerce_expr;
use bitstring::emit_bitstring;
use bool_expr::emit_bool_expr;
use boolean::emit_boolean;
use boolean_test::emit_boolean_test;
use call_stmt::emit_call_stmt;
use case_expr::emit_case_expr;
use case_when::emit_case_when;
use checkpoint_stmt::emit_checkpoint_stmt;
use close_portal_stmt::emit_close_portal_stmt;
use cluster_stmt::emit_cluster_stmt;
use coalesce_expr::emit_coalesce_expr;
use coerce_to_domain::emit_coerce_to_domain;
use coerce_to_domain_value::emit_coerce_to_domain_value;
use coerce_via_io::emit_coerce_via_io;
use collate_clause::emit_collate_clause;
use column_def::emit_column_def;
use column_ref::emit_column_ref;
use comment_stmt::emit_comment_stmt;
use common_table_expr::emit_common_table_expr;
use composite_type_stmt::emit_composite_type_stmt;
use constraint::emit_constraint;
use constraints_set_stmt::emit_constraints_set_stmt;
use convert_rowtype_expr::emit_convert_rowtype_expr;
use copy_stmt::emit_copy_stmt;
use create_am_stmt::emit_create_am_stmt;
use create_cast_stmt::emit_create_cast_stmt;
use create_conversion_stmt::emit_create_conversion_stmt;
use create_domain_stmt::emit_create_domain_stmt;
use create_enum_stmt::emit_create_enum_stmt;
use create_event_trig_stmt::emit_create_event_trig_stmt;
use create_extension_stmt::emit_create_extension_stmt;
use create_fdw_stmt::emit_create_fdw_stmt;
use create_foreign_server_stmt::emit_create_foreign_server_stmt;
use create_foreign_table_stmt::emit_create_foreign_table_stmt;
use create_function_stmt::{emit_create_function_stmt, emit_function_parameter};
use create_op_class_item::emit_create_op_class_item;
use create_op_class_stmt::emit_create_op_class_stmt;
use create_op_family_stmt::emit_create_op_family_stmt;
use create_plang_stmt::emit_create_plang_stmt;
use create_policy_stmt::emit_create_policy_stmt;
use create_publication_stmt::emit_create_publication_stmt;
use create_range_stmt::emit_create_range_stmt;
use create_role_stmt::emit_create_role_stmt;
use create_schema_stmt::emit_create_schema_stmt;
use create_seq_stmt::emit_create_seq_stmt;
use create_stats_stmt::emit_create_stats_stmt;
use create_stmt::emit_create_stmt;
use create_subscription_stmt::emit_create_subscription_stmt;
use create_table_as_stmt::emit_create_table_as_stmt;
use create_table_space_stmt::emit_create_table_space_stmt;
use create_transform_stmt::emit_create_transform_stmt;
use create_trig_stmt::emit_create_trig_stmt;
use create_user_mapping_stmt::emit_create_user_mapping_stmt;
use createdb_stmt::emit_createdb_stmt;
use current_of_expr::emit_current_of_expr;
use deallocate_stmt::emit_deallocate_stmt;
use declare_cursor_stmt::emit_declare_cursor_stmt;
use def_elem::{emit_def_elem, emit_options_def_elem, emit_sequence_option};
use define_stmt::emit_define_stmt;
use delete_stmt::{emit_delete_stmt, emit_delete_stmt_no_semicolon};
use discard_stmt::emit_discard_stmt;
use do_stmt::emit_do_stmt;
use drop_owned_stmt::emit_drop_owned_stmt;
use drop_role_stmt::emit_drop_role_stmt;
use drop_stmt::emit_drop_stmt;
use drop_subscription_stmt::emit_drop_subscription_stmt;
use drop_table_space_stmt::emit_drop_table_space_stmt;
use drop_user_mapping_stmt::emit_drop_user_mapping_stmt;
use dropdb_stmt::emit_dropdb_stmt;
use execute_stmt::emit_execute_stmt;
use explain_stmt::emit_explain_stmt;
use fetch_stmt::emit_fetch_stmt;
use field_select::emit_field_select;
use field_store::emit_field_store;
use float::emit_float;
use func_call::emit_func_call;
use func_expr::emit_func_expr;
use grant_role_stmt::emit_grant_role_stmt;
use grant_stmt::emit_grant_stmt;
use grouping_func::emit_grouping_func;
use grouping_set::emit_grouping_set;
use import_foreign_schema_stmt::emit_import_foreign_schema_stmt;
use index_elem::emit_index_elem;
use index_stmt::emit_index_stmt;
use infer_clause::emit_infer_clause;
use insert_stmt::{emit_insert_stmt, emit_insert_stmt_no_semicolon};
use integer::emit_integer;
use join_expr::emit_join_expr;
use json_array_constructor::{
    emit_json_array_agg, emit_json_array_constructor, emit_json_array_query_constructor,
};
use json_func_expr::emit_json_func_expr;
use json_is_predicate::emit_json_is_predicate;
use json_key_value::emit_json_key_value;
use json_object_constructor::{emit_json_object_agg, emit_json_object_constructor};
use json_parse_expr::emit_json_parse_expr;
use json_scalar_expr::emit_json_scalar_expr;
use json_serialize_expr::emit_json_serialize_expr;
use json_table::emit_json_table;
use json_value_expr::emit_json_value_expr;
use list::emit_list;
use listen_stmt::emit_listen_stmt;
use load_stmt::emit_load_stmt;
use lock_stmt::emit_lock_stmt;
use locking_clause::emit_locking_clause;
use merge_stmt::emit_merge_stmt;
use min_max_expr::emit_min_max_expr;
use named_arg_expr::emit_named_arg_expr;
use notify_stmt::emit_notify_stmt;
use null_test::emit_null_test;
use object_with_args::emit_object_with_args;
use on_conflict_clause::emit_on_conflict_clause;
use op_expr::{emit_distinct_expr, emit_null_if_expr, emit_op_expr};
use param_ref::emit_param_ref;
use partition_bound_spec::emit_partition_bound_spec;
use partition_elem::emit_partition_elem;
use partition_spec::emit_partition_spec;
use prepare_stmt::emit_prepare_stmt;
use publication_obj_spec::emit_publication_obj_spec;
use range_function::emit_range_function;
use range_subselect::emit_range_subselect;
use range_table_func::emit_range_table_func;
use range_table_sample::emit_range_table_sample;
use range_var::emit_range_var;
use reassign_owned_stmt::emit_reassign_owned_stmt;
use refresh_matview_stmt::emit_refresh_matview_stmt;
use reindex_stmt::emit_reindex_stmt;
use relabel_type::emit_relabel_type;
use rename_stmt::emit_rename_stmt;
use replica_identity_stmt::emit_replica_identity_stmt;
use res_target::emit_res_target;
use role_spec::emit_role_spec;
use row_compare_expr::emit_row_compare_expr;
use row_expr::emit_row_expr;
use rule_stmt::emit_rule_stmt;
use scalar_array_op_expr::emit_scalar_array_op_expr;
use sec_label_stmt::emit_sec_label_stmt;
use select_stmt::{emit_select_stmt, emit_select_stmt_no_semicolon};
use set_operation_stmt::emit_set_operation_stmt;
use set_to_default::emit_set_to_default;
use sort_by::emit_sort_by;
use sql_value_function::emit_sql_value_function;
use string::{
    emit_identifier, emit_identifier_maybe_quoted, emit_string, emit_string_identifier,
    emit_string_literal,
};
use sub_link::emit_sub_link;
use sub_plan::{emit_alternative_sub_plan, emit_sub_plan};
use table_like_clause::emit_table_like_clause;
use transaction_stmt::emit_transaction_stmt;
use truncate_stmt::emit_truncate_stmt;
use type_cast::emit_type_cast;
use type_name::emit_type_name;
use unlisten_stmt::emit_unlisten_stmt;
use update_stmt::{emit_update_stmt, emit_update_stmt_no_semicolon};
use vacuum_relation::emit_vacuum_relation;
use vacuum_stmt::emit_vacuum_stmt;
use variable_set_stmt::emit_variable_set_stmt;
use variable_show_stmt::emit_variable_show_stmt;
use view_stmt::emit_view_stmt;
use window_clause::emit_window_clause;
use window_def::emit_window_def;
use window_func::emit_window_func;
use with_check_option::emit_with_check_option;
use with_clause::emit_with_clause;
use xml_expr::emit_xml_expr;
use xml_serialize::emit_xml_serialize;

use crate::emitter::{EventEmitter, GroupKind};
use pgt_query::{NodeEnum, protobuf::Node};

pub fn emit_node(node: &Node, e: &mut EventEmitter) {
    if let Some(ref inner) = node.node {
        emit_node_enum(inner, e)
    }
}

pub fn emit_node_enum(node: &NodeEnum, e: &mut EventEmitter) {
    match &node {
        NodeEnum::SelectStmt(n) => emit_select_stmt(e, n),
        NodeEnum::InsertStmt(n) => emit_insert_stmt(e, n),
        NodeEnum::UpdateStmt(n) => emit_update_stmt(e, n),
        NodeEnum::DeleteStmt(n) => emit_delete_stmt(e, n),
        NodeEnum::MergeStmt(n) => emit_merge_stmt(e, n),
        NodeEnum::DiscardStmt(n) => emit_discard_stmt(e, n),
        NodeEnum::DropStmt(n) => emit_drop_stmt(e, n),
        NodeEnum::DropRoleStmt(n) => emit_drop_role_stmt(e, n),
        NodeEnum::DropTableSpaceStmt(n) => emit_drop_table_space_stmt(e, n),
        NodeEnum::DropdbStmt(n) => emit_dropdb_stmt(e, n),
        NodeEnum::DropUserMappingStmt(n) => emit_drop_user_mapping_stmt(e, n),
        NodeEnum::DropSubscriptionStmt(n) => emit_drop_subscription_stmt(e, n),
        NodeEnum::DropOwnedStmt(n) => emit_drop_owned_stmt(e, n),
        NodeEnum::TruncateStmt(n) => emit_truncate_stmt(e, n),
        NodeEnum::CreateStmt(n) => emit_create_stmt(e, n),
        NodeEnum::CreateAmStmt(n) => emit_create_am_stmt(e, n),
        NodeEnum::CreateCastStmt(n) => emit_create_cast_stmt(e, n),
        NodeEnum::CreateConversionStmt(n) => emit_create_conversion_stmt(e, n),
        NodeEnum::CreateExtensionStmt(n) => emit_create_extension_stmt(e, n),
        NodeEnum::CreateFdwStmt(n) => emit_create_fdw_stmt(e, n),
        NodeEnum::CreateForeignServerStmt(n) => emit_create_foreign_server_stmt(e, n),
        NodeEnum::CreateForeignTableStmt(n) => emit_create_foreign_table_stmt(e, n),
        NodeEnum::CreateOpClassStmt(n) => emit_create_op_class_stmt(e, n),
        NodeEnum::CreateOpFamilyStmt(n) => emit_create_op_family_stmt(e, n),
        NodeEnum::CreateTableSpaceStmt(n) => emit_create_table_space_stmt(e, n),
        NodeEnum::ResTarget(n) => emit_res_target(e, n),
        NodeEnum::ColumnRef(n) => emit_column_ref(e, n),
        NodeEnum::ColumnDef(n) => emit_column_def(e, n),
        NodeEnum::Constraint(n) => emit_constraint(e, n),
        NodeEnum::ConvertRowtypeExpr(n) => emit_convert_rowtype_expr(e, n),
        NodeEnum::DefElem(n) => emit_def_elem(e, n),
        NodeEnum::String(n) => emit_string(e, n),
        NodeEnum::RangeVar(n) => emit_range_var(e, n),
        NodeEnum::AConst(n) => emit_a_const(e, n),
        NodeEnum::Integer(n) => emit_integer(e, n),
        NodeEnum::Float(n) => emit_float(e, n),
        NodeEnum::Boolean(n) => emit_boolean(e, n),
        NodeEnum::BitString(n) => emit_bitstring(e, n),
        NodeEnum::AArrayExpr(n) => emit_a_array_expr(e, n),
        NodeEnum::AIndices(n) => emit_a_indices(e, n),
        NodeEnum::AIndirection(n) => emit_a_indirection(e, n),
        NodeEnum::AExpr(n) => emit_a_expr(e, n),
        NodeEnum::Aggref(n) => emit_aggref(e, n),
        NodeEnum::OpExpr(n) => emit_op_expr(e, n),
        NodeEnum::DistinctExpr(n) => emit_distinct_expr(e, n),
        NodeEnum::NullIfExpr(n) => emit_null_if_expr(e, n),
        NodeEnum::ArrayCoerceExpr(n) => emit_array_coerce_expr(e, n),
        NodeEnum::AStar(n) => emit_a_star(e, n),
        NodeEnum::BoolExpr(n) => emit_bool_expr(e, n),
        NodeEnum::BooleanTest(n) => emit_boolean_test(e, n),
        NodeEnum::CaseExpr(n) => emit_case_expr(e, n),
        NodeEnum::CaseWhen(n) => emit_case_when(e, n),
        NodeEnum::CoalesceExpr(n) => emit_coalesce_expr(e, n),
        NodeEnum::CoerceToDomain(n) => emit_coerce_to_domain(e, n),
        NodeEnum::CoerceToDomainValue(n) => emit_coerce_to_domain_value(e, n),
        NodeEnum::CoerceViaIo(n) => emit_coerce_via_io(e, n),
        NodeEnum::CollateClause(n) => emit_collate_clause(e, n),
        NodeEnum::CurrentOfExpr(n) => emit_current_of_expr(e, n),
        NodeEnum::FuncExpr(n) => emit_func_expr(e, n),
        NodeEnum::FuncCall(n) => emit_func_call(e, n),
        NodeEnum::FunctionParameter(n) => emit_function_parameter(e, n),
        NodeEnum::FieldSelect(n) => emit_field_select(e, n),
        NodeEnum::FieldStore(n) => emit_field_store(e, n),
        NodeEnum::GroupingFunc(n) => emit_grouping_func(e, n),
        NodeEnum::GroupingSet(n) => emit_grouping_set(e, n),
        NodeEnum::NamedArgExpr(n) => emit_named_arg_expr(e, n),
        NodeEnum::MinMaxExpr(n) => emit_min_max_expr(e, n),
        NodeEnum::NullTest(n) => emit_null_test(e, n),
        NodeEnum::ParamRef(n) => emit_param_ref(e, n),
        NodeEnum::PartitionElem(n) => emit_partition_elem(e, n),
        NodeEnum::PartitionSpec(n) => emit_partition_spec(e, n),
        NodeEnum::RowCompareExpr(n) => emit_row_compare_expr(e, n),
        NodeEnum::RowExpr(n) => emit_row_expr(e, n),
        NodeEnum::ScalarArrayOpExpr(n) => emit_scalar_array_op_expr(e, n),
        NodeEnum::SetToDefault(n) => emit_set_to_default(e, n),
        NodeEnum::SqlvalueFunction(n) => emit_sql_value_function(e, n),
        NodeEnum::TypeCast(n) => emit_type_cast(e, n),
        NodeEnum::TypeName(n) => emit_type_name(e, n),
        NodeEnum::JoinExpr(n) => emit_join_expr(e, n),
        NodeEnum::Alias(n) => emit_alias(e, n),
        NodeEnum::RangeSubselect(n) => emit_range_subselect(e, n),
        NodeEnum::RangeFunction(n) => emit_range_function(e, n),
        NodeEnum::SortBy(n) => emit_sort_by(e, n),
        NodeEnum::SubLink(n) => emit_sub_link(e, n),
        NodeEnum::SubPlan(n) => emit_sub_plan(e, n),
        NodeEnum::AlternativeSubPlan(n) => emit_alternative_sub_plan(e, n),
        NodeEnum::List(n) => emit_list(e, n),
        NodeEnum::VariableSetStmt(n) => emit_variable_set_stmt(e, n),
        NodeEnum::VariableShowStmt(n) => emit_variable_show_stmt(e, n),
        NodeEnum::TransactionStmt(n) => emit_transaction_stmt(e, n),
        NodeEnum::VacuumStmt(n) => emit_vacuum_stmt(e, n),
        NodeEnum::ViewStmt(n) => emit_view_stmt(e, n),
        NodeEnum::CreateSchemaStmt(n) => emit_create_schema_stmt(e, n),
        NodeEnum::CreateRoleStmt(n) => emit_create_role_stmt(e, n),
        NodeEnum::CreateSeqStmt(n) => emit_create_seq_stmt(e, n),
        NodeEnum::CreatedbStmt(n) => emit_createdb_stmt(e, n),
        NodeEnum::CreateDomainStmt(n) => emit_create_domain_stmt(e, n),
        NodeEnum::CreateEnumStmt(n) => emit_create_enum_stmt(e, n),
        NodeEnum::CreateEventTrigStmt(n) => emit_create_event_trig_stmt(e, n),
        NodeEnum::CreateFunctionStmt(n) => emit_create_function_stmt(e, n),
        NodeEnum::CreatePlangStmt(n) => emit_create_plang_stmt(e, n),
        NodeEnum::CreatePolicyStmt(n) => emit_create_policy_stmt(e, n),
        NodeEnum::CreatePublicationStmt(n) => emit_create_publication_stmt(e, n),
        NodeEnum::CreateRangeStmt(n) => emit_create_range_stmt(e, n),
        NodeEnum::CreateStatsStmt(n) => emit_create_stats_stmt(e, n),
        NodeEnum::CreateSubscriptionStmt(n) => emit_create_subscription_stmt(e, n),
        NodeEnum::CreateTransformStmt(n) => emit_create_transform_stmt(e, n),
        NodeEnum::CreateTrigStmt(n) => emit_create_trig_stmt(e, n),
        NodeEnum::CreateUserMappingStmt(n) => emit_create_user_mapping_stmt(e, n),
        NodeEnum::IndexStmt(n) => emit_index_stmt(e, n),
        NodeEnum::IndexElem(n) => emit_index_elem(e, n),
        NodeEnum::DoStmt(n) => emit_do_stmt(e, n),
        NodeEnum::PrepareStmt(n) => emit_prepare_stmt(e, n),
        NodeEnum::CallStmt(n) => emit_call_stmt(e, n),
        NodeEnum::CheckPointStmt(n) => emit_checkpoint_stmt(e, n),
        NodeEnum::ClosePortalStmt(n) => emit_close_portal_stmt(e, n),
        NodeEnum::ClusterStmt(n) => emit_cluster_stmt(e, n),
        NodeEnum::CommentStmt(n) => emit_comment_stmt(e, n),
        NodeEnum::ConstraintsSetStmt(n) => emit_constraints_set_stmt(e, n),
        NodeEnum::CopyStmt(n) => emit_copy_stmt(e, n),
        NodeEnum::LoadStmt(n) => emit_load_stmt(e, n),
        NodeEnum::NotifyStmt(n) => emit_notify_stmt(e, n),
        NodeEnum::DeclareCursorStmt(n) => emit_declare_cursor_stmt(e, n),
        NodeEnum::ObjectWithArgs(n) => emit_object_with_args(e, n),
        NodeEnum::DefineStmt(n) => emit_define_stmt(e, n),
        NodeEnum::GrantStmt(n) => emit_grant_stmt(e, n),
        NodeEnum::GrantRoleStmt(n) => emit_grant_role_stmt(e, n),
        NodeEnum::RoleSpec(n) => emit_role_spec(e, n),
        NodeEnum::AlterCollationStmt(n) => emit_alter_collation_stmt(e, n),
        NodeEnum::AlterDatabaseStmt(n) => emit_alter_database_stmt(e, n),
        NodeEnum::AlterDatabaseSetStmt(n) => emit_alter_database_set_stmt(e, n),
        NodeEnum::AlterDatabaseRefreshCollStmt(n) => emit_alter_database_refresh_coll_stmt(e, n),
        NodeEnum::AlterDefaultPrivilegesStmt(n) => emit_alter_default_privileges_stmt(e, n),
        NodeEnum::AlterDomainStmt(n) => emit_alter_domain_stmt(e, n),
        NodeEnum::AlterEnumStmt(n) => emit_alter_enum_stmt(e, n),
        NodeEnum::AlterEventTrigStmt(n) => emit_alter_event_trig_stmt(e, n),
        NodeEnum::AlterExtensionStmt(n) => emit_alter_extension_stmt(e, n),
        NodeEnum::AlterExtensionContentsStmt(n) => emit_alter_extension_contents_stmt(e, n),
        NodeEnum::AlterFdwStmt(n) => emit_alter_fdw_stmt(e, n),
        NodeEnum::AlterForeignServerStmt(n) => emit_alter_foreign_server_stmt(e, n),
        NodeEnum::AlterFunctionStmt(n) => emit_alter_function_stmt(e, n),
        NodeEnum::AlterObjectDependsStmt(n) => emit_alter_object_depends_stmt(e, n),
        NodeEnum::AlterObjectSchemaStmt(n) => emit_alter_object_schema_stmt(e, n),
        NodeEnum::AlterOperatorStmt(n) => emit_alter_operator_stmt(e, n),
        NodeEnum::AlterOpFamilyStmt(n) => emit_alter_op_family_stmt(e, n),
        NodeEnum::AlterOwnerStmt(n) => emit_alter_owner_stmt(e, n),
        NodeEnum::AlterPolicyStmt(n) => emit_alter_policy_stmt(e, n),
        NodeEnum::AlterPublicationStmt(n) => emit_alter_publication_stmt(e, n),
        NodeEnum::AlterRoleStmt(n) => emit_alter_role_stmt(e, n),
        NodeEnum::AlterRoleSetStmt(n) => emit_alter_role_set_stmt(e, n),
        NodeEnum::AlterSeqStmt(n) => emit_alter_seq_stmt(e, n),
        NodeEnum::AlterStatsStmt(n) => emit_alter_stats_stmt(e, n),
        NodeEnum::AlterSubscriptionStmt(n) => emit_alter_subscription_stmt(e, n),
        NodeEnum::AlterSystemStmt(n) => emit_alter_system_stmt(e, n),
        NodeEnum::AlterTableStmt(n) => emit_alter_table_stmt(e, n),
        NodeEnum::AlterTableCmd(n) => emit_alter_table_cmd(e, n),
        NodeEnum::AlterTableMoveAllStmt(n) => emit_alter_table_move_all_stmt(e, n),
        NodeEnum::AlterTableSpaceOptionsStmt(n) => emit_alter_tablespace_options_stmt(e, n),
        NodeEnum::AlterTsconfigurationStmt(n) => emit_alter_ts_configuration_stmt(e, n),
        NodeEnum::AlterTsdictionaryStmt(n) => emit_alter_ts_dictionary_stmt(e, n),
        NodeEnum::AlterUserMappingStmt(n) => emit_alter_user_mapping_stmt(e, n),
        NodeEnum::ExplainStmt(n) => emit_explain_stmt(e, n),
        NodeEnum::ImportForeignSchemaStmt(n) => emit_import_foreign_schema_stmt(e, n),
        NodeEnum::InferClause(n) => emit_infer_clause(e, n),
        NodeEnum::ExecuteStmt(n) => emit_execute_stmt(e, n),
        NodeEnum::FetchStmt(n) => emit_fetch_stmt(e, n),
        NodeEnum::ListenStmt(n) => emit_listen_stmt(e, n),
        NodeEnum::UnlistenStmt(n) => emit_unlisten_stmt(e, n),
        NodeEnum::LockStmt(n) => emit_lock_stmt(e, n),
        NodeEnum::LockingClause(n) => emit_locking_clause(e, n),
        NodeEnum::RelabelType(n) => emit_relabel_type(e, n),
        NodeEnum::ReindexStmt(n) => emit_reindex_stmt(e, n),
        NodeEnum::RenameStmt(n) => emit_rename_stmt(e, n),
        NodeEnum::ReplicaIdentityStmt(n) => emit_replica_identity_stmt(e, n),
        NodeEnum::DeallocateStmt(n) => emit_deallocate_stmt(e, n),
        NodeEnum::RefreshMatViewStmt(n) => emit_refresh_matview_stmt(e, n),
        NodeEnum::ReassignOwnedStmt(n) => emit_reassign_owned_stmt(e, n),
        NodeEnum::RuleStmt(n) => emit_rule_stmt(e, n),
        NodeEnum::CompositeTypeStmt(n) => emit_composite_type_stmt(e, n),
        NodeEnum::CreateTableAsStmt(n) => emit_create_table_as_stmt(e, n),
        NodeEnum::TableLikeClause(n) => emit_table_like_clause(e, n),
        NodeEnum::VacuumRelation(n) => emit_vacuum_relation(e, n),
        NodeEnum::JsonFuncExpr(n) => emit_json_func_expr(e, n),
        NodeEnum::JsonIsPredicate(n) => emit_json_is_predicate(e, n),
        NodeEnum::JsonParseExpr(n) => emit_json_parse_expr(e, n),
        NodeEnum::JsonSerializeExpr(n) => emit_json_serialize_expr(e, n),
        NodeEnum::JsonScalarExpr(n) => emit_json_scalar_expr(e, n),
        NodeEnum::JsonTable(n) => emit_json_table(e, n),
        NodeEnum::JsonValueExpr(n) => emit_json_value_expr(e, n),
        NodeEnum::JsonKeyValue(n) => emit_json_key_value(e, n),
        NodeEnum::JsonObjectConstructor(n) => emit_json_object_constructor(e, n),
        NodeEnum::JsonArrayConstructor(n) => emit_json_array_constructor(e, n),
        NodeEnum::JsonArrayQueryConstructor(n) => emit_json_array_query_constructor(e, n),
        NodeEnum::JsonObjectAgg(n) => emit_json_object_agg(e, n),
        NodeEnum::JsonArrayAgg(n) => emit_json_array_agg(e, n),
        NodeEnum::RangeTableFunc(n) => emit_range_table_func(e, n),
        NodeEnum::RangeTableSample(n) => emit_range_table_sample(e, n),
        NodeEnum::XmlExpr(n) => emit_xml_expr(e, n),
        NodeEnum::XmlSerialize(n) => emit_xml_serialize(e, n),
        NodeEnum::AccessPriv(n) => emit_access_priv(e, n),
        NodeEnum::CreateOpClassItem(n) => emit_create_op_class_item(e, n),
        NodeEnum::PublicationObjSpec(n) => emit_publication_obj_spec(e, n),
        NodeEnum::SecLabelStmt(n) => emit_sec_label_stmt(e, n),
        NodeEnum::SetOperationStmt(n) => emit_set_operation_stmt(e, n),
        NodeEnum::WindowClause(n) => emit_window_clause(e, n),
        NodeEnum::WindowFunc(n) => emit_window_func(e, n),
        NodeEnum::WindowDef(n) => {
            e.group_start(GroupKind::WindowDef);
            emit_window_def(e, n);
            e.group_end();
        }
        NodeEnum::WithClause(n) => emit_with_clause(e, n),
        NodeEnum::WithCheckOption(n) => emit_with_check_option(e, n),
        NodeEnum::CommonTableExpr(n) => emit_common_table_expr(e, n),
        _ => todo!("emit_node_enum: unhandled node type {:?}", node),
    }
}
