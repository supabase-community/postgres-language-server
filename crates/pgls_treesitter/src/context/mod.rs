use std::{
    cmp,
    collections::{HashMap, HashSet},
};

use crate::queries::{self, QueryResult, TreeSitterQueriesExecutor};
use pgls_text_size::TextSize;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum WrappingClause<'a> {
    Select,
    Where,
    From,
    Join {
        on_node: Option<tree_sitter::Node<'a>>,
    },
    Update,
    Delete,
    ColumnDefinitions,
    Insert,
    AlterTable,
    DropTable,
    DropColumn,
    AlterColumn,
    RenameColumn,
    SetStatement,
    AlterRole,
    DropRole,
    RevokeStatement,
    GrantStatement,

    CreatePolicy,
    AlterPolicy,
    DropPolicy,
    CheckOrUsingClause,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct MentionedColumn {
    pub column: String,
    pub alias: Option<String>,
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
    List,
}

impl TryFrom<&str> for WrappingNode {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "relation" => Ok(Self::Relation),
            "assignment" => Ok(Self::Assignment),
            "binary_expression" => Ok(Self::BinaryExpression),
            "list" => Ok(Self::List),
            _ => {
                let message = format!("Unimplemented Relation: {value}");

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

pub struct TreeSitterContextParams<'a> {
    pub position: TextSize,
    pub text: &'a str,
    pub tree: &'a tree_sitter::Tree,
}

#[derive(Debug)]
pub struct TreesitterContext<'a> {
    pub node_under_cursor: Option<tree_sitter::Node<'a>>,

    pub tree: &'a tree_sitter::Tree,
    pub text: &'a str,
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

    mentioned_relations: HashMap<Option<String>, HashSet<String>>,
    mentioned_table_aliases: HashMap<String, String>,
    mentioned_columns: HashMap<Option<WrappingClause<'a>>, HashSet<MentionedColumn>>,
}

impl<'a> TreesitterContext<'a> {
    pub fn new(params: TreeSitterContextParams<'a>) -> Self {
        let mut ctx = Self {
            tree: params.tree,
            text: params.text,
            position: usize::from(params.position),
            node_under_cursor: None,
            schema_or_alias_name: None,
            wrapping_clause_type: None,
            wrapping_node_kind: None,
            wrapping_statement_range: None,
            is_invocation: false,
            mentioned_relations: HashMap::new(),
            mentioned_table_aliases: HashMap::new(),
            mentioned_columns: HashMap::new(),
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
        executor.add_query_results::<queries::SelectColumnMatch>();
        executor.add_query_results::<queries::InsertColumnMatch>();
        executor.add_query_results::<queries::WhereColumnMatch>();

        for relation_match in executor.get_iter(stmt_range) {
            match relation_match {
                QueryResult::Relation(r) => {
                    let schema_name = r.get_schema(sql);
                    let table_name = r.get_table(sql);

                    self.mentioned_relations
                        .entry(schema_name)
                        .and_modify(|s| {
                            s.insert(table_name.clone());
                        })
                        .or_insert(HashSet::from([table_name]));
                }

                QueryResult::TableAliases(table_alias_match) => {
                    self.mentioned_table_aliases.insert(
                        table_alias_match.get_alias(sql),
                        table_alias_match.get_table(sql),
                    );
                }

                QueryResult::SelectClauseColumns(c) => {
                    let mentioned = MentionedColumn {
                        column: c.get_column(sql),
                        alias: c.get_alias(sql),
                    };

                    self.mentioned_columns
                        .entry(Some(WrappingClause::Select))
                        .and_modify(|s| {
                            s.insert(mentioned.clone());
                        })
                        .or_insert(HashSet::from([mentioned]));
                }

                QueryResult::WhereClauseColumns(c) => {
                    let mentioned = MentionedColumn {
                        column: c.get_column(sql),
                        alias: c.get_alias(sql),
                    };

                    self.mentioned_columns
                        .entry(Some(WrappingClause::Where))
                        .and_modify(|s| {
                            s.insert(mentioned.clone());
                        })
                        .or_insert(HashSet::from([mentioned]));
                }

                QueryResult::InsertClauseColumns(c) => {
                    let mentioned = MentionedColumn {
                        column: c.get_column(sql),
                        alias: None,
                    };

                    self.mentioned_columns
                        .entry(Some(WrappingClause::Insert))
                        .and_modify(|s| {
                            s.insert(mentioned.clone());
                        })
                        .or_insert(HashSet::from([mentioned]));
                }
                _ => {}
            };
        }
    }

    fn get_ts_node_content(&self, ts_node: &tree_sitter::Node<'a>) -> Option<String> {
        let source = self.text;
        ts_node
            .utf8_text(source.as_bytes())
            .ok()
            .map(|txt| txt.into())
    }

    pub fn get_node_under_cursor_content(&self) -> Option<String> {
        self.node_under_cursor
            .as_ref()
            .and_then(|node| self.get_ts_node_content(node))
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

        let mut chars = self.text.chars();

        if chars
            .nth(self.position)
            .is_some_and(|c| !c.is_ascii_whitespace() && !&[';', ')'].contains(&c))
        {
            self.position = cmp::min(self.position, self.text.len().saturating_sub(1));
        } else {
            self.position = cmp::min(
                self.position.saturating_sub(1),
                self.text.len().saturating_sub(1),
            );
        }

        cursor.goto_first_child_for_byte(self.position);

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

        // prevent infinite recursion – this can happen with ERROR nodes
        if current_node_kind == parent_node_kind && ["ERROR", "program"].contains(&parent_node_kind)
        {
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
        if parent_node_kind == "ERROR" {
            if let Some(clause_type) = self.get_wrapping_clause_from_error_node_child(current_node)
            {
                self.wrapping_clause_type = Some(clause_type);
            }
            if let Some(wrapping_node) = self.get_wrapping_node_from_error_node_child(current_node)
            {
                self.wrapping_node_kind = Some(wrapping_node)
            }

            self.get_info_from_error_node_child(current_node);
        }

        match current_node_kind {
            "object_reference" | "field" => {
                let start = current_node.start_byte();
                let content = self.get_ts_node_content(&current_node);
                if let Some(txt) = content {
                    let parts: Vec<&str> = txt.split('.').collect();
                    // we do not want to set it if we're on the schema or alias node itself
                    let is_on_schema_node = start + parts[0].len() >= self.position;
                    if parts.len() == 2 && !is_on_schema_node {
                        self.schema_or_alias_name = Some(parts[0].to_string());
                    }
                }
            }

            "relation" | "binary_expression" | "assignment" => {
                self.wrapping_node_kind = current_node_kind.try_into().ok();
            }

            "list" => {
                if current_node
                    .prev_sibling()
                    .is_none_or(|n| n.kind() != "keyword_values")
                {
                    self.wrapping_node_kind = current_node_kind.try_into().ok();
                }
            }

            _ => {
                if let Some(clause_type) =
                    self.get_wrapping_clause_from_current_node(current_node, &mut cursor)
                {
                    self.wrapping_clause_type = Some(clause_type);
                }
            }
        }

        // We have arrived at the leaf node
        if current_node.child_count() == 0
            || current_node.first_child_for_byte(self.position).is_none()
        {
            self.node_under_cursor = Some(current_node);
            return;
        }

        cursor.goto_first_child_for_byte(self.position);
        self.gather_context_from_node(cursor, current_node);
    }

    fn get_first_sibling(&self, node: tree_sitter::Node<'a>) -> tree_sitter::Node<'a> {
        let mut first_sibling = node;
        while let Some(n) = first_sibling.prev_sibling() {
            first_sibling = n;
        }
        first_sibling
    }

    fn get_wrapping_node_from_error_node_child(
        &self,
        node: tree_sitter::Node<'a>,
    ) -> Option<WrappingNode> {
        self.wrapping_clause_type
            .as_ref()
            .and_then(|clause| match clause {
                WrappingClause::Insert => {
                    let mut first_sib = self.get_first_sibling(node);

                    let mut after_opening_bracket = false;
                    let mut before_closing_bracket = false;

                    while let Some(next_sib) = first_sib.next_sibling() {
                        if next_sib.kind() == "("
                            && next_sib.end_position() <= node.start_position()
                        {
                            after_opening_bracket = true;
                        }

                        if next_sib.kind() == ")"
                            && next_sib.start_position() >= node.end_position()
                        {
                            before_closing_bracket = true;
                        }

                        first_sib = next_sib;
                    }

                    if after_opening_bracket && before_closing_bracket {
                        Some(WrappingNode::List)
                    } else {
                        None
                    }
                }
                _ => None,
            })
    }

    fn get_wrapping_clause_from_error_node_child(
        &self,
        node: tree_sitter::Node<'a>,
    ) -> Option<WrappingClause<'a>> {
        let clause_combinations: Vec<(WrappingClause, &[&'static str])> = vec![
            (WrappingClause::Where, &["where"]),
            (WrappingClause::Update, &["update"]),
            (WrappingClause::Select, &["select"]),
            (WrappingClause::Delete, &["delete"]),
            (WrappingClause::Insert, &["insert", "into"]),
            (WrappingClause::From, &["from"]),
            (WrappingClause::Join { on_node: None }, &["join"]),
            (WrappingClause::AlterTable, &["alter", "table"]),
            (WrappingClause::AlterColumn, &["alter", "table", "alter"]),
            (WrappingClause::RenameColumn, &["alter", "table", "rename"]),
            (
                WrappingClause::AlterTable,
                &["alter", "table", "if", "exists"],
            ),
            (WrappingClause::DropTable, &["drop", "table"]),
            (
                WrappingClause::DropTable,
                &["drop", "table", "if", "exists"],
            ),
        ];

        let first_sibling = self.get_first_sibling(node);

        /*
         * For each clause, we'll iterate from first_sibling to the next ones,
         * either until the end or until we land on the node under the cursor.
         * We'll score the `WrappingClause` by how many tokens it matches in order.
         */
        let mut clauses_with_score: Vec<(WrappingClause, usize)> = clause_combinations
            .into_iter()
            .map(|(clause, tokens)| {
                let mut idx = 0;

                let mut sibling = Some(first_sibling);
                while let Some(sib) = sibling {
                    if sib.end_byte() >= node.end_byte() || idx >= tokens.len() {
                        break;
                    }

                    if let Some(sibling_content) = self.get_ts_node_content(&sib) {
                        if sibling_content == tokens[idx] {
                            idx += 1;
                        }
                    } else {
                        break;
                    }

                    sibling = sib.next_sibling();
                }

                (clause, idx)
            })
            .collect();

        clauses_with_score.sort_by(|(_, score_a), (_, score_b)| score_b.cmp(score_a));
        clauses_with_score
            .iter()
            .find(|(_, score)| *score > 0)
            .map(|c| c.0.clone())
    }

    fn get_info_from_error_node_child(&mut self, node: tree_sitter::Node<'a>) {
        let mut first_sibling = self.get_first_sibling(node);

        if let Some(clause) = self.wrapping_clause_type.as_ref() {
            match *clause {
                WrappingClause::Insert => {
                    while let Some(sib) = first_sibling.next_sibling() {
                        match sib.kind() {
                            "object_reference" => {
                                if let Some(txt) = self.get_ts_node_content(&sib) {
                                    let mut iter = txt.split('.').rev();
                                    let table = iter.next().unwrap().to_string();
                                    let schema = iter.next().map(|s| s.to_string());
                                    self.mentioned_relations
                                        .entry(schema)
                                        .and_modify(|s| {
                                            s.insert(table.clone());
                                        })
                                        .or_insert(HashSet::from([table]));
                                }
                            }

                            "column" => {
                                if let Some(txt) = self.get_ts_node_content(&sib) {
                                    let entry = MentionedColumn {
                                        column: txt,
                                        alias: None,
                                    };

                                    self.mentioned_columns
                                        .entry(Some(WrappingClause::Insert))
                                        .and_modify(|s| {
                                            s.insert(entry.clone());
                                        })
                                        .or_insert(HashSet::from([entry]));
                                }
                            }

                            _ => {}
                        }

                        first_sibling = sib;
                    }
                }

                WrappingClause::AlterColumn => {
                    while let Some(sib) = first_sibling.next_sibling() {
                        if sib.kind() == "object_reference" {
                            if let Some(txt) = self.get_ts_node_content(&sib) {
                                let mut iter = txt.split('.').rev();
                                let table = iter.next().unwrap().to_string();
                                let schema = iter.next().map(|s| s.to_string());
                                self.mentioned_relations
                                    .entry(schema)
                                    .and_modify(|s| {
                                        s.insert(table.clone());
                                    })
                                    .or_insert(HashSet::from([table]));
                            }
                        }

                        first_sibling = sib;
                    }
                }

                _ => {}
            }
        }
    }

    fn get_wrapping_clause_from_current_node(
        &self,
        node: tree_sitter::Node<'a>,
        cursor: &mut tree_sitter::TreeCursor<'a>,
    ) -> Option<WrappingClause<'a>> {
        match node.kind() {
            "where" => Some(WrappingClause::Where),
            "update" => Some(WrappingClause::Update),
            "select" => Some(WrappingClause::Select),
            "delete" => Some(WrappingClause::Delete),
            "from" => Some(WrappingClause::From),
            "drop_table" => Some(WrappingClause::DropTable),
            "alter_role" => Some(WrappingClause::AlterRole),
            "drop_role" => Some(WrappingClause::DropRole),
            "drop_column" => Some(WrappingClause::DropColumn),
            "alter_column" => Some(WrappingClause::AlterColumn),
            "rename_column" => Some(WrappingClause::RenameColumn),
            "alter_table" => Some(WrappingClause::AlterTable),
            "set_statement" => Some(WrappingClause::SetStatement),
            "revoke_statement" => Some(WrappingClause::RevokeStatement),
            "grant_statement" => Some(WrappingClause::RevokeStatement),
            "column_definitions" => Some(WrappingClause::ColumnDefinitions),
            "create_policy" => Some(WrappingClause::CreatePolicy),
            "alter_policy" => Some(WrappingClause::AlterPolicy),
            "drop_policy" => Some(WrappingClause::DropPolicy),
            "check_or_using_clause" => Some(WrappingClause::CheckOrUsingClause),
            "insert" => Some(WrappingClause::Insert),
            "join" => {
                // sadly, we need to manually iterate over the children –
                // `node.child_by_field_id(..)` does not work as expected
                let mut on_node = None;
                for child in node.children(cursor) {
                    if child.kind() == "keyword_on" {
                        on_node = Some(child);
                    }
                }
                cursor.goto_parent();
                Some(WrappingClause::Join { on_node })
            }
            _ => None,
        }
    }

    pub fn before_cursor_matches_kind(&self, kinds: &[&'static str]) -> bool {
        self.node_under_cursor.as_ref().is_some_and(|node| {
            let mut current = *node;

            // move up to the parent until we're at top OR we have a prev sibling
            while current.prev_sibling().is_none() && current.parent().is_some() {
                current = current.parent().unwrap();
            }

            current
                .prev_sibling()
                .is_some_and(|sib| kinds.contains(&sib.kind()))
        })
    }

    /// Verifies whether the node_under_cursor has the passed in ancestors in the right order.
    /// Note that you need to pass in the ancestors in the order as they would appear in the tree:
    ///
    /// If the tree shows `relation > object_reference > any_identifier` and the "any_identifier" is a leaf node,
    /// you need to pass `&["relation", "object_reference"]`.
    pub fn matches_ancestor_history(&self, expected_ancestors: &[&'static str]) -> bool {
        self.node_under_cursor.as_ref().is_some_and(|node| {
            let mut current = Some(*node);

            for &expected_kind in expected_ancestors.iter().rev() {
                current = current.and_then(|n| n.parent());

                match current {
                    Some(ancestor) if ancestor.kind() == expected_kind => continue,
                    _ => return false,
                }
            }

            true
        })
    }

    /// Verifies whether the node_under_cursor has the passed in ancestors in the right order.
    /// Note that you need to pass in the ancestors in the order as they would appear in the tree:
    ///
    /// If the tree shows `relation > object_reference > any_identifier` and the "any_identifier" is a leaf node,
    /// you need to pass `&["relation", "object_reference"]`.
    pub fn matches_one_of_ancestors(&self, expected_ancestors: &[&'static str]) -> bool {
        self.node_under_cursor.as_ref().is_some_and(|node| {
            node.parent()
                .is_some_and(|p| expected_ancestors.contains(&p.kind()))
        })
    }

    /// Checks whether the Node under the cursor is the nth child of the parent.
    ///
    /// ```
    /// /*
    ///  * Given `select * from "a|uth"."users";`
    ///  * The node under the cursor is "auth".
    ///  *
    ///  * [...] redacted
    ///  * from [9..28] 'from "auth"."users"'
    ///  *   keyword_from [9..13] 'from'
    ///  *   relation [14..28] '"auth"."users"'
    ///  *     object_reference [14..28] '"auth"."users"'
    ///  *       any_identifier [14..20] '"auth"'
    ///  *         . [20..21] '.'
    ///  *       any_identifier [21..28] '"users"'
    ///  */
    ///
    /// if node_under_cursor_is_nth_child(1) {
    ///     node_type = "schema";
    /// } else if node_under_cursor_is_nth_child(3) {
    ///     node_type = "table";
    /// }
    /// ```
    pub fn node_under_cursor_is_nth_child(&self, nth: usize) -> bool {
        self.node_under_cursor.as_ref().is_some_and(|node| {
            let mut cursor = node.walk();
            node.parent().is_some_and(|p| {
                p.children(&mut cursor)
                    .nth(nth - 1)
                    .is_some_and(|n| n.id() == node.id())
            })
        })
    }

    /// Returns the number of siblings of the node under the cursor.
    pub fn num_siblings(&self) -> usize {
        self.node_under_cursor
            .as_ref()
            .map(|node| {
                // if there's no parent, we're on the top of the tree,
                // where we have 0 siblings.
                node.parent().map(|p| p.child_count() - 1).unwrap_or(0)
            })
            .unwrap_or(0)
    }

    /// Returns true if the node under the cursor matches the field_name OR has a parent that matches the field_name.
    pub fn node_under_cursor_is_within_field_name(&self, name: &str) -> bool {
        self.node_under_cursor
            .as_ref()
            .map(|node| {
                // It might seem weird that we have to check for the field_name from the parent,
                // but TreeSitter wants it this way, since nodes often can only be named in
                // the context of their parents.
                let root_node = self.tree.root_node();
                let mut cursor = node.walk();
                let mut parent = node.parent();

                while let Some(p) = parent {
                    if p == root_node {
                        break;
                    }

                    if p.children_by_field_name(name, &mut cursor).any(|c| {
                        let r = c.range();
                        // if the parent range contains the node range, the node is of the field_name.
                        r.start_byte <= node.start_byte() && r.end_byte >= node.end_byte()
                    }) {
                        return true;
                    } else {
                        parent = p.parent();
                    }
                }

                false
            })
            .unwrap_or(false)
    }

    pub fn get_mentioned_relations(&self, key: &Option<String>) -> Option<&HashSet<String>> {
        if let Some(key) = key.as_ref() {
            let sanitized_key = key.replace('"', "");

            self.mentioned_relations
                .get(&Some(sanitized_key.clone()))
                .or(self
                    .mentioned_relations
                    .get(&Some(format!(r#""{sanitized_key}""#))))
        } else {
            self.mentioned_relations.get(&None)
        }
    }

    pub fn get_mentioned_table_for_alias(&self, key: &str) -> Option<&String> {
        let sanitized_key = key.replace('"', "");

        self.mentioned_table_aliases.get(&sanitized_key).or(self
            .mentioned_table_aliases
            .get(&format!(r#""{sanitized_key}""#)))
    }

    pub fn get_used_alias_for_table(&self, table_name: &str) -> Option<String> {
        for (alias, table) in self.mentioned_table_aliases.iter() {
            if table == table_name {
                return Some(alias.to_string());
            }
        }
        None
    }

    pub fn get_mentioned_columns(
        &self,
        clause: &Option<WrappingClause<'a>>,
    ) -> Option<&HashSet<MentionedColumn>> {
        self.mentioned_columns.get(clause)
    }

    pub fn has_any_mentioned_relations(&self) -> bool {
        !self.mentioned_relations.is_empty()
    }

    pub fn has_mentioned_table_aliases(&self) -> bool {
        !self.mentioned_table_aliases.is_empty()
    }

    pub fn has_mentioned_columns(&self) -> bool {
        !self.mentioned_columns.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use crate::context::{TreeSitterContextParams, TreesitterContext, WrappingClause};

    use pgls_test_utils::QueryWithCursorPosition;

    fn get_tree(input: &str) -> tree_sitter::Tree {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .expect("Couldn't set language");

        parser.parse(input, None).expect("Unable to parse tree")
    }

    #[test]
    fn identifies_clauses() {
        let test_cases = vec![
            (
                format!(
                    "Select {}* from users;",
                    QueryWithCursorPosition::cursor_marker()
                ),
                WrappingClause::Select,
            ),
            (
                format!(
                    "Select * from u{};",
                    QueryWithCursorPosition::cursor_marker()
                ),
                WrappingClause::From,
            ),
            (
                format!(
                    "Select {}* from users where n = 1;",
                    QueryWithCursorPosition::cursor_marker()
                ),
                WrappingClause::Select,
            ),
            (
                format!(
                    "Select * from users where {}n = 1;",
                    QueryWithCursorPosition::cursor_marker()
                ),
                WrappingClause::Where,
            ),
            (
                format!(
                    "update users set u{} = 1 where n = 2;",
                    QueryWithCursorPosition::cursor_marker()
                ),
                WrappingClause::Update,
            ),
            (
                format!(
                    "update users set u = 1 where n{} = 2;",
                    QueryWithCursorPosition::cursor_marker()
                ),
                WrappingClause::Where,
            ),
            (
                format!(
                    "delete{} from users;",
                    QueryWithCursorPosition::cursor_marker()
                ),
                WrappingClause::Delete,
            ),
            (
                format!(
                    "delete from {}users;",
                    QueryWithCursorPosition::cursor_marker()
                ),
                WrappingClause::From,
            ),
            (
                format!(
                    "select name, age, location from public.u{}sers",
                    QueryWithCursorPosition::cursor_marker()
                ),
                WrappingClause::From,
            ),
        ];

        for (query, expected_clause) in test_cases {
            let (position, text) = QueryWithCursorPosition::from(query).get_text_and_position();

            let tree = get_tree(text.as_str());

            let params = TreeSitterContextParams {
                position: (position as u32).into(),
                text: &text,
                tree: &tree,
            };

            let ctx = TreesitterContext::new(params);

            assert_eq!(ctx.wrapping_clause_type, Some(expected_clause));
        }
    }

    #[test]
    fn identifies_schema() {
        let test_cases = vec![
            (
                format!(
                    "Select * from private.u{}",
                    QueryWithCursorPosition::cursor_marker()
                ),
                Some("private"),
            ),
            (
                format!(
                    "Select * from private.u{}sers()",
                    QueryWithCursorPosition::cursor_marker()
                ),
                Some("private"),
            ),
            (
                format!(
                    "Select * from u{}sers",
                    QueryWithCursorPosition::cursor_marker()
                ),
                None,
            ),
            (
                format!(
                    "Select * from u{}sers()",
                    QueryWithCursorPosition::cursor_marker()
                ),
                None,
            ),
        ];

        for (query, expected_schema) in test_cases {
            let (position, text) = QueryWithCursorPosition::from(query).get_text_and_position();

            let tree = get_tree(text.as_str());
            let params = TreeSitterContextParams {
                position: (position as u32).into(),
                text: &text,
                tree: &tree,
            };

            let ctx = TreesitterContext::new(params);

            assert_eq!(
                ctx.schema_or_alias_name,
                expected_schema.map(|f| f.to_string())
            );
        }
    }

    #[test]
    fn identifies_invocation() {
        let test_cases = vec![
            (
                format!(
                    "Select * from u{}sers",
                    QueryWithCursorPosition::cursor_marker()
                ),
                false,
            ),
            (
                format!(
                    "Select * from u{}sers()",
                    QueryWithCursorPosition::cursor_marker()
                ),
                true,
            ),
            (
                format!("Select cool{};", QueryWithCursorPosition::cursor_marker()),
                false,
            ),
            (
                format!("Select cool{}();", QueryWithCursorPosition::cursor_marker()),
                true,
            ),
            (
                format!(
                    "Select upp{}ercase as title from users;",
                    QueryWithCursorPosition::cursor_marker()
                ),
                false,
            ),
            (
                format!(
                    "Select upp{}ercase(name) as title from users;",
                    QueryWithCursorPosition::cursor_marker()
                ),
                true,
            ),
        ];

        for (query, is_invocation) in test_cases {
            let (position, text) = QueryWithCursorPosition::from(query).get_text_and_position();

            let tree = get_tree(text.as_str());
            let params = TreeSitterContextParams {
                position: (position as u32).into(),
                text: text.as_str(),
                tree: &tree,
            };

            let ctx = TreesitterContext::new(params);

            assert_eq!(ctx.is_invocation, is_invocation);
        }
    }

    #[test]
    fn does_not_fail_on_leading_whitespace() {
        let cases = vec![
            format!(
                "{}      select * from",
                QueryWithCursorPosition::cursor_marker()
            ),
            format!(
                " {}      select * from",
                QueryWithCursorPosition::cursor_marker()
            ),
        ];

        for query in cases {
            let (position, text) = QueryWithCursorPosition::from(query).get_text_and_position();

            let tree = get_tree(text.as_str());

            let params = TreeSitterContextParams {
                position: (position as u32).into(),
                text: &text,
                tree: &tree,
            };

            let ctx = TreesitterContext::new(params);

            let node = ctx.node_under_cursor.as_ref().unwrap();

            assert_eq!(ctx.get_ts_node_content(node), Some("select".into()));

            assert_eq!(
                ctx.wrapping_clause_type,
                Some(crate::context::WrappingClause::Select)
            );
        }
    }

    #[test]
    fn does_not_fail_on_trailing_whitespace() {
        let query = format!(
            "select * from   {}",
            QueryWithCursorPosition::cursor_marker()
        );

        let (position, text) = QueryWithCursorPosition::from(query).get_text_and_position();

        let tree = get_tree(text.as_str());

        let params = TreeSitterContextParams {
            position: (position as u32).into(),
            text: &text,
            tree: &tree,
        };

        let ctx = TreesitterContext::new(params);

        let node = ctx.node_under_cursor.as_ref().unwrap();

        assert_eq!(ctx.get_ts_node_content(node), Some("from".into()));
    }

    #[test]
    fn does_not_fail_with_empty_statements() {
        let query = format!("{}", QueryWithCursorPosition::cursor_marker());

        let (position, text) = QueryWithCursorPosition::from(query).get_text_and_position();

        let tree = get_tree(text.as_str());

        let params = TreeSitterContextParams {
            position: (position as u32).into(),
            text: &text,
            tree: &tree,
        };

        let ctx = TreesitterContext::new(params);

        let node = ctx.node_under_cursor.as_ref().unwrap();

        assert_eq!(ctx.get_ts_node_content(node), Some("".into()));
        assert_eq!(ctx.wrapping_clause_type, None);
    }

    #[test]
    fn does_not_fail_on_incomplete_keywords() {
        //  Instead of autocompleting "FROM", we'll assume that the user
        // is selecting a certain column name, such as `frozen_account`.
        let query = format!("select * fro{}", QueryWithCursorPosition::cursor_marker());

        let (position, text) = QueryWithCursorPosition::from(query).get_text_and_position();

        let tree = get_tree(text.as_str());

        let params = TreeSitterContextParams {
            position: (position as u32).into(),
            text: &text,
            tree: &tree,
        };

        let ctx = TreesitterContext::new(params);

        let node = ctx.node_under_cursor.as_ref().unwrap();

        assert_eq!(ctx.get_ts_node_content(node), Some("fro".into()));
        assert_eq!(ctx.wrapping_clause_type, Some(WrappingClause::Select));
    }

    #[test]
    fn verifies_node_has_field_name() {
        let query = format!(
            r#"create table foo (id int not null, compfoo som{}e_type);"#,
            QueryWithCursorPosition::cursor_marker()
        );
        let (position, text) = QueryWithCursorPosition::from(query).get_text_and_position();

        let tree = get_tree(text.as_str());

        let params = TreeSitterContextParams {
            position: (position as u32).into(),
            text: &text,
            tree: &tree,
        };

        let ctx = TreesitterContext::new(params);

        assert!(ctx.node_under_cursor_is_within_field_name("custom_type"));
    }

    #[test]
    fn does_not_overflow_callstack_on_smaller_treesitter_child() {
        let query = format!(
            r#"select * from persons where id = @i{}d;"#,
            QueryWithCursorPosition::cursor_marker()
        );

        /*
            The query (currently) yields the following treesitter tree for the WHERE clause:

            where [29..43] 'where id = @id'
                keyword_where [29..34] 'where'
                binary_expression [35..43] 'id = @id'
                field [35..37] 'id'
                    any_identifier [35..37] 'id'
                = [38..39] '='
                field [40..43] '@id'
                    any_identifier [40..43] '@id'
                        @ [40..41] '@'

            You can see that the '@' is a child of the "any_identifier" but has a range smaller than its parent's.
            This would crash our context parsing because, at position 42, we weren't at the leaf node but also couldn't
            go to a child on that position.
        */

        let (position, text) = QueryWithCursorPosition::from(query).get_text_and_position();

        let tree = get_tree(text.as_str());

        let params = TreeSitterContextParams {
            position: (position as u32).into(),
            text: &text,
            tree: &tree,
        };

        // should simply not panic
        let _ = TreesitterContext::new(params);
    }
}
