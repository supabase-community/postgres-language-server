#[derive(Debug)]
pub(super) struct Scope {
    pub ancestors: AncestorTracker,
}

static SCOPE_BOUNDARIES: &[&str] = &[
    "statement",
    "ERROR",
    "program",
    "block",
    "transaction",
    "psql_meta_command",
    "copy_data_stream",
];

#[derive(Debug)]
pub(super) struct ScopeTracker {
    /// The scope of the `program` root node, which is never registered, so it exists from the
    /// start. Children of `program` that aren't scope boundaries, such as a stray `;`, belong
    /// to it.
    root: Scope,
    /// The scopes opened since, innermost last.
    nested: Vec<Scope>,
}

impl ScopeTracker {
    pub(super) fn new() -> Self {
        Self {
            root: Scope {
                ancestors: AncestorTracker::new(),
            },
            nested: Vec::new(),
        }
    }

    pub(super) fn register<'a>(&mut self, node: tree_sitter::Node<'a>, position: usize) {
        if SCOPE_BOUNDARIES.contains(&node.kind()) {
            self.add_new_scope(node);
        }

        self.nested
            .last_mut()
            .unwrap_or(&mut self.root)
            .ancestors
            .register(node, position);
    }

    fn add_new_scope(&mut self, _node: tree_sitter::Node<'_>) {
        self.nested.push(Scope {
            ancestors: AncestorTracker::new(),
        })
    }

    pub(super) fn current(&self) -> &Scope {
        self.nested.last().unwrap_or(&self.root)
    }
}

#[derive(Clone, Debug)]
struct AncestorNode {
    kind: &'static str,
    field: Option<&'static str>,
}

#[derive(Debug)]
pub(crate) struct AncestorTracker {
    ancestors: Vec<AncestorNode>,
    next_field: Option<&'static str>,
}

impl AncestorTracker {
    pub(super) fn new() -> Self {
        Self {
            ancestors: vec![],
            next_field: None,
        }
    }

    pub(super) fn register<'a>(&mut self, node: tree_sitter::Node<'a>, position: usize) {
        let ancestor_node = AncestorNode {
            kind: node.kind(),
            field: self.next_field.take(),
        };

        if let Some(child_on_cursor) = node.first_child_for_byte(position) {
            // `first_child_for_byte` returns one of the children.
            self.next_field = (0..)
                .zip(node.children(&mut node.walk()))
                .find(|(_, n)| n == &child_on_cursor)
                .and_then(|(idx, _)| node.field_name_for_child(idx))
        }

        self.ancestors.push(ancestor_node);
    }

    pub(super) fn is_within_one_of_fields(&self, field_names: &[&'static str]) -> bool {
        self.ancestors
            .iter()
            .any(|n| n.field.is_some_and(|f| field_names.contains(&f)))
    }

    pub(super) fn history_ends_with(&self, matchers: &[&'static str]) -> bool {
        debug_assert!(!matchers.is_empty());

        self.ancestors
            .iter()
            .rev()
            .zip(matchers.iter().rev())
            .all(|(ancestor, matcher)| ancestor.kind == *matcher)
    }
}

#[cfg(test)]
mod tests {
    use crate::context::{TreeSitterContextParams, TreesitterContext};

    fn get_tree(input: &str) -> tree_sitter::Tree {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .expect("Couldn't set language");

        parser.parse(input, None).expect("Unable to parse tree")
    }

    fn assert_no_panic_for_all_positions(sql: &str) {
        let tree = get_tree(sql);
        for pos in 0..sql.len() {
            let params = TreeSitterContextParams {
                position: (pos as u32).into(),
                text: sql,
                tree: &tree,
                server_version_num: None,
            };
            let _ = TreesitterContext::new(params);
        }
    }

    #[test]
    fn scope_boundary_block() {
        assert_no_panic_for_all_positions("BEGIN; SELECT 1; END;");
    }

    #[test]
    fn scope_boundary_transaction() {
        assert_no_panic_for_all_positions("BEGIN TRANSACTION; SELECT 1; COMMIT;");
        assert_no_panic_for_all_positions("BEGIN; INSERT INTO t VALUES (1); ROLLBACK;");
    }

    #[test]
    fn scope_boundary_psql_meta_command() {
        assert_no_panic_for_all_positions("\\dt\n\\d users");
    }

    #[test]
    fn scope_boundary_copy_data_stream() {
        assert_no_panic_for_all_positions("COPY t FROM STDIN;\n1\tAlice\n\\.\n");
    }

    #[test]
    fn scope_boundary_comment() {
        assert_no_panic_for_all_positions("-- a comment\nSELECT 1;");
    }

    #[test]
    fn issue_769_regression() {
        assert_no_panic_for_all_positions("BEGIN END; ");
        assert_no_panic_for_all_positions(
            "create procedure p() language plpgsql as $$ begin null; end; $$; ",
        );
    }

    #[test]
    fn issue_704_regression() {
        let statements = vec![
            r#"
            CREATE OR REPLACE FUNCTION my_schema.my_function1(
                pi_1 character varying, 
                pi_2 character varying, 
                pi_3 jsonb, 
                OUT po_1 integer, 
                OUT po_2 integer, 
                OUT result integer
            )
            RETURNS record
            LANGUAGE plpgsql
            AS $function$
            "#
            .trim(),

            r#"
            CREATE OR REPLACE FUNCTION my_schema.my_function2(
                pi_1 character varying, 
                pi_2 character varying, 
                pi_3 jsonb, 
                OUT po_1 integer, 
                OUT po_2 integer, 
                OUT result integer
            )
            RETURNS record
            LANGUAGE plpgsql
            AS $function$
            DECLARE
            BEGIN
                -- Function logic goes here
                -- For example, you can perform some operations using the input parameters and set the output parameters accordingly

                -- Example logic (replace with actual implementation):
                po_1 := length(pi_1); -- Set po_1 to the length of pi_1
                po_2 := length(pi_2); -- Set po_2 to the length of pi_2
                result := po_1 + po_2; -- Set result to the sum of po_1 and po_2
            END;
            $function$;
            "#.trim(),
        ];

        for stmt in statements {
            assert_no_panic_for_all_positions(stmt);
        }
    }
}
