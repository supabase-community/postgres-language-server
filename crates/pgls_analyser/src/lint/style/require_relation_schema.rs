use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::declare_lint_rule;
use pgls_catalog::resolve::{LookupKind, SearchPathLookup};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Relations in queries should be schema-qualified.
    ///
    /// Without a schema, Postgres looks the table or view up in the schemas on the
    /// `search_path`, starting with `pg_temp`. A relation with the same name in a schema earlier
    /// on the search path, like a temporary table that any user can create, is used instead of
    /// the intended one.
    ///
    /// The rule is meant for code where this matters, like functions with `SECURITY DEFINER` and
    /// extension scripts, as defence in depth next to setting the `search_path` explicitly.
    ///
    /// The rule checks the relations that queries and DML read or write: `FROM`, `JOIN`,
    /// subqueries, CTEs, the targets of `INSERT`, `UPDATE`, `DELETE` and `MERGE`, and their
    /// `USING`, as well as the queries of views, `CREATE TABLE AS`, `EXPLAIN`, `DECLARE` and
    /// `PREPARE`, and the relations of `COPY`, `TRUNCATE` and `LOCK`. Names that refer to a CTE
    /// and temporary tables, including the ones created earlier in the file, are not reported.
    /// A CTE doesn't hide the target of `INSERT`, `UPDATE`, `DELETE` or `MERGE`, so a target
    /// with the name of a CTE is reported.
    /// The tables that DDL creates, changes or drops are checked by `requireTableSchema`.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// select * from pg_statio_all_tables;
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// insert into users (name) values ('alice');
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// select * from pg_catalog.pg_statio_all_tables;
    /// with recent as (select * from public.posts) select * from recent;
    /// ```
    ///
    pub RequireRelationSchema {
        version: "next",
        name: "requireRelationSchema",
        severity: Severity::Warning,
        recommended: false,
    }
}

impl LinterRule for RequireRelationSchema {
    type Options = ();
    const READS_LOOKUPS: bool = true;

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut relations: Vec<&SearchPathLookup> = ctx
            .lookups()
            .iter()
            .filter(|lookup| lookup.kind == LookupKind::Relation)
            .collect();
        relations.sort_by_key(|relation| relation.location);
        relations
            .into_iter()
            .map(|relation| {
                let name = &relation.name;
                LinterDiagnostic::new(
                    rule_category!(),
                    relation.span,
                    markup! {
                        "Relation "<Emphasis>{name}</Emphasis>" is not schema-qualified."
                    },
                )
                .detail(
                    None,
                    "Without a schema, the relation is resolved through the search_path: a relation with the same name earlier on the search_path, like a temporary table, is used instead.",
                )
                .note(markup! {
                    "Specify the schema, e.g. "<Emphasis>"public."{name}</Emphasis>"."
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    fn spans(sql: &str) -> Vec<String> {
        crate::spanned_texts("style", "requireRelationSchema", sql)
    }

    #[test]
    fn points_at_the_relation_name() {
        assert_eq!(
            spans("select * from \"Users\" u join public.posts p on true, comments"),
            ["\"Users\"", "comments"]
        );
    }

    #[test]
    fn skips_cte_references() {
        assert_eq!(
            spans("with a as (select * from b), b as (select * from a) select * from a, b"),
            ["b"]
        );
        assert!(
            spans("with recursive r as (select 1 union all select * from r) select * from r")
                .is_empty()
        );
    }

    #[test]
    fn reports_dml_targets_with_the_name_of_a_cte() {
        assert_eq!(
            spans("with users as (select 1 as id) update users set id = 2 from users u"),
            ["users"]
        );
        assert_eq!(
            spans(
                "with users as (select 1 as id) merge into users t using users s on true \
                 when matched then delete"
            ),
            ["users"]
        );
    }

    #[test]
    fn skips_the_target_of_select_into_and_locked_aliases() {
        assert_eq!(
            spans("select * into public.t from u for update of u"),
            ["u"]
        );
    }
}
