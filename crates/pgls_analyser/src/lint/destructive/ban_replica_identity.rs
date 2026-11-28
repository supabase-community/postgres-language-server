use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::NodeEnum;
use pgls_query::protobuf::{AlterPublicationAction, AlterTableType, ObjectType};

declare_lint_rule! {
    /// Changing the replica identity of a table or a publication may break logical replication
    /// consumers.
    ///
    /// The replica identity decides which old values of a row logical replication sends for
    /// updates and deletes. Dropping a publication, dropping or replacing its objects, or
    /// changing its options stops or changes what its consumers receive.
    ///
    /// Adding objects to a publication is not reported.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// alter table orders replica identity full;
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// alter publication analytics drop table orders;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// alter publication analytics add table orders;
    /// ```
    ///
    pub BanReplicaIdentity {
        version: "next",
        name: "banReplicaIdentity",
        severity: Severity::Warning,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("ban-replica-identity")],
    }
}

impl LinterRule for BanReplicaIdentity {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let note = "Update the replication consumers before the change.";

        let publication = match ctx.stmt() {
            NodeEnum::DropStmt(stmt) if stmt.remove_type() == ObjectType::ObjectPublication => {
                Some("Dropping a publication stops replication for its consumers.")
            }
            NodeEnum::AlterPublicationStmt(stmt)
                if stmt.action() != AlterPublicationAction::ApAddObjects
                    || !stmt.options.is_empty() =>
            {
                Some(
                    "Changing the objects or options of a publication changes what its consumers receive.",
                )
            }
            _ => None,
        };
        if let Some(message) = publication {
            return vec![
                LinterDiagnostic::new(rule_category!(), None, markup! { {message} })
                    .detail(
                        None,
                        "Consumers may miss changes or receive changes they don't expect.",
                    )
                    .note(note),
            ];
        }

        ctx.stmt().alter_table_commands()
            .filter(|cmd| cmd.subtype() == AlterTableType::AtReplicaIdentity)
            .map(|_| {
                LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "Changing the replica identity of a table may break logical replication consumers."
                    },
                )
                .detail(
                    None,
                    "Consumers receive different old values of the rows that are updated or deleted.",
                )
                .note(note)
            })
            .collect()
    }
}
