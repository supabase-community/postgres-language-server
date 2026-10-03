//! Applying the effects of statements to the catalog, one module per statement, like the nodes
//! of `pgls_pretty_print`.
//!
//! Only DDL that creates, alters, renames, or drops relations, columns, types, functions, or
//! schemas is modelled. Statements that may change these in ways we can't follow taint the
//! catalog. Everything else (DML, `SET`, indexes, grants, comments, policies, triggers,
//! constraints, ...) has no effect on name resolution and is ignored.
//!
//! Known gaps, all of which can only hide errors, never report wrong ones:
//! - `CASCADE` does not drop dependent objects.
//! - `DROP EXTENSION` does not drop the extension's objects.

mod alter_object_schema_stmt;
mod alter_table_stmt;
mod composite_type_stmt;
mod create_domain_stmt;
mod create_enum_stmt;
mod create_extension_stmt;
mod create_foreign_table_stmt;
mod create_function_stmt;
mod create_range_stmt;
mod create_schema_stmt;
mod create_seq_stmt;
mod create_stmt;
mod create_table_as_stmt;
mod define_stmt;
mod drop_stmt;
mod into_clause;
mod rename_stmt;
mod select_stmt;
mod transaction_stmt;
mod view_stmt;

use pgls_query::NodeEnum;

use super::Catalog;

use alter_object_schema_stmt::apply_alter_object_schema_stmt;
use alter_table_stmt::apply_alter_table_stmt;
use composite_type_stmt::apply_composite_type_stmt;
use create_domain_stmt::apply_create_domain_stmt;
use create_enum_stmt::apply_create_enum_stmt;
use create_extension_stmt::apply_create_extension_stmt;
use create_foreign_table_stmt::apply_create_foreign_table_stmt;
use create_function_stmt::apply_create_function_stmt;
use create_range_stmt::apply_create_range_stmt;
use create_schema_stmt::apply_create_schema_stmt;
use create_seq_stmt::apply_create_seq_stmt;
use create_stmt::apply_create_stmt;
use create_table_as_stmt::apply_create_table_as_stmt;
use define_stmt::apply_define_stmt;
use drop_stmt::apply_drop_stmt;
use rename_stmt::apply_rename_stmt;
use select_stmt::apply_select_stmt;
use transaction_stmt::apply_transaction_stmt;
use view_stmt::apply_view_stmt;

impl Catalog {
    /// Applies the effects of a top-level statement, executed with the given search path.
    pub fn apply(&mut self, stmt: &NodeEnum, search_path: &[String]) {
        let c = self;
        match stmt {
            NodeEnum::CreateSchemaStmt(n) => apply_create_schema_stmt(c, n, search_path),
            NodeEnum::CreateStmt(n) => apply_create_stmt(c, n, search_path),
            NodeEnum::CreateForeignTableStmt(n) => {
                apply_create_foreign_table_stmt(c, n, search_path)
            }
            NodeEnum::CreateTableAsStmt(n) => apply_create_table_as_stmt(c, n, search_path),
            NodeEnum::SelectStmt(n) => apply_select_stmt(c, n, search_path),
            NodeEnum::ViewStmt(n) => apply_view_stmt(c, n, search_path),
            NodeEnum::CreateSeqStmt(n) => apply_create_seq_stmt(c, n, search_path),
            NodeEnum::CompositeTypeStmt(n) => apply_composite_type_stmt(c, n, search_path),
            NodeEnum::CreateEnumStmt(n) => apply_create_enum_stmt(c, n, search_path),
            NodeEnum::CreateDomainStmt(n) => apply_create_domain_stmt(c, n, search_path),
            NodeEnum::CreateRangeStmt(n) => apply_create_range_stmt(c, n, search_path),
            NodeEnum::DefineStmt(n) => apply_define_stmt(c, n, search_path),
            NodeEnum::CreateFunctionStmt(n) => apply_create_function_stmt(c, n, search_path),
            NodeEnum::AlterTableStmt(n) => apply_alter_table_stmt(c, n, search_path),
            NodeEnum::RenameStmt(n) => apply_rename_stmt(c, n, search_path),
            NodeEnum::AlterObjectSchemaStmt(n) => apply_alter_object_schema_stmt(c, n, search_path),
            NodeEnum::DropStmt(n) => apply_drop_stmt(c, n, search_path),
            NodeEnum::CreateCastStmt(_) => c.casts_incomplete = true,
            NodeEnum::AlterOperatorStmt(_) => c.operators_incomplete = true,
            NodeEnum::TransactionStmt(n) => apply_transaction_stmt(c, n),
            NodeEnum::CreateExtensionStmt(n) => apply_create_extension_stmt(c, n),
            // These run arbitrary code or create objects we can't see.
            NodeEnum::DoStmt(_)
            | NodeEnum::CallStmt(_)
            | NodeEnum::ExecuteStmt(_)
            | NodeEnum::ImportForeignSchemaStmt(_)
            | NodeEnum::AlterExtensionStmt(_)
            | NodeEnum::AlterExtensionContentsStmt(_) => c.tainted = true,
            _ => {}
        }
    }
}
