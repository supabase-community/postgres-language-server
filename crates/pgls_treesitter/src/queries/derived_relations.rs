use std::sync::LazyLock;

use crate::queries::{Query, QueryResult};
use pgls_query::Identifier;
use tree_sitter::StreamingIterator;

use super::QueryTryFrom;

static TS_QUERY: LazyLock<Option<tree_sitter::Query>> = LazyLock::new(|| {
    static QUERY_STR: &str = r#"
    [
        (with_query . (any_identifier) @name) @item
        (relation (subquery) (alias (any_identifier) @name)) @item
    ]
"#;
    tree_sitter::Query::new(&pgls_treesitter_grammar::LANGUAGE.into(), QUERY_STR)
        // A query that doesn't compile fails the tests of this module.
        .ok()
});

/// A common table expression, or a subquery in FROM with an alias: a relation whose columns are
/// those of its query rather than of a relation of the database.
#[derive(Debug)]
pub struct DerivedRelationMatch<'a> {
    /// The `with_query`, or the `relation` of the subquery with its alias.
    pub(crate) item: tree_sitter::Node<'a>,
    pub(crate) name: tree_sitter::Node<'a>,
}

impl DerivedRelationMatch<'_> {
    /// The name of the CTE or the alias of the subquery, see [`Identifier::name`].
    pub fn get_name(&self, sql: &str) -> String {
        Identifier::new(self.name.utf8_text(sql.as_bytes()).unwrap_or_default()).name()
    }

    /// A query whose output columns are the columns of the relation: `select * from` the
    /// relation, after the CTEs it can reference. For a CTE, those before it in its WITH
    /// clause; for a subquery, the CTEs of the nearest query that has some. `None` if that text
    /// contains `position`, where the user is typing.
    pub fn columns_query(&self, sql: &str, position: usize) -> Option<String> {
        let (with, from) = if self.item.kind() == "with_query" {
            let cte = self.item.parent()?;
            (
                Some(cte.start_byte()..self.item.end_byte()),
                self.name.byte_range(),
            )
        } else {
            (
                self.enclosing_cte().map(|cte| cte.byte_range()),
                self.item.byte_range(),
            )
        };
        if with
            .iter()
            .chain([&from])
            .any(|range| (range.start..=range.end).contains(&position))
        {
            return None;
        }
        let from = sql.get(from)?;
        match with {
            Some(with) => Some(format!("{} select * from {from}", sql.get(with)?)),
            None => Some(format!("select * from {from}")),
        }
    }

    /// The WITH clause of the nearest query around the subquery that has one.
    fn enclosing_cte(&self) -> Option<tree_sitter::Node<'_>> {
        let mut node = self.item.parent();
        while let Some(current) = node {
            if matches!(current.kind(), "statement" | "subquery")
                && let Some(cte) = current.named_child(0).filter(|child| child.kind() == "cte")
            {
                return Some(cte);
            }
            node = current.parent();
        }
        None
    }
}

impl<'a> TryFrom<&'a QueryResult<'a>> for &'a DerivedRelationMatch<'a> {
    type Error = String;

    fn try_from(q: &'a QueryResult<'a>) -> Result<Self, Self::Error> {
        match q {
            QueryResult::DerivedRelation(r) => Ok(r),

            _ => Err("Invalid QueryResult type".into()),
        }
    }
}

impl<'a> QueryTryFrom<'a> for DerivedRelationMatch<'a> {
    type Ref = &'a DerivedRelationMatch<'a>;
}

impl<'a> Query<'a> for DerivedRelationMatch<'a> {
    fn execute(root_node: tree_sitter::Node<'a>, stmt: &'a str) -> Vec<QueryResult<'a>> {
        let mut cursor = tree_sitter::QueryCursor::new();

        let Some(query) = TS_QUERY.as_ref() else {
            return vec![];
        };
        let mut matches = cursor.matches(query, root_node, stmt.as_bytes());

        let item_index = query.capture_index_for_name("item");
        let name_index = query.capture_index_for_name("name");

        let mut to_return = vec![];

        while let Some(m) = matches.next() {
            let capture = |index| {
                m.captures
                    .iter()
                    .find(|capture| Some(capture.index) == index)
                    .map(|capture| capture.node)
            };
            if let (Some(item), Some(name)) = (capture(item_index), capture(name_index)) {
                to_return.push(QueryResult::DerivedRelation(DerivedRelationMatch {
                    item,
                    name,
                }));
            }
        }

        to_return
    }
}

#[cfg(test)]
mod tests {
    use crate::queries::TreeSitterQueriesExecutor;

    use super::DerivedRelationMatch;

    /// The name and the columns query of each derived relation in `sql`, with the cursor at
    /// `position`.
    fn derived_relations(sql: &str, position: usize) -> Vec<(String, Option<String>)> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .unwrap();

        let tree = parser.parse(sql, None).unwrap();

        let mut executor = TreeSitterQueriesExecutor::new(tree.root_node(), sql);
        executor.add_query_results::<DerivedRelationMatch>();

        executor
            .get_iter(None)
            .filter_map(|q| q.try_into().ok())
            .map(|m: &DerivedRelationMatch| (m.get_name(sql), m.columns_query(sql, position)))
            .collect()
    }

    #[test]
    fn finds_ctes_with_the_ctes_before_them() {
        let sql = r#"with a(x) as (select 1), "B" as (select x from a) select  from "B""#;
        assert_eq!(
            derived_relations(sql, sql.find("select  ").unwrap() + 7),
            [
                (
                    "a".into(),
                    Some("with a(x) as (select 1) select * from a".into())
                ),
                (
                    "B".into(),
                    Some(
                        r#"with a(x) as (select 1), "B" as (select x from a) select * from "B""#
                            .into()
                    )
                ),
            ]
        );
    }

    #[test]
    fn finds_subqueries_with_an_alias() {
        let sql = "with c as (select 1 a) select  from (select a from c) as s(b), (select 2)";
        assert_eq!(
            derived_relations(sql, sql.find("select  ").unwrap() + 7),
            [
                (
                    "c".into(),
                    Some("with c as (select 1 a) select * from c".into())
                ),
                (
                    "s".into(),
                    Some("with c as (select 1 a) select * from (select a from c) as s(b)".into())
                ),
            ]
        );
    }

    #[test]
    fn skips_the_query_under_the_cursor() {
        let sql = "with a as (select 1 x), b as (select ) select * from b";
        assert_eq!(
            derived_relations(sql, sql.find("select )").unwrap() + 7),
            [
                (
                    "a".into(),
                    Some("with a as (select 1 x) select * from a".into())
                ),
                ("b".into(), None),
            ]
        );
    }
}
