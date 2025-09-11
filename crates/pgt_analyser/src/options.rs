//! Generated file, do not edit by hand, see `xtask/codegen`

use crate::lint;
pub type AddingForeignKeyConstraint = < lint :: safety :: adding_foreign_key_constraint :: AddingForeignKeyConstraint as pgt_analyse :: Rule > :: Options ;
pub type AddingNotNullField =
    <lint::safety::adding_not_null_field::AddingNotNullField as pgt_analyse::Rule>::Options;
pub type AddingPrimaryKeyConstraint = < lint :: safety :: adding_primary_key_constraint :: AddingPrimaryKeyConstraint as pgt_analyse :: Rule > :: Options ;
pub type AddingRequiredField =
    <lint::safety::adding_required_field::AddingRequiredField as pgt_analyse::Rule>::Options;
pub type BanDropColumn =
    <lint::safety::ban_drop_column::BanDropColumn as pgt_analyse::Rule>::Options;
pub type BanDropDatabase =
    <lint::safety::ban_drop_database::BanDropDatabase as pgt_analyse::Rule>::Options;
pub type BanDropNotNull =
    <lint::safety::ban_drop_not_null::BanDropNotNull as pgt_analyse::Rule>::Options;
pub type BanDropTable = <lint::safety::ban_drop_table::BanDropTable as pgt_analyse::Rule>::Options;
pub type BanTruncateCascade =
    <lint::safety::ban_truncate_cascade::BanTruncateCascade as pgt_analyse::Rule>::Options;
