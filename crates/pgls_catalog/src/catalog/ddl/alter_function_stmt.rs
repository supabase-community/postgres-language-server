use pgls_query::{
    NodeEnum,
    protobuf::{AlterFunctionStmt, Node},
};

use crate::Behavior;
use crate::catalog::Catalog;
use crate::lookup::Origin;

/// `ALTER FUNCTION`. Models the volatility change of [`AlterFunction`]; the other options don't
/// change what the catalog knows about a function.
///
/// [`AlterFunction`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/functioncmds.c#L1361
pub(super) fn apply_alter_function_stmt(
    c: &mut Catalog,
    n: &AlterFunctionStmt,
    search_path: &[String],
) {
    let Some(volatility) = volatility_option(&n.actions) else {
        return;
    };
    let Some(object) = &n.func else {
        return;
    };
    // If the overload can't be identified, its function set becomes unknown.
    if let Some(mut function) = c.take_function(object, search_path) {
        function.volatility = Some(volatility);
        function.origin = Origin::File;
        c.add_function(function);
    }
}

/// The volatility that the options of `CREATE FUNCTION` or `ALTER FUNCTION` set, if any.
pub(super) fn volatility_option(options: &[Node]) -> Option<Behavior> {
    options.iter().find_map(|option| match &option.node {
        Some(NodeEnum::DefElem(def)) if def.defname == "volatility" => {
            match def.arg.as_ref().and_then(|arg| arg.node.as_ref()) {
                Some(NodeEnum::String(value)) => match value.sval.as_str() {
                    "immutable" => Some(Behavior::Immutable),
                    "stable" => Some(Behavior::Stable),
                    "volatile" => Some(Behavior::Volatile),
                    _ => None,
                },
                _ => None,
            }
        }
        _ => None,
    })
}
