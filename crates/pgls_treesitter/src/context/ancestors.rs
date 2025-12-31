#[derive(Debug)]
pub struct Scope {
    pub ancestors: AncestorTracker,
}

static SCOPE_BOUNDARIES: &[&str] = &["statement", "ERROR", "program"];

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
            .unwrap_or_else(|| panic!("Unhandled node kind: {}", node.kind()))
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
