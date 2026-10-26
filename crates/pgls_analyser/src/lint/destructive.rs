//! Generated file, do not edit by hand, see `xtask/codegen`

use pgls_analyse::declare_lint_group;
pub mod adding_required_field;
pub mod ban_alter_identity;
pub mod ban_delete_without_where;
pub mod ban_drop_column;
pub mod ban_drop_constraint;
pub mod ban_drop_database;
pub mod ban_drop_default;
pub mod ban_drop_domain;
pub mod ban_drop_extension;
pub mod ban_drop_function;
pub mod ban_drop_generated_expression;
pub mod ban_drop_not_null;
pub mod ban_drop_schema;
pub mod ban_drop_sequence;
pub mod ban_drop_table;
pub mod ban_drop_type;
pub mod ban_drop_view;
pub mod ban_set_schema;
pub mod ban_truncate;
pub mod ban_truncate_cascade;
pub mod ban_update_without_where;
pub mod changing_column_type;
pub mod renaming_column;
pub mod renaming_object;
pub mod renaming_table;
declare_lint_group! { pub Destructive { name : "destructive" , rules : [self :: adding_required_field :: AddingRequiredField , self :: ban_alter_identity :: BanAlterIdentity , self :: ban_delete_without_where :: BanDeleteWithoutWhere , self :: ban_drop_column :: BanDropColumn , self :: ban_drop_constraint :: BanDropConstraint , self :: ban_drop_database :: BanDropDatabase , self :: ban_drop_default :: BanDropDefault , self :: ban_drop_domain :: BanDropDomain , self :: ban_drop_extension :: BanDropExtension , self :: ban_drop_function :: BanDropFunction , self :: ban_drop_generated_expression :: BanDropGeneratedExpression , self :: ban_drop_not_null :: BanDropNotNull , self :: ban_drop_schema :: BanDropSchema , self :: ban_drop_sequence :: BanDropSequence , self :: ban_drop_table :: BanDropTable , self :: ban_drop_type :: BanDropType , self :: ban_drop_view :: BanDropView , self :: ban_set_schema :: BanSetSchema , self :: ban_truncate :: BanTruncate , self :: ban_truncate_cascade :: BanTruncateCascade , self :: ban_update_without_where :: BanUpdateWithoutWhere , self :: changing_column_type :: ChangingColumnType , self :: renaming_column :: RenamingColumn , self :: renaming_object :: RenamingObject , self :: renaming_table :: RenamingTable ,] } }
