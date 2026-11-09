use std::sync::LazyLock;

use crate::queries::{Query, QueryResult, object_references::parts_of_reference_query};
use tree_sitter::StreamingIterator;

use super::QueryTryFrom;

static TS_QUERY: LazyLock<tree_sitter::Query> = LazyLock::new(|| {
    static QUERY_STR: &str = r#"
    [
        (statement (insert (table_reference) @target) (returning) @returning)
        (statement (update (update_target (table_reference) @target)) (returning) @returning)
        (delete_statement (from (table_reference) @target) (returning) @returning)
        (statement (merge (keyword_into) . (table_reference) @target) (returning) @returning)
    ]
"#;
    tree_sitter::Query::new(&pgls_treesitter_grammar::LANGUAGE.into(), QUERY_STR)
        .expect("Invalid TS Query")
});

/// The target table of an INSERT, UPDATE, DELETE, or MERGE with a RETURNING clause. Since
/// Postgres 18, `old` and `new` in the RETURNING list are the target row before and after the
/// change, and `RETURNING WITH (OLD AS o, NEW AS n)` renames them.
#[derive(Debug)]
pub struct ReturningAliasMatch<'a> {
    pub(crate) table: tree_sitter::Node<'a>,
    pub(crate) returning: tree_sitter::Node<'a>,
}

/// A name of the old or new row in RETURNING.
#[derive(Debug, PartialEq, Eq)]
pub struct ReturningAlias {
    /// The alias as written, like `"O"`.
    pub text: String,
    /// Whether `WITH (...)` gives the name. Postgres only adds the default names `old` and `new`
    /// if no FROM item has that name.
    pub explicit: bool,
}

impl ReturningAliasMatch<'_> {
    pub fn get_table(&self, sql: &str) -> Option<String> {
        self.table
            .utf8_text(sql.as_bytes())
            .ok()
            .map(str::to_string)
    }

    /// The names of the old and the new row.
    pub fn get_aliases(&self, sql: &str) -> Vec<ReturningAlias> {
        let mut explicit_old = None;
        let mut explicit_new = None;

        let mut cursor = self.returning.walk();
        let with = self
            .returning
            .children(&mut cursor)
            .find(|child| child.kind() == "returning_with");
        if let Some(with) = with {
            let mut cursor = with.walk();
            for item in with
                .children(&mut cursor)
                .filter(|child| child.kind() == "returning_with_item")
            {
                let mut item_cursor = item.walk();
                let mut kind = None;
                let mut name = None;
                for child in item.children(&mut item_cursor) {
                    match child.kind() {
                        "keyword_old" | "keyword_new" => kind = Some(child.kind()),
                        "any_identifier" => name = child.utf8_text(sql.as_bytes()).ok(),
                        _ => {}
                    }
                }
                match (kind, name) {
                    (Some("keyword_old"), Some(name)) => explicit_old = Some(name),
                    (Some("keyword_new"), Some(name)) => explicit_new = Some(name),
                    _ => {}
                }
            }
        }

        [(explicit_old, "old"), (explicit_new, "new")]
            .into_iter()
            .map(|(explicit, default)| ReturningAlias {
                text: explicit.unwrap_or(default).to_string(),
                explicit: explicit.is_some(),
            })
            .collect()
    }
}

impl<'a> TryFrom<&'a QueryResult<'a>> for &'a ReturningAliasMatch<'a> {
    type Error = String;

    fn try_from(q: &'a QueryResult<'a>) -> Result<Self, Self::Error> {
        match q {
            QueryResult::ReturningAliases(r) => Ok(r),

            #[allow(unreachable_patterns)]
            _ => Err("Invalid QueryResult type".into()),
        }
    }
}

impl<'a> QueryTryFrom<'a> for ReturningAliasMatch<'a> {
    type Ref = &'a ReturningAliasMatch<'a>;
}

impl<'a> Query<'a> for ReturningAliasMatch<'a> {
    fn execute(root_node: tree_sitter::Node<'a>, stmt: &'a str) -> Vec<QueryResult<'a>> {
        let mut cursor = tree_sitter::QueryCursor::new();

        let mut matches = cursor.matches(&TS_QUERY, root_node, stmt.as_bytes());

        let target_index = TS_QUERY.capture_index_for_name("target");
        let returning_index = TS_QUERY.capture_index_for_name("returning");

        let mut to_return = vec![];

        while let Some(m) = matches.next() {
            let capture = |index| {
                m.captures
                    .iter()
                    .find(|capture| Some(capture.index) == index)
                    .map(|capture| capture.node)
            };
            let (Some(target), Some(returning)) = (capture(target_index), capture(returning_index))
            else {
                continue;
            };
            if let Some((_, _, table)) = parts_of_reference_query(target, stmt) {
                to_return.push(QueryResult::ReturningAliases(ReturningAliasMatch {
                    table,
                    returning,
                }));
            }
        }

        to_return
    }
}

#[cfg(test)]
mod tests {
    use crate::queries::TreeSitterQueriesExecutor;

    use super::{ReturningAlias, ReturningAliasMatch};

    /// The target table and the names of the old and new row of each RETURNING clause in `sql`.
    fn returning_aliases(sql: &str) -> Vec<(String, Vec<ReturningAlias>)> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .unwrap();

        let tree = parser.parse(sql, None).unwrap();

        let mut executor = TreeSitterQueriesExecutor::new(tree.root_node(), sql);
        executor.add_query_results::<ReturningAliasMatch>();

        executor
            .get_iter(None)
            .filter_map(|q| q.try_into().ok())
            .map(|m: &ReturningAliasMatch| (m.get_table(sql).unwrap(), m.get_aliases(sql)))
            .collect()
    }

    fn alias(text: &str, explicit: bool) -> ReturningAlias {
        ReturningAlias {
            text: text.into(),
            explicit,
        }
    }

    #[test]
    fn finds_the_target_of_each_statement() {
        let defaults = || vec![alias("old", false), alias("new", false)];
        for sql in [
            "insert into public.users (id) values (1) returning new.id",
            "update users set name = 'x' returning old.name",
            "delete from users returning old.id",
            "merge into users u using posts p on u.id = p.user_id when matched then delete returning old.id",
            "with d as (delete from users returning old.id) select * from d",
        ] {
            assert_eq!(
                returning_aliases(sql),
                [("users".into(), defaults())],
                "{sql}"
            );
        }
    }

    #[test]
    fn finds_the_names_given_in_with() {
        assert_eq!(
            returning_aliases(
                r#"update users set name = 'x' returning with (old as "O") "O".name"#
            ),
            [(
                "users".into(),
                vec![alias(r#""O""#, true), alias("new", false)]
            )]
        );
        assert_eq!(
            returning_aliases("delete from users returning with (new as n, old as o) o.id"),
            [("users".into(), vec![alias("o", true), alias("n", true)])]
        );
    }

    #[test]
    fn ignores_statements_without_returning() {
        assert!(returning_aliases("update users set name = 'x'").is_empty());
    }
}
