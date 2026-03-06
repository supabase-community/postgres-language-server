#[derive(Debug)]
pub struct Scope {
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
pub struct ScopeTracker {
    scopes: Vec<Scope>,
}

impl ScopeTracker {
    pub fn new() -> Self {
        Self { scopes: vec![] }
    }

    pub fn register<'a>(&mut self, node: tree_sitter::Node<'a>, position: usize) {
        if SCOPE_BOUNDARIES.contains(&node.kind()) {
            self.add_new_scope(node);
        }

        self.scopes
            .last_mut()
            .unwrap_or_else(|| panic!("No top-level grammar-rule found. Please create an issue with the entire Postgres file, noting where you've hovered."))
            .ancestors
            .register(node, position);
    }

    fn add_new_scope(&mut self, _node: tree_sitter::Node<'_>) {
        self.scopes.push(Scope {
            ancestors: AncestorTracker::new(),
        })
    }

    pub fn current(&self) -> &Scope {
        self.scopes.last().unwrap()
    }
}

#[derive(Clone, Debug)]
struct AncestorNode {
    kind: String,
    field: Option<String>,
}

#[derive(Debug)]
pub(crate) struct AncestorTracker {
    ancestors: Vec<AncestorNode>,
    next_field: Option<String>,
}

impl AncestorTracker {
    pub fn new() -> Self {
        Self {
            ancestors: vec![],
            next_field: None,
        }
    }

    pub fn register<'a>(&mut self, node: tree_sitter::Node<'a>, position: usize) {
        let ancestor_node = AncestorNode {
            kind: node.kind().into(),
            field: self.next_field.take(),
        };

        if let Some(child_on_cursor) = node.first_child_for_byte(position) {
            let (idx, _) = node
                .children(&mut node.walk())
                .enumerate()
                .find(|(_, n)| n == &child_on_cursor)
                .expect("Node has to exist");

            self.next_field = node
                .field_name_for_child(idx.try_into().unwrap())
                .map(|f| f.to_string())
        }

        self.ancestors.push(ancestor_node);
    }

    pub fn is_within_one_of_fields(&self, field_names: &[&'static str]) -> bool {
        self.ancestors
            .iter()
            .any(|n| n.field.as_deref().is_some_and(|f| field_names.contains(&f)))
    }

    pub fn history_ends_with(&self, matchers: &[&'static str]) -> bool {
        assert!(!matchers.is_empty());

        let mut tracking_idx = matchers.len() - 1;

        for ancestor in self.ancestors.iter().rev() {
            if ancestor.kind != matchers[tracking_idx] {
                return false;
            }

            if tracking_idx >= 1 {
                tracking_idx -= 1;
            } else {
                break;
            }
        }

        true
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
