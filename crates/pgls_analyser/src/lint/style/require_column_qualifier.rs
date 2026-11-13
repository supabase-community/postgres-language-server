use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::declare_lint_rule;
use pgls_catalog::resolve::{LookupKind, SearchPathLookup};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Columns in queries should be qualified with their table.
    ///
    /// Without a table name or alias, Postgres looks the column up in every relation of the
    /// query, and then in the ones of the queries around it. If another relation gets a column
    /// with the same name, the query fails, or a subquery silently refers to the column of the
    /// outer query. In the body of a SQL function, a column with the name of a parameter takes
    /// its place.
    ///
    /// The rule is meant for code where this matters, like functions with `SECURITY DEFINER` and
    /// extension scripts, as defence in depth next to qualifying relations, functions, types and
    /// operators with their schema.
    ///
    /// The rule checks the column references of queries and DML that read from or write to a
    /// relation. It does not report:
    ///
    /// - queries without relations, like `select 1`
    /// - references to output columns or their positions in `ORDER BY`, `GROUP BY` and
    ///   `DISTINCT ON`, and the `ORDER BY` of `UNION`, `INTERSECT` and `EXCEPT`
    /// - whole-row references, like `select u from public.users u`
    /// - the parameters of the SQL function the statement is in
    /// - the column lists of `INSERT`, `USING (...)` and `ON CONFLICT (...)`, the targets of
    ///   `UPDATE ... SET`, and the `WHERE` clauses of `ON CONFLICT (...)` and `COPY FROM`, which
    ///   can only name columns of one table
    /// - expressions outside queries, like column definitions, defaults, `CHECK` constraints,
    ///   index expressions, and the expressions and actions of `CREATE POLICY` and `CREATE RULE`
    ///
    /// A name of an output column, relation, alias or parameter is still reported where Postgres
    /// resolves it to a column, like `u` in `select u from public.t u` if `t` has a column `u`,
    /// or `n` in `select u.id as n from public.t u group by n` if `t` has a column `n`: in
    /// `GROUP BY`, unlike in `ORDER BY`, the columns of `FROM` take precedence over the output
    /// columns. An output column can only be named on its own, so in `order by lower(n)`, `n` is
    /// a column. This needs the columns of the relations from the database.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// select relid from pg_catalog.pg_statio_all_tables;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// select at.relid from pg_catalog.pg_statio_all_tables as at;
    /// select at.relid as id from pg_catalog.pg_statio_all_tables as at order by id;
    /// insert into public.users (id, name) values (1, 'alice') on conflict (id) do nothing;
    /// ```
    ///
    pub RequireColumnQualifier {
        version: "next",
        name: "requireColumnQualifier",
        severity: Severity::Warning,
        recommended: false,
    }
}

impl LinterRule for RequireColumnQualifier {
    type Options = ();
    const READS_LOOKUPS: bool = true;

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut lookups: Vec<&SearchPathLookup> = ctx
            .lookups()
            .iter()
            .filter(|lookup| lookup.kind == LookupKind::Column)
            .collect();
        lookups.sort_by_key(|lookup| lookup.location);
        lookups
            .into_iter()
            .map(|lookup| {
                let name = &lookup.name;
                LinterDiagnostic::new(
                    rule_category!(),
                    lookup.span,
                    markup! {
                        "Column "<Emphasis>{name}</Emphasis>" is not qualified with its table."
                    },
                )
                .detail(
                    None,
                    "Without a table, the column is looked up in every relation of the query and of the queries around it, so a column with the same name in another relation can take its place.",
                )
                .note(markup! {
                    "Qualify the column with the name or alias of its table, e.g. "<Emphasis>"t."{name}</Emphasis>"."
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    fn spans(sql: &str) -> Vec<String> {
        crate::spanned_texts("style", "requireColumnQualifier", sql)
    }

    #[test]
    fn points_at_the_column() {
        assert_eq!(
            spans("select \"Id\", u.name, * from public.users u where email = $1"),
            ["\"Id\"", "email"]
        );
        assert_eq!(
            spans("update public.users set name = upper(name) returning id"),
            ["name", "id"]
        );
    }

    #[test]
    fn reports_references_to_outer_queries() {
        assert_eq!(spans("select (select id) from public.users u"), ["id"]);
    }

    #[test]
    fn skips_output_columns() {
        assert_eq!(
            spans(
                "select u.id as n, u.name from public.users u group by n, name order by n, name, 1, lower(email)"
            ),
            ["email"]
        );
        assert!(
            spans(
                "select u.id from public.users u union select p.id from public.posts p order by id"
            )
            .is_empty()
        );
    }

    #[test]
    fn skips_output_columns_through_casts_and_grouping_sets() {
        assert!(
            spans(
                "select u.name::pg_catalog.text, u.id as n from public.users u \
                 group by grouping sets ((n), rollup (n, name), cube ((n, name))), (n, name) \
                 order by name"
            )
            .is_empty()
        );
        assert_eq!(
            spans("select u.id as n from public.users u group by grouping sets ((n, email))"),
            ["email"]
        );
    }

    #[test]
    fn skips_whole_rows_and_queries_without_relations() {
        assert!(spans("select u from public.users u").is_empty());
        assert!(spans("select 1, now()").is_empty());
    }

    #[test]
    fn skips_insert_columns_and_conflict_targets() {
        assert_eq!(
            spans(
                "insert into public.users (id, name) values (1, 'a') \
                 on conflict (id) where active do update set name = excluded.name \
                 returning id"
            ),
            ["id"]
        );
    }

    #[test]
    fn skips_ddl() {
        assert!(spans("create table public.t (a int check (a > 0), b int default 1)").is_empty());
        assert!(spans("create index on public.t (lower(a)) where b > 0").is_empty());
    }
}
