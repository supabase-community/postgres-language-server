use pgls_analyse::{Rule, RuleDiagnostic, RuleSource, context::RuleContext, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Prefer BIGINT over SMALLINT types.
    ///
    /// SMALLINT has a very limited range (-32,768 to 32,767) that is easily exceeded.
    /// Even for values that seem small initially, using SMALLINT can lead to problems
    /// as your application grows.
    ///
    /// The storage savings of SMALLINT (2 bytes) vs BIGINT (8 bytes) are negligible
    /// on modern systems, while the cost of migrating when you exceed the limit is high.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// CREATE TABLE users (
    ///     age smallint
    /// );
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// CREATE TABLE products (
    ///     quantity smallserial
    /// );
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// CREATE TABLE users (
    ///     age integer
    /// );
    /// ```
    ///
    /// ```sql
    /// CREATE TABLE products (
    ///     quantity bigint
    /// );
    /// ```
    ///
    pub PreferBigintOverSmallint {
        version: "next",
        name: "preferBigintOverSmallint",
        severity: Severity::Warning,
        recommended: false,
        sources: &[RuleSource::Squawk("prefer-bigint-over-smallint")],
    }
}

impl Rule for PreferBigintOverSmallint {
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
        if !matches!(
            type_name_lower.as_str(),
            "smallint" | "int2" | "smallserial" | "serial2"
        ) {
            continue;
        }

        diagnostics.push(
            RuleDiagnostic::new(
                rule_category!(),
                None,
                markup! {
                    "SMALLINT has a very limited range that is easily exceeded."
                },
            )
            .detail(None, "SMALLINT can only store values from -32,768 to 32,767. This range is often insufficient.")
            .note("Consider using INTEGER or BIGINT for better range and future-proofing."),
        );
    }
}
