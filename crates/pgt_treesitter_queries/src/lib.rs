pub mod queries;

use std::slice::Iter;

use queries::{Query, QueryResult};

pub struct TreeSitterQueriesExecutor<'a> {
    root_node: tree_sitter::Node<'a>,
    stmt: &'a str,
    results: Vec<QueryResult<'a>>,
}

impl<'a> TreeSitterQueriesExecutor<'a> {
    pub fn new(root_node: tree_sitter::Node<'a>, stmt: &'a str) -> Self {
        Self {
            root_node,
            stmt,
            results: vec![],
        }
    }

    #[allow(private_bounds)]
    pub fn add_query_results<Q: Query<'a>>(&mut self) {
        let mut results = Q::execute(self.root_node, self.stmt);
        self.results.append(&mut results);
    }

    pub fn get_iter(&self, range: Option<&'a tree_sitter::Range>) -> QueryResultIter {
        match range {
            Some(r) => QueryResultIter::new(&self.results).within_range(r),
            None => QueryResultIter::new(&self.results),
        }
    }
}

pub struct QueryResultIter<'a> {
    inner: Iter<'a, QueryResult<'a>>,
    range: Option<&'a tree_sitter::Range>,
}

impl<'a> QueryResultIter<'a> {
    pub(crate) fn new(results: &'a Vec<QueryResult<'a>>) -> Self {
        Self {
            inner: results.iter(),
            range: None,
        }
    }

    fn within_range(mut self, r: &'a tree_sitter::Range) -> Self {
        self.range = Some(r);
        self
    }
}

impl<'a> Iterator for QueryResultIter<'a> {
    type Item = &'a QueryResult<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.inner.next()?;

        if self.range.as_ref().is_some_and(|r| !next.within_range(r)) {
            return self.next();
        }

        Some(next)
    }
}

#[cfg(test)]
mod tests {

    use crate::{TreeSitterQueriesExecutor, queries::RelationMatch};

    #[test]
    fn finds_all_relations_and_ignores_functions() {
        let sql = r#"
select
  *
from
  (
    select
      something
    from
      public.cool_table pu
      join private.cool_tableau pr on pu.id = pr.id
    where
      x = '123'
    union
    select
      something_else
    from
      another_table puat
      inner join private.another_tableau prat on puat.id = prat.id
    union
    select
      x,
      y
    from
      public.get_something_cool ()
  )
where
  col = 17;
"#;

        let mut parser = tree_sitter::Parser::new();
        parser.set_language(tree_sitter_sql::language()).unwrap();

        let tree = parser.parse(sql, None).unwrap();

        let mut executor = TreeSitterQueriesExecutor::new(tree.root_node(), sql);

        executor.add_query_results::<RelationMatch>();

        let results: Vec<&RelationMatch> = executor
            .get_iter(None)
            .filter_map(|q| q.try_into().ok())
            .collect();

        assert_eq!(results[0].get_schema(sql), Some("public".into()));
        assert_eq!(results[0].get_table(sql), "cool_table");

        assert_eq!(results[1].get_schema(sql), Some("private".into()));
        assert_eq!(results[1].get_table(sql), "cool_tableau");

        assert_eq!(results[2].get_schema(sql), None);
        assert_eq!(results[2].get_table(sql), "another_table");

        assert_eq!(results[3].get_schema(sql), Some("private".into()));
        assert_eq!(results[3].get_table(sql), "another_tableau");

        // we have exhausted the matches: function invocations are ignored.
        assert!(results.len() == 4);
    }

    #[test]
    fn only_considers_nodes_in_requested_range() {
        let sql = r#"
select
  *
from (
    select * 
    from (
      select *
      from private.something
    ) as sq2 
    join private.tableau pt1
    on sq2.id = pt1.id
  ) as sq1
join private.table pt
on sq1.id = pt.id;
"#;

        let mut parser = tree_sitter::Parser::new();
        parser.set_language(tree_sitter_sql::language()).unwrap();

        let tree = parser.parse(sql, None).unwrap();

        // trust me bro
        let range = {
            let mut cursor = tree.root_node().walk();
            cursor.goto_first_child(); // statement
            cursor.goto_first_child(); // select
            cursor.goto_next_sibling(); // from
            cursor.goto_first_child(); // keyword_from
            cursor.goto_next_sibling(); // relation
            cursor.goto_first_child(); // subquery (1)
            cursor.goto_first_child(); // "("
            cursor.goto_next_sibling(); // select
            cursor.goto_next_sibling(); // from
            cursor.goto_first_child(); // keyword_from
            cursor.goto_next_sibling(); // relation
            cursor.goto_first_child(); // subquery (2)
            cursor.node().range()
        };

        let mut executor = TreeSitterQueriesExecutor::new(tree.root_node(), sql);

        executor.add_query_results::<RelationMatch>();

        let results: Vec<&RelationMatch> = executor
            .get_iter(Some(&range))
            .filter_map(|q| q.try_into().ok())
            .collect();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get_schema(sql), Some("private".into()));
        assert_eq!(results[0].get_table(sql), "something");
    }
}
