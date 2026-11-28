//! Generated file, do not edit by hand, see `xtask/codegen`

use pgls_analyse::declare_lint_group;
pub(crate) mod adding_required_field;
pub(crate) mod ban_alter_identity;
pub(crate) mod ban_alter_system_options;
pub(crate) mod ban_delete_without_where;
pub(crate) mod ban_drop_column;
pub(crate) mod ban_drop_constraint;
pub(crate) mod ban_drop_database;
pub(crate) mod ban_drop_default;
pub(crate) mod ban_drop_domain;
pub(crate) mod ban_drop_extension;
pub(crate) mod ban_drop_function;
pub(crate) mod ban_drop_generated_expression;
pub(crate) mod ban_drop_not_null;
pub(crate) mod ban_drop_schema;
pub(crate) mod ban_drop_sequence;
pub(crate) mod ban_drop_table;
pub(crate) mod ban_drop_type;
pub(crate) mod ban_drop_view;
pub(crate) mod ban_set_schema;
pub(crate) mod ban_truncate;
pub(crate) mod ban_truncate_cascade;
pub(crate) mod ban_update_without_where;
pub(crate) mod changing_column_type;
pub(crate) mod renaming_column;
pub(crate) mod renaming_object;
pub(crate) mod renaming_table;
declare_lint_group! { pub Destructive { name : "destructive" , rules : [self :: adding_required_field :: AddingRequiredField , self :: ban_alter_identity :: BanAlterIdentity , self :: ban_alter_system_options :: BanAlterSystemOptions , self :: ban_delete_without_where :: BanDeleteWithoutWhere , self :: ban_drop_column :: BanDropColumn , self :: ban_drop_constraint :: BanDropConstraint , self :: ban_drop_database :: BanDropDatabase , self :: ban_drop_default :: BanDropDefault , self :: ban_drop_domain :: BanDropDomain , self :: ban_drop_extension :: BanDropExtension , self :: ban_drop_function :: BanDropFunction , self :: ban_drop_generated_expression :: BanDropGeneratedExpression , self :: ban_drop_not_null :: BanDropNotNull , self :: ban_drop_schema :: BanDropSchema , self :: ban_drop_sequence :: BanDropSequence , self :: ban_drop_table :: BanDropTable , self :: ban_drop_type :: BanDropType , self :: ban_drop_view :: BanDropView , self :: ban_set_schema :: BanSetSchema , self :: ban_truncate :: BanTruncate , self :: ban_truncate_cascade :: BanTruncateCascade , self :: ban_update_without_where :: BanUpdateWithoutWhere , self :: changing_column_type :: ChangingColumnType , self :: renaming_column :: RenamingColumn , self :: renaming_object :: RenamingObject , self :: renaming_table :: RenamingTable ,] } }
