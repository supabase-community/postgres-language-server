use std::collections::{HashMap, HashSet};

use pgt_schema_cache::SchemaCache;
use pgt_treesitter_queries::{
    TreeSitterQueriesExecutor,
    queries::{self, QueryResult},
};

use crate::sanitization::SanitizedCompletionParams;

#[derive(Debug, PartialEq, Eq)]
pub enum WrappingClause<'a> {
    Select,
    Where,
    From,
    Join {
        on_node: Option<tree_sitter::Node<'a>>,
    },
    Update,
    Delete,
}

#[derive(PartialEq, Eq, Debug)]
pub(crate) enum NodeText<'a> {
    Replaced,
    Original(&'a str),
}

/// We can map a few nodes, such as the "update" node, to actual SQL clauses.
/// That gives us a lot of insight for completions.
/// Other nodes, such as the "relation" node, gives us less but still
/// relevant information.
/// `WrappingNode` maps to such nodes.
///
/// Note: This is not the direct parent of the `node_under_cursor`, but the closest
/// *relevant* parent.
#[derive(Debug, PartialEq, Eq)]
pub enum WrappingNode {
    Relation,
    BinaryExpression,
    Assignment,
}

impl TryFrom<&str> for WrappingNode {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "relation" => Ok(Self::Relation),
            "assignment" => Ok(Self::Assignment),
            "binary_expression" => Ok(Self::BinaryExpression),
            _ => {
                let message = format!("Unimplemented Relation: {}", value);

                // Err on tests, so we notice that we're lacking an implementation immediately.
                if cfg!(test) {
                    panic!("{}", message);
                }

                Err(message)
            }
        }
    }
}

impl TryFrom<String> for WrappingNode {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

pub(crate) struct CompletionContext<'a> {
    pub node_under_cursor: Option<tree_sitter::Node<'a>>,

    pub tree: &'a tree_sitter::Tree,
    pub text: &'a str,
    pub schema_cache: &'a SchemaCache,
    pub position: usize,

    /// If the cursor is on a node that uses dot notation
    /// to specify an alias or schema, this will hold the schema's or
    /// alias's name.
    ///
    /// Here, `auth` is a schema name:
    /// ```sql
    /// select * from auth.users;
    /// ```
    ///
    /// Here, `u` is an alias name:
    /// ```sql
    /// select
    ///     *
    /// from
    ///     auth.users u
    ///     left join identities i
    ///     on u.id = i.user_id;
    /// ```
    pub schema_or_alias_name: Option<String>,
    pub wrapping_clause_type: Option<WrappingClause<'a>>,

    pub wrapping_node_kind: Option<WrappingNode>,

    pub is_invocation: bool,
    pub wrapping_statement_range: Option<tree_sitter::Range>,

    /// Some incomplete statements can't be correctly parsed by TreeSitter.
    pub is_in_error_node: bool,

    pub mentioned_relations: HashMap<Option<String>, HashSet<String>>,

    pub mentioned_table_aliases: HashMap<String, String>,
}

impl<'a> CompletionContext<'a> {
    pub fn new(params: &'a SanitizedCompletionParams) -> Self {
        let mut ctx = Self {
            tree: params.tree.as_ref(),
            text: &params.text,
            schema_cache: params.schema,
            position: usize::from(params.position),
            node_under_cursor: None,
            schema_or_alias_name: None,
            wrapping_clause_type: None,
            wrapping_node_kind: None,
            wrapping_statement_range: None,
            is_invocation: false,
            mentioned_relations: HashMap::new(),
            mentioned_table_aliases: HashMap::new(),
            is_in_error_node: false,
        };

        ctx.gather_tree_context();
        ctx.gather_info_from_ts_queries();

        ctx
    }

    fn gather_info_from_ts_queries(&mut self) {
        let stmt_range = self.wrapping_statement_range.as_ref();
        let sql = self.text;

        let mut executor = TreeSitterQueriesExecutor::new(self.tree.root_node(), sql);

        executor.add_query_results::<queries::RelationMatch>();
        executor.add_query_results::<queries::TableAliasMatch>();

        for relation_match in executor.get_iter(stmt_range) {
            match relation_match {
                QueryResult::Relation(r) => {
                    let schema_name = r.get_schema(sql);
                    let table_name = r.get_table(sql);

                    let current = self.mentioned_relations.get_mut(&schema_name);

                    match current {
                        Some(c) => {
                            c.insert(table_name);
                        }
                        None => {
                            let mut new = HashSet::new();
                            new.insert(table_name);
                            self.mentioned_relations.insert(schema_name, new);
                        }
                    };
                }

                QueryResult::TableAliases(table_alias_match) => {
                    self.mentioned_table_aliases.insert(
                        table_alias_match.get_alias(sql),
                        table_alias_match.get_table(sql),
                    );
                }
            };
        }
    }

    pub fn get_ts_node_content(&self, ts_node: tree_sitter::Node<'a>) -> Option<NodeText<'a>> {
        let source = self.text;
        ts_node.utf8_text(source.as_bytes()).ok().map(|txt| {
            if SanitizedCompletionParams::is_sanitized_token(txt) {
                NodeText::Replaced
            } else {
                NodeText::Original(txt)
            }
        })
    }

    pub fn get_node_under_cursor_content(&self) -> Option<String> {
        self.node_under_cursor
            .and_then(|n| self.get_ts_node_content(n))
            .and_then(|txt| match txt {
                NodeText::Replaced => None,
                NodeText::Original(c) => Some(c.to_string()),
            })
    }

    fn gather_tree_context(&mut self) {
        let mut cursor = self.tree.root_node().walk();

        /*
         * The head node of any treesitter tree is always the "PROGRAM" node.
         *
         * We want to enter the next layer and focus on the child node that matches the user's cursor position.
         * If there is no node under the users position, however, the cursor won't enter the next level – it
         * will stay on the Program node.
         *
         * This might lead to an unexpected context or infinite recursion.
         *
         * We'll therefore adjust the cursor position such that it meets the last node of the AST.
         * `select * from use           {}` becomes `select * from use{}`.
         */
        let current_node = cursor.node();
        while cursor.goto_first_child_for_byte(self.position).is_none() && self.position > 0 {
            self.position -= 1;
        }

        self.gather_context_from_node(cursor, current_node);
    }

    fn gather_context_from_node(
        &mut self,
        mut cursor: tree_sitter::TreeCursor<'a>,
        parent_node: tree_sitter::Node<'a>,
    ) {
        let current_node = cursor.node();

        let parent_node_kind = parent_node.kind();
        let current_node_kind = current_node.kind();

        // prevent infinite recursion – this can happen if we only have a PROGRAM node
        if current_node_kind == parent_node_kind {
            self.node_under_cursor = Some(current_node);
            return;
        }

        match parent_node_kind {
            "statement" | "subquery" => {
                self.wrapping_clause_type =
                    self.get_wrapping_clause_from_current_node(current_node, &mut cursor);

                self.wrapping_statement_range = Some(parent_node.range());
            }
            "invocation" => self.is_invocation = true,
            _ => {}
        }

        // try to gather context from the siblings if we're within an error node.
        if self.is_in_error_node {
            let mut next_sibling = current_node.next_named_sibling();
            while let Some(n) = next_sibling {
                if let Some(clause_type) = self.get_wrapping_clause_from_keyword_node(n) {
                    self.wrapping_clause_type = Some(clause_type);
                    break;
                } else {
                    next_sibling = n.next_named_sibling();
                }
            }
            let mut prev_sibling = current_node.prev_named_sibling();
            while let Some(n) = prev_sibling {
                if let Some(clause_type) = self.get_wrapping_clause_from_keyword_node(n) {
                    self.wrapping_clause_type = Some(clause_type);
                    break;
                } else {
                    prev_sibling = n.prev_named_sibling();
                }
            }
        }

        match current_node_kind {
            "object_reference" | "field" => {
                let content = self.get_ts_node_content(current_node);
                if let Some(node_txt) = content {
                    match node_txt {
                        NodeText::Original(txt) => {
                            let parts: Vec<&str> = txt.split('.').collect();
                            if parts.len() == 2 {
                                self.schema_or_alias_name = Some(parts[0].to_string());
                            }
                        }
                        NodeText::Replaced => {}
                    }
                }
            }

            "where" | "update" | "select" | "delete" | "from" | "join" => {
                self.wrapping_clause_type =
                    self.get_wrapping_clause_from_current_node(current_node, &mut cursor);
            }

            "relation" | "binary_expression" | "assignment" => {
                self.wrapping_node_kind = current_node_kind.try_into().ok();
            }

            "ERROR" => {
                self.is_in_error_node = true;
            }

            _ => {}
        }

        // We have arrived at the leaf node
        if current_node.child_count() == 0 {
            self.node_under_cursor = Some(current_node);
            return;
        }

        cursor.goto_first_child_for_byte(self.position);
        self.gather_context_from_node(cursor, current_node);
    }

    fn get_wrapping_clause_from_keyword_node(
        &self,
        node: tree_sitter::Node<'a>,
    ) -> Option<WrappingClause<'a>> {
        if node.kind().starts_with("keyword_") {
            if let Some(txt) = self.get_ts_node_content(node).and_then(|txt| match txt {
                NodeText::Original(txt) => Some(txt),
                NodeText::Replaced => None,
            }) {
                match txt {
                    "where" => return Some(WrappingClause::Where),
                    "update" => return Some(WrappingClause::Update),
                    "select" => return Some(WrappingClause::Select),
                    "delete" => return Some(WrappingClause::Delete),
                    "from" => return Some(WrappingClause::From),
                    "join" => {
                        // TODO: not sure if we can infer it here.
                        return Some(WrappingClause::Join { on_node: None });
                    }
                    _ => {}
                }
            };
        }

        None
    }

    fn get_wrapping_clause_from_current_node(
        &self,
        node: tree_sitter::Node<'a>,
        cursor: &mut tree_sitter::TreeCursor<'a>,
    ) -> Option<WrappingClause<'a>> {
        return match node.kind() {
            "where" => Some(WrappingClause::Where),
            "update" => Some(WrappingClause::Update),
            "select" => Some(WrappingClause::Select),
            "delete" => Some(WrappingClause::Delete),
            "from" => Some(WrappingClause::From),
            "join" => {
                // sadly, we need to manually iterate over the children –
                // `node.child_by_field_id(..)` does not work as expected
                let mut on_node = None;
                for child in node.children(cursor) {
                    // 28 is the id for "keyword_on"
                    if child.kind_id() == 28 {
                        on_node = Some(child);
                    }
                }
                cursor.goto_parent();
                Some(WrappingClause::Join { on_node })
            }
            _ => None,
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        context::{CompletionContext, NodeText, WrappingClause},
        sanitization::SanitizedCompletionParams,
        test_helper::{CURSOR_POS, get_text_and_position},
    };

    fn get_tree(input: &str) -> tree_sitter::Tree {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(tree_sitter_sql::language())
            .expect("Couldn't set language");

        parser.parse(input, None).expect("Unable to parse tree")
    }

    #[test]
    fn identifies_clauses() {
        let test_cases = vec![
            (
                format!("Select {}* from users;", CURSOR_POS),
                WrappingClause::Select,
            ),
            (
                format!("Select * from u{};", CURSOR_POS),
                WrappingClause::From,
            ),
            (
                format!("Select {}* from users where n = 1;", CURSOR_POS),
                WrappingClause::Select,
            ),
            (
                format!("Select * from users where {}n = 1;", CURSOR_POS),
                WrappingClause::Where,
            ),
            (
                format!("update users set u{} = 1 where n = 2;", CURSOR_POS),
                WrappingClause::Update,
            ),
            (
                format!("update users set u = 1 where n{} = 2;", CURSOR_POS),
                WrappingClause::Where,
            ),
            (
                format!("delete{} from users;", CURSOR_POS),
                WrappingClause::Delete,
            ),
            (
                format!("delete from {}users;", CURSOR_POS),
                WrappingClause::From,
            ),
            (
                format!("select name, age, location from public.u{}sers", CURSOR_POS),
                WrappingClause::From,
            ),
        ];

        for (query, expected_clause) in test_cases {
            let (position, text) = get_text_and_position(query.as_str().into());

            let tree = get_tree(text.as_str());

            let params = SanitizedCompletionParams {
                position: (position as u32).into(),
                text,
                tree: std::borrow::Cow::Owned(tree),
                schema: &pgt_schema_cache::SchemaCache::default(),
            };

            let ctx = CompletionContext::new(&params);

            assert_eq!(ctx.wrapping_clause_type, Some(expected_clause));
        }
    }

    #[test]
    fn identifies_schema() {
        let test_cases = vec![
            (
                format!("Select * from private.u{}", CURSOR_POS),
                Some("private"),
            ),
            (
                format!("Select * from private.u{}sers()", CURSOR_POS),
                Some("private"),
            ),
            (format!("Select * from u{}sers", CURSOR_POS), None),
            (format!("Select * from u{}sers()", CURSOR_POS), None),
        ];

        for (query, expected_schema) in test_cases {
            let (position, text) = get_text_and_position(query.as_str().into());

            let tree = get_tree(text.as_str());
            let params = SanitizedCompletionParams {
                position: (position as u32).into(),
                text,
                tree: std::borrow::Cow::Owned(tree),
                schema: &pgt_schema_cache::SchemaCache::default(),
            };

            let ctx = CompletionContext::new(&params);

            assert_eq!(
                ctx.schema_or_alias_name,
                expected_schema.map(|f| f.to_string())
            );
        }
    }

    #[test]
    fn identifies_invocation() {
        let test_cases = vec![
            (format!("Select * from u{}sers", CURSOR_POS), false),
            (format!("Select * from u{}sers()", CURSOR_POS), true),
            (format!("Select cool{};", CURSOR_POS), false),
            (format!("Select cool{}();", CURSOR_POS), true),
            (
                format!("Select upp{}ercase as title from users;", CURSOR_POS),
                false,
            ),
            (
                format!("Select upp{}ercase(name) as title from users;", CURSOR_POS),
                true,
            ),
        ];

        for (query, is_invocation) in test_cases {
            let (position, text) = get_text_and_position(query.as_str().into());

            let tree = get_tree(text.as_str());
            let params = SanitizedCompletionParams {
                position: (position as u32).into(),
                text,
                tree: std::borrow::Cow::Owned(tree),
                schema: &pgt_schema_cache::SchemaCache::default(),
            };

            let ctx = CompletionContext::new(&params);

            assert_eq!(ctx.is_invocation, is_invocation);
        }
    }

    #[test]
    fn does_not_fail_on_leading_whitespace() {
        let cases = vec![
            format!("{}      select * from", CURSOR_POS),
            format!(" {}      select * from", CURSOR_POS),
        ];

        for query in cases {
            let (position, text) = get_text_and_position(query.as_str().into());

            let tree = get_tree(text.as_str());

            let params = SanitizedCompletionParams {
                position: (position as u32).into(),
                text,
                tree: std::borrow::Cow::Owned(tree),
                schema: &pgt_schema_cache::SchemaCache::default(),
            };

            let ctx = CompletionContext::new(&params);

            let node = ctx.node_under_cursor.unwrap();

            assert_eq!(
                ctx.get_ts_node_content(node),
                Some(NodeText::Original("select"))
            );

            assert_eq!(
                ctx.wrapping_clause_type,
                Some(crate::context::WrappingClause::Select)
            );
        }
    }

    #[test]
    fn does_not_fail_on_trailing_whitespace() {
        let query = format!("select * from   {}", CURSOR_POS);

        let (position, text) = get_text_and_position(query.as_str().into());

        let tree = get_tree(text.as_str());

        let params = SanitizedCompletionParams {
            position: (position as u32).into(),
            text,
            tree: std::borrow::Cow::Owned(tree),
            schema: &pgt_schema_cache::SchemaCache::default(),
        };

        let ctx = CompletionContext::new(&params);

        let node = ctx.node_under_cursor.unwrap();

        assert_eq!(
            ctx.get_ts_node_content(node),
            Some(NodeText::Original("from"))
        );
    }

    #[test]
    fn does_not_fail_with_empty_statements() {
        let query = format!("{}", CURSOR_POS);

        let (position, text) = get_text_and_position(query.as_str().into());

        let tree = get_tree(text.as_str());

        let params = SanitizedCompletionParams {
            position: (position as u32).into(),
            text,
            tree: std::borrow::Cow::Owned(tree),
            schema: &pgt_schema_cache::SchemaCache::default(),
        };

        let ctx = CompletionContext::new(&params);

        let node = ctx.node_under_cursor.unwrap();

        assert_eq!(ctx.get_ts_node_content(node), Some(NodeText::Original("")));
        assert_eq!(ctx.wrapping_clause_type, None);
    }

    #[test]
    fn does_not_fail_on_incomplete_keywords() {
        //  Instead of autocompleting "FROM", we'll assume that the user
        // is selecting a certain column name, such as `frozen_account`.
        let query = format!("select * fro{}", CURSOR_POS);

        let (position, text) = get_text_and_position(query.as_str().into());

        let tree = get_tree(text.as_str());

        let params = SanitizedCompletionParams {
            position: (position as u32).into(),
            text,
            tree: std::borrow::Cow::Owned(tree),
            schema: &pgt_schema_cache::SchemaCache::default(),
        };

        let ctx = CompletionContext::new(&params);

        let node = ctx.node_under_cursor.unwrap();

        assert_eq!(
            ctx.get_ts_node_content(node),
            Some(NodeText::Original("fro"))
        );
        assert_eq!(ctx.wrapping_clause_type, Some(WrappingClause::Select));
    }
}
