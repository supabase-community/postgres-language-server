//! Generated file, do not edit by hand, see `xtask/codegen`

use pglt_analyse::declare_lint_group;
pub mod adding_required_field;
pub mod ban_drop_column;
pub mod ban_drop_not_null;
pub mod ban_drop_table;
declare_lint_group! { pub Safety { name : "safety" , rules : [self :: adding_required_field :: AddingRequiredField , self :: ban_drop_column :: BanDropColumn , self :: ban_drop_not_null :: BanDropNotNull , self :: ban_drop_table :: BanDropTable ,] } }
