//! Generated file, do not edit by hand, see `xtask/codegen`

use pgt_analyse::declare_lint_group;
pub mod adding_foreign_key_constraint;
pub mod adding_not_null_field;
pub mod adding_primary_key_constraint;
pub mod adding_required_field;
pub mod ban_drop_column;
pub mod ban_drop_database;
pub mod ban_drop_not_null;
pub mod ban_drop_table;
pub mod ban_truncate_cascade;
declare_lint_group! { pub Safety { name : "safety" , rules : [self :: adding_foreign_key_constraint :: AddingForeignKeyConstraint , self :: adding_not_null_field :: AddingNotNullField , self :: adding_primary_key_constraint :: AddingPrimaryKeyConstraint , self :: adding_required_field :: AddingRequiredField , self :: ban_drop_column :: BanDropColumn , self :: ban_drop_database :: BanDropDatabase , self :: ban_drop_not_null :: BanDropNotNull , self :: ban_drop_table :: BanDropTable , self :: ban_truncate_cascade :: BanTruncateCascade ,] } }
