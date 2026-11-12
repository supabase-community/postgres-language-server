use crate::utils::{alter_table_cmds, altered_relation, builtin_type_name, cmd_def};
use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_catalog::{
    CatalogView, Decision, Generated, Lookup, Type, base_type, is_binary_coercible,
    normalize_type_name,
};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::{ColumnDef, RangeVar};
use pgls_query_ext::utils::string_value;

declare_lint_rule! {
    /// Changing a column type may require a table rewrite and break existing clients.
    ///
    /// Most column type changes require an exclusive lock on the table while the entire
    /// table is rewritten. This can take a long time for large tables and will block
    /// reads and writes.
    ///
    /// A type change doesn't rewrite the table when the old type is binary coercible to the new
    /// one, like `varchar` to `text`, or when the column keeps its type and only drops a type
    /// modifier, like `numeric(10,2)` to `numeric`. A `USING` clause other than the column itself
    /// rewrites the table. The old type comes from the database and the statements before; when
    /// it isn't known, only changes to these types count as safe:
    /// - `text`
    /// - `varchar` without a length limit
    /// - `numeric` without a precision
    ///
    /// Changing the type of a virtual generated column (Postgres 18) doesn't rewrite the table
    /// either: its values are computed when rows are read.
    ///
    /// For unsafe type changes, consider creating a new column with the desired type,
    /// migrating the data, and then dropping the old column.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// ALTER TABLE "core_recipe" ALTER COLUMN "count" TYPE bigint;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// ALTER TABLE "core_recipe" ALTER COLUMN "edits" TYPE text;
    /// ```
    ///
    pub ChangingColumnType {
        version: "0.15.0",
        name: "changingColumnType",
        severity: Severity::Warning,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("changing-column-type")],
    }
}

impl LinterRule for ChangingColumnType {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        // Other transactions can't see a table created earlier in the transaction, so its
        // rewrite blocks no one.
        if altered_relation(ctx.stmt()).is_some_and(|relation| ctx.is_new_relation(relation)) {
            return vec![];
        }
        let mut diagnostics = Vec::new();
        let relation = match ctx.stmt() {
            pgls_query::NodeEnum::AlterTableStmt(stmt) => stmt.relation.as_ref(),
            _ => None,
        };

        for cmd in alter_table_cmds(ctx.stmt()) {
            if cmd.subtype() == pgls_query::protobuf::AlterTableType::AtAlterColumnType {
                if let Some(pgls_query::NodeEnum::ColumnDef(col_def)) = cmd_def(cmd)
                    && !rewrites(ctx, relation, &cmd.name, col_def)
                {
                    continue;
                }
                if is_virtual_column(ctx, relation, &cmd.name) {
                    continue;
                }

                diagnostics.push(LinterDiagnostic::new(
                        rule_category!(),
                        None,
                        markup! {
                            "Changing a column type requires a table rewrite and blocks reads and writes."
                        }
                    ).detail(None, "Consider creating a new column with the desired type, migrating data, and then dropping the old column."));
            }
        }

        diagnostics
    }
}

/// Whether the column is a virtual generated column, which [`ATExecAlterColumnType`] doesn't
/// rewrite.
///
/// [`ATExecAlterColumnType`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L14747
fn is_virtual_column(
    ctx: &LinterRuleContext<ChangingColumnType>,
    relation: Option<&RangeVar>,
    column: &str,
) -> bool {
    let Some(relation) = relation else {
        return false;
    };
    ctx.catalog()
        .column_of(relation, column, ctx.session().search_path())
        .found()
        .is_some_and(|column| column.generated == Generated::Virtual)
}

/// Whether `ALTER COLUMN ... TYPE` rewrites the table. Postgres coerces the column, or the
/// `USING` expression, to the new type, and rewrites the table unless the result is the column
/// itself, relabeled to the new type ([`ATPrepAlterColumnType`], [`ATColumnChangeRequiresRewrite`]).
///
/// [`ATPrepAlterColumnType`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L14506
/// [`ATColumnChangeRequiresRewrite`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L14700
fn rewrites(
    ctx: &LinterRuleContext<ChangingColumnType>,
    relation: Option<&RangeVar>,
    column: &str,
    col_def: &ColumnDef,
) -> bool {
    let using = col_def
        .raw_default
        .as_deref()
        .and_then(|using| using.node.as_ref());
    if using.is_some_and(|using| !is_column(using, column)) {
        return true;
    }
    match type_change_rewrites(ctx, relation, column, col_def) {
        Decision::Known(rewrites) => rewrites,
        Decision::Unknown => !is_safe_type_widening(col_def),
    }
}

/// Whether `expr` is a reference to `column`, like `USING price`.
fn is_column(expr: &pgls_query::NodeEnum, column: &str) -> bool {
    match expr {
        pgls_query::NodeEnum::ColumnRef(column_ref) => {
            matches!(column_ref.fields.as_slice(), [field] if string_value(field) == Some(column))
        }
        _ => false,
    }
}

/// Whether changing the column from its old type to the new one rewrites the table. A binary
/// coercible type becomes a `RelabelType`, which doesn't rewrite. Postgres applies a type
/// modifier with a function, which only keeps the column if the old modifier fits in the new one,
/// and the catalog doesn't know the old modifier. Any other coercion is a function call or an
/// I/O conversion and rewrites, except for domains without constraints, and for `timestamp` and
/// `timestamptz`, which don't rewrite when the session's time zone is UTC.
fn type_change_rewrites(
    ctx: &LinterRuleContext<ChangingColumnType>,
    relation: Option<&RangeVar>,
    column: &str,
    col_def: &ColumnDef,
) -> Decision<bool> {
    let catalog = ctx.catalog();
    let search_path = ctx.session().search_path();
    let Some(relation) = relation else {
        return Decision::Unknown;
    };
    let Lookup::Found(relation) = catalog.relation_of(relation, search_path) else {
        return Decision::Unknown;
    };
    let Some(Type::Named(old)) = relation
        .columns
        .iter()
        .flatten()
        .find(|info| info.name == column)
        .and_then(|info| info.ty.clone())
    else {
        return Decision::Unknown;
    };
    let Some(type_name) = col_def.type_name.as_ref() else {
        return Decision::Unknown;
    };
    let (Some(new), _) = normalize_type_name(catalog, type_name, search_path) else {
        return Decision::Unknown;
    };
    let has_type_modifier = !type_name.typmods.is_empty();
    if old == new {
        return if has_type_modifier {
            Decision::Unknown
        } else {
            Decision::Known(false)
        };
    }
    if base_type(catalog, &new) != Decision::Known(new.clone()) {
        return Decision::Unknown;
    }
    let timestamps = [
        catalog.builtin_type("timestamp"),
        catalog.builtin_type("timestamptz"),
    ];
    if timestamps.contains(&Some(old.clone())) && timestamps.contains(&Some(new.clone())) {
        return Decision::Unknown;
    }
    match is_binary_coercible(catalog, &old, &new) {
        Decision::Known(true) if has_type_modifier => Decision::Unknown,
        Decision::Known(binary_coercible) => Decision::Known(!binary_coercible),
        Decision::Unknown => Decision::Unknown,
    }
}

/// The guess when the old type isn't known.
fn is_safe_type_widening(col_def: &ColumnDef) -> bool {
    let Some(type_name) = &col_def.type_name else {
        return false;
    };

    // An array of the type is a different type.
    let Some(target_type) =
        builtin_type_name(type_name).filter(|_| type_name.array_bounds.is_empty())
    else {
        return false;
    };

    let has_type_modifier = !type_name.typmods.is_empty();

    match target_type {
        // text is always safe — binary compatible with varchar/char
        "text" => true,
        // varchar without length is safe (dropping a length constraint)
        "varchar" if !has_type_modifier => true,
        // numeric (or decimal, which the parser turns into numeric) without precision is safe
        // (dropping precision constraint)
        "numeric" if !has_type_modifier => true,
        _ => false,
    }
}
