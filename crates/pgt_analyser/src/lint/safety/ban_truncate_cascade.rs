use pgt_analyse::{
    AnalysedFileContext, Rule, RuleDiagnostic, RuleSource, context::RuleContext, declare_lint_rule,
};
use pgt_console::markup;
use pgt_diagnostics::Severity;
use pgt_query_ext::protobuf::DropBehavior;
use pgt_schema_cache::SchemaCache;

declare_lint_rule! {
    /// Using `TRUNCATE`'s `CASCADE` option will truncate any tables that are also foreign-keyed to the specified tables.
    ///
    /// So if you had tables with foreign-keys like:
    ///
    /// `a <- b <- c`
    ///
    /// and ran:
    ///
    /// `truncate a cascade;`
    ///
    /// You'd end up with a, b, & c all being truncated!
    ///
    /// Instead, you can manually specify the tables you want.
    ///
    /// `truncate a, b;`
    pub BanTruncateCascade {
        version: "next",
        name: "banTruncateCascade",
        severity: Severity::Error,
        recommended: false,
        sources: &[RuleSource::Squawk("ban-truncate-cascade")],
    }
}

impl Rule for BanTruncateCascade {
    type Options = ();

    fn run(
        ctx: &RuleContext<Self>,
        _file_context: &AnalysedFileContext,
        _schema_cache: Option<&SchemaCache>,
    ) -> Vec<RuleDiagnostic> {
        let mut diagnostics = Vec::new();

        if let pgt_query_ext::NodeEnum::TruncateStmt(stmt) = &ctx.stmt() {
            if stmt.behavior() == DropBehavior::DropCascade {
                diagnostics.push(RuleDiagnostic::new(
                            rule_category!(),
                            None,
                            markup! {
                                "The `CASCADE` option will also truncate any tables that are foreign-keyed to the specified tables."
                            },
                        ).detail(None, "Do not use the `CASCADE` option. Instead, specify manually what you want: `TRUNCATE a, b;`."));
            }
        }

        diagnostics
    }
}
