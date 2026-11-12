use std::collections::HashSet;

use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::declare_lint_rule;
use pgls_catalog::CatalogView;
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::{Node, RangeVar, WithClause};
use pgls_query::{NodeEnum, NodeRef};

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

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let tokens = ctx.tokens();
        let mut relations: Vec<&RangeVar> = Vec::new();
        // Relations that are not looked up through the search path, by address: the tables of
        // `SELECT INTO`, which `requireTableSchema` checks, the aliases of `FOR UPDATE OF`, and
        // the references to CTEs.
        let mut skipped: HashSet<*const RangeVar> = HashSet::new();
        for query in queries(ctx.stmt()) {
            for node in query.iter() {
                match node {
                    NodeRef::RangeVar(relation) => relations.push(relation),
                    NodeRef::IntoClause(into) => {
                        skipped.extend(into.rel.as_ref().map(std::ptr::from_ref));
                    }
                    NodeRef::LockingClause(locking) => {
                        skipped.extend(range_vars(&locking.locked_rels).map(std::ptr::from_ref));
                    }
                    NodeRef::SelectStmt(stmt) => {
                        skipped.extend(cte_references(node, stmt.with_clause.as_ref()));
                    }
                    NodeRef::InsertStmt(stmt) => {
                        skipped.extend(cte_references(node, stmt.with_clause.as_ref()));
                    }
                    NodeRef::UpdateStmt(stmt) => {
                        skipped.extend(cte_references(node, stmt.with_clause.as_ref()));
                    }
                    NodeRef::DeleteStmt(stmt) => {
                        skipped.extend(cte_references(node, stmt.with_clause.as_ref()));
                    }
                    NodeRef::MergeStmt(stmt) => {
                        skipped.extend(cte_references(node, stmt.with_clause.as_ref()));
                    }
                    _ => {}
                }
            }
        }

        // An unqualified name refers to a temporary table if there is one, since `pg_temp` comes
        // first on the search path.
        let is_temporary = |name: &str| {
            ctx.catalog()
                .relation(None, name, ctx.session().search_path())
                .found()
                .is_some_and(|relation| relation.schema == "pg_temp")
        };
        relations.sort_by_key(|relation| relation.location);
        relations
            .into_iter()
            .filter(|relation| {
                relation.schemaname.is_empty()
                    && !skipped.contains(&std::ptr::from_ref(*relation))
                    && !is_temporary(&relation.relname)
            })
            .map(|relation| {
                let name = &relation.relname;
                LinterDiagnostic::new(
                    rule_category!(),
                    tokens.and_then(|tokens| tokens.range_at(relation.location)),
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

/// The parts of a statement that are queries or DML, or that name the relations they read or
/// write, like `TRUNCATE`.
fn queries(stmt: &NodeEnum) -> Vec<NodeRef<'_>> {
    let stmt = match stmt {
        NodeEnum::ExplainStmt(explain) => match node_enum(explain.query.as_deref()) {
            Some(query) => query,
            None => return vec![],
        },
        _ => stmt,
    };
    fn query(query: &Option<Box<Node>>) -> Option<NodeRef<'_>> {
        node_enum(query.as_deref()).map(NodeEnum::to_ref)
    }
    match stmt {
        NodeEnum::SelectStmt(_)
        | NodeEnum::InsertStmt(_)
        | NodeEnum::UpdateStmt(_)
        | NodeEnum::DeleteStmt(_)
        | NodeEnum::MergeStmt(_) => vec![stmt.to_ref()],
        NodeEnum::ViewStmt(view) => query(&view.query).into_iter().collect(),
        NodeEnum::CreateTableAsStmt(create) => query(&create.query).into_iter().collect(),
        NodeEnum::DeclareCursorStmt(declare) => query(&declare.query).into_iter().collect(),
        NodeEnum::PrepareStmt(prepare) => query(&prepare.query).into_iter().collect(),
        NodeEnum::CopyStmt(copy) => copy
            .relation
            .iter()
            .map(RangeVar::to_ref)
            .chain(query(&copy.query))
            .collect(),
        NodeEnum::TruncateStmt(truncate) => range_vars(&truncate.relations)
            .map(RangeVar::to_ref)
            .collect(),
        NodeEnum::LockStmt(lock) => range_vars(&lock.relations).map(RangeVar::to_ref).collect(),
        _ => vec![],
    }
}

fn node_enum(node: Option<&Node>) -> Option<&NodeEnum> {
    node?.node.as_ref()
}

fn range_vars(nodes: &[Node]) -> impl Iterator<Item = &RangeVar> {
    nodes.iter().filter_map(|node| match &node.node {
        Some(NodeEnum::RangeVar(relation)) => Some(relation),
        _ => None,
    })
}

/// The relations in `query` that refer to a CTE of its `WITH` clause, by address. The CTEs are
/// in scope in the query, and in the CTEs after them. In `WITH RECURSIVE`, they are in scope
/// in all CTEs.
fn cte_references<'a>(query: NodeRef<'a>, with: Option<&'a WithClause>) -> Vec<*const RangeVar> {
    let Some(with) = with else {
        return vec![];
    };
    let ctes: Vec<(&str, Option<NodeRef>)> = with
        .ctes
        .iter()
        .filter_map(|cte| match &cte.node {
            Some(NodeEnum::CommonTableExpr(cte)) => Some((
                cte.ctename.as_str(),
                node_enum(cte.ctequery.as_deref()).map(NodeEnum::to_ref),
            )),
            _ => None,
        })
        .collect();
    let refers_to_cte = |relation: &RangeVar, in_scope: usize| {
        relation.schemaname.is_empty()
            && ctes
                .iter()
                .take(in_scope)
                .any(|(name, _)| *name == relation.relname)
    };
    let mut references = Vec::new();
    for (index, (_, cte_query)) in ctes.iter().enumerate() {
        let in_scope = if with.recursive { ctes.len() } else { index };
        references.extend(
            cte_query
                .iter()
                .flat_map(NodeRef::iter)
                .filter_map(|node| match node {
                    NodeRef::RangeVar(relation) if refers_to_cte(relation, in_scope) => {
                        Some(std::ptr::from_ref(relation))
                    }
                    _ => None,
                }),
        );
    }
    // The relations of the query outside its `WITH` clause.
    let in_with: HashSet<*const RangeVar> = with
        .to_ref()
        .iter()
        .filter_map(|node| match node {
            NodeRef::RangeVar(relation) => Some(std::ptr::from_ref(relation)),
            _ => None,
        })
        .collect();
    references.extend(query.iter().filter_map(|node| match node {
        NodeRef::RangeVar(relation)
            if refers_to_cte(relation, ctes.len())
                && !in_with.contains(&std::ptr::from_ref(relation)) =>
        {
            Some(std::ptr::from_ref(relation))
        }
        _ => None,
    }));
    references
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
    fn skips_the_target_of_select_into_and_locked_aliases() {
        assert_eq!(
            spans("select * into public.t from u for update of u"),
            ["u"]
        );
    }
}
