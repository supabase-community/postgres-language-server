use pgls_analyse::{Rule, RuleDiagnostic, RuleSource, context::RuleContext, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Using CHAR(n) or CHARACTER(n) types is discouraged.
    ///
    /// CHAR types are fixed-length and padded with spaces, which can lead to unexpected behavior
    /// when comparing strings or concatenating values. They also waste storage space when values
    /// are shorter than the declared length.
    ///
    /// Use VARCHAR or TEXT instead for variable-length character data.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// CREATE TABLE "core_bar" (
    ///     "id" serial NOT NULL PRIMARY KEY,
    ///     "alpha" char(100) NOT NULL
    /// );
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// CREATE TABLE "core_bar" (
    ///     "id" serial NOT NULL PRIMARY KEY,
    ///     "alpha" varchar(100) NOT NULL
    /// );
    /// ```
    ///
    pub BanCharField {
        version: "next",
        name: "banCharField",
        severity: Severity::Warning,
        recommended: false,
        sources: &[RuleSource::Squawk("ban-char-field")],
    }
}

impl Rule for BanCharField {
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<RuleDiagnostic> {
        let mut diagnostics = Vec::new();

        if let pgls_query::NodeEnum::CreateStmt(stmt) = &ctx.stmt() {
            for table_elt in &stmt.table_elts {
                if let Some(pgls_query::NodeEnum::ColumnDef(col_def)) = &table_elt.node {
                    if let Some(diagnostic) = check_column_for_char_type(col_def) {
                        diagnostics.push(diagnostic);
                    }
                }
            }
        }

        // Also check ALTER TABLE ADD COLUMN statements
        if let pgls_query::NodeEnum::AlterTableStmt(stmt) = &ctx.stmt() {
            for cmd in &stmt.cmds {
                if let Some(pgls_query::NodeEnum::AlterTableCmd(cmd)) = &cmd.node {
                    if cmd.subtype() == pgls_query::protobuf::AlterTableType::AtAddColumn {
                        if let Some(pgls_query::NodeEnum::ColumnDef(col_def)) =
                            &cmd.def.as_ref().and_then(|d| d.node.as_ref())
                        {
                            if let Some(diagnostic) = check_column_for_char_type(col_def) {
                                diagnostics.push(diagnostic);
                            }
                        }
                    }
                }
            }
        }

        diagnostics
    }
}

fn check_column_for_char_type(col_def: &pgls_query::protobuf::ColumnDef) -> Option<RuleDiagnostic> {
    if let Some(type_name) = &col_def.type_name {
        for name_node in &type_name.names {
            if let Some(pgls_query::NodeEnum::String(name)) = &name_node.node {
                // Check for "bpchar" (internal name for CHAR type)
                // or "char" or "character"
                let type_str = name.sval.to_lowercase();
                if type_str == "bpchar" || type_str == "char" || type_str == "character" {
                    return Some(
                        RuleDiagnostic::new(
                            rule_category!(),
                            None,
                            markup! {
                                "CHAR type is discouraged due to space padding behavior."
                            },
                        )
                        .detail(None, "CHAR types are fixed-length and padded with spaces, which can lead to unexpected behavior.")
                        .note("Use VARCHAR or TEXT instead for variable-length character data."),
                    );
                }
            }
        }
    }
    None
}
