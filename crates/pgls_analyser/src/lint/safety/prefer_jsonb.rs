use pgls_analyse::{Rule, RuleDiagnostic, RuleSource, context::RuleContext, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Prefer JSONB over JSON types.
    ///
    /// JSONB is the binary JSON data type in PostgreSQL that is more efficient for most use cases.
    /// Unlike JSON, JSONB stores data in a decomposed binary format which provides several advantages:
    /// - Significantly faster query performance for operations like indexing and searching
    /// - Support for indexing (GIN indexes)
    /// - Duplicate keys are automatically removed
    /// - Keys are sorted
    ///
    /// The only reasons to use JSON instead of JSONB are:
    /// - You need to preserve exact input formatting (whitespace, key order)
    /// - You need to preserve duplicate keys
    /// - You have very specific performance requirements where JSON's lack of parsing overhead matters
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// CREATE TABLE users (
    ///     data json
    /// );
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// ALTER TABLE users ADD COLUMN metadata json;
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// ALTER TABLE users ALTER COLUMN data TYPE json;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// CREATE TABLE users (
    ///     data jsonb
    /// );
    /// ```
    ///
    /// ```sql
    /// ALTER TABLE users ADD COLUMN metadata jsonb;
    /// ```
    ///
    /// ```sql
    /// ALTER TABLE users ALTER COLUMN data TYPE jsonb;
    /// ```
    ///
    pub PreferJsonb {
        version: "next",
        name: "preferJsonb",
        severity: Severity::Warning,
        recommended: false,
        sources: &[RuleSource::Eugene("E3")],
    }
}

impl Rule for PreferJsonb {
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<RuleDiagnostic> {
        let mut diagnostics = Vec::new();

        match &ctx.stmt() {
            pgls_query::NodeEnum::CreateStmt(stmt) => {
                for table_elt in &stmt.table_elts {
                    if let Some(pgls_query::NodeEnum::ColumnDef(col_def)) = &table_elt.node {
                        check_column_def(&mut diagnostics, col_def);
                    }
                }
            }
            pgls_query::NodeEnum::AlterTableStmt(stmt) => {
                for cmd in &stmt.cmds {
                    if let Some(pgls_query::NodeEnum::AlterTableCmd(cmd)) = &cmd.node {
                        if cmd.subtype() == pgls_query::protobuf::AlterTableType::AtAddColumn
                            || cmd.subtype()
                                == pgls_query::protobuf::AlterTableType::AtAlterColumnType
                        {
                            if let Some(pgls_query::NodeEnum::ColumnDef(col_def)) =
                                &cmd.def.as_ref().and_then(|d| d.node.as_ref())
                            {
                                check_column_def(&mut diagnostics, col_def);
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        diagnostics
    }
}

fn check_column_def(
    diagnostics: &mut Vec<RuleDiagnostic>,
    col_def: &pgls_query::protobuf::ColumnDef,
) {
    let Some(type_name) = &col_def.type_name else {
        return;
    };

    for name_node in &type_name.names {
        let Some(pgls_query::NodeEnum::String(name)) = &name_node.node else {
            continue;
        };

        let type_name_lower = name.sval.to_lowercase();
        if type_name_lower != "json" {
            continue;
        }

        diagnostics.push(
            RuleDiagnostic::new(
                rule_category!(),
                None,
                markup! {
                    "Prefer JSONB over JSON for better performance and functionality."
                },
            )
            .detail(None, "JSON stores exact text representation while JSONB stores parsed binary format. JSONB is faster for queries, supports indexing, and removes duplicate keys.")
            .note("Consider using JSONB instead unless you specifically need to preserve formatting or duplicate keys."),
        );
    }
}
