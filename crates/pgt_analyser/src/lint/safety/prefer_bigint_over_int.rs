use pgt_analyse::{Rule, RuleDiagnostic, RuleSource, context::RuleContext, declare_lint_rule};
use pgt_console::markup;
use pgt_diagnostics::Severity;

declare_lint_rule! {
    /// Prefer BIGINT over INT/INTEGER types.
    ///
    /// Using INTEGER (INT4) can lead to overflow issues, especially for ID columns.
    /// While SMALLINT might be acceptable for certain use cases with known small ranges,
    /// INTEGER often becomes a limiting factor as applications grow.
    ///
    /// The storage difference between INTEGER (4 bytes) and BIGINT (8 bytes) is minimal,
    /// but the cost of migrating when you hit the 2.1 billion limit can be significant.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// CREATE TABLE users (
    ///     id integer
    /// );
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// CREATE TABLE users (
    ///     id serial
    /// );
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// CREATE TABLE users (
    ///     id bigint
    /// );
    /// ```
    ///
    /// ```sql
    /// CREATE TABLE users (
    ///     id bigserial
    /// );
    /// ```
    ///
    /// ```sql
    /// CREATE TABLE users (
    ///     id smallint
    /// );
    /// ```
    ///
    pub PreferBigintOverInt {
        version: "next",
        name: "preferBigintOverInt",
        severity: Severity::Warning,
        recommended: false,
        sources: &[RuleSource::Squawk("prefer-bigint-over-int")],
    }
}

impl Rule for PreferBigintOverInt {
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<RuleDiagnostic> {
        let mut diagnostics = Vec::new();

        match &ctx.stmt() {
            pgt_query::NodeEnum::CreateStmt(stmt) => {
                for table_elt in &stmt.table_elts {
                    if let Some(pgt_query::NodeEnum::ColumnDef(col_def)) = &table_elt.node {
                        check_column_def(&mut diagnostics, col_def);
                    }
                }
            }
            pgt_query::NodeEnum::AlterTableStmt(stmt) => {
                for cmd in &stmt.cmds {
                    if let Some(pgt_query::NodeEnum::AlterTableCmd(cmd)) = &cmd.node {
                        if cmd.subtype() == pgt_query::protobuf::AlterTableType::AtAddColumn
                            || cmd.subtype()
                                == pgt_query::protobuf::AlterTableType::AtAlterColumnType
                        {
                            if let Some(pgt_query::NodeEnum::ColumnDef(col_def)) =
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
    col_def: &pgt_query::protobuf::ColumnDef,
) {
    if let Some(type_name) = &col_def.type_name {
        for name_node in &type_name.names {
            if let Some(pgt_query::NodeEnum::String(name)) = &name_node.node {
                let type_name_lower = name.sval.to_lowercase();
                // Only check for INT4/INTEGER types, not SMALLINT
                let is_int4 = matches!(
                    type_name_lower.as_str(),
                    "integer" | "int4" | "serial" | "serial4"
                );

                if is_int4 {
                    diagnostics.push(
                        RuleDiagnostic::new(
                            rule_category!(),
                            None,
                            markup! {
                                "INTEGER type may lead to overflow issues."
                            },
                        )
                        .detail(None, "INTEGER has a maximum value of 2,147,483,647 which can be exceeded by ID columns and counters.")
                        .note("Consider using BIGINT instead for better future-proofing."),
                    );
                }
            }
        }
    }
}
