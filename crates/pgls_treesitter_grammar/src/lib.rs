//! This crate provides PgLS language support for the [tree-sitter] parsing library.
//!
//! Typically, you will use the [`LANGUAGE`] constant to add this language to a
//! tree-sitter [`Parser`], and then use the parser to parse some code:
//!
//! ```
//! let code = r#"
//! "#;
//! let mut parser = tree_sitter::Parser::new();
//! let language = pgls_treesitter_grammar::LANGUAGE;
//! parser
//!     .set_language(&language.into())
//!     .expect("Error loading Postgres Language Server parser");
//! let tree = parser.parse(code, None).unwrap();
//! assert!(!tree.root_node().has_error());
//! ```
//!
//! [`Parser`]: https://docs.rs/tree-sitter/0.25.5/tree_sitter/struct.Parser.html
//! [tree-sitter]: https://tree-sitter.github.io/

use tree_sitter_language::LanguageFn;

unsafe extern "C" {
    unsafe fn tree_sitter_pgls() -> *const ();
}

/// The tree-sitter [`LanguageFn`] for this grammar.
pub const LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_pgls) };

/// The content of the generated [`node-types.json`] file for this grammar.
///
/// [`node-types.json`]: https://tree-sitter.github.io/tree-sitter/using-parsers/6-static-node-types
pub const NODE_TYPES: &str = include_str!(concat!(env!("OUT_DIR"), "/generated/node-types.json"));

// NOTE: uncomment these to include any queries that this grammar contains:

// pub const HIGHLIGHTS_QUERY: &str = include_str!("../../queries/highlights.scm");
// pub const INJECTIONS_QUERY: &str = include_str!("../../queries/injections.scm");
// pub const LOCALS_QUERY: &str = include_str!("../../queries/locals.scm");
// pub const TAGS_QUERY: &str = include_str!("../../queries/tags.scm");

/// A node kind of the grammar: the id that [`tree_sitter::Node::kind_id`] returns. `build.rs`
/// generates a constant for every kind a node can have from `parser.c`, so a kind the grammar
/// doesn't have is a compile error, and comparing kinds compares two integers. Anonymous kinds,
/// like `(`, are named after their C identifier: `Kind::ANON_LPAREN`.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Kind(u16);

impl Kind {
    /// The kind of an `ERROR` node, tree-sitter's `ts_builtin_sym_error`.
    pub const ERROR: Kind = Kind(u16::MAX);

    /// The kind of a node, or of a reference to one.
    pub fn of<'tree>(node: impl std::borrow::Borrow<tree_sitter::Node<'tree>>) -> Self {
        Self(node.borrow().kind_id())
    }

    pub const fn id(self) -> u16 {
        self.0
    }

    /// The name, as [`tree_sitter::Node::kind`] returns it.
    pub fn name(self) -> &'static str {
        if self == Self::ERROR {
            return "ERROR";
        }
        KIND_NAMES.get(usize::from(self.0)).copied().unwrap_or("")
    }
}

impl std::fmt::Debug for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Kind::{}", self.name())
    }
}

/// A field of the grammar: the id that [`tree_sitter::Node::child_by_field_id`] takes. Generated
/// like [`Kind`].
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Field(u16);

impl Field {
    /// The field of the name that [`tree_sitter::Node::field_name_for_child`] returns.
    pub fn named(name: &str) -> Option<Self> {
        let id = FIELD_NAMES
            .iter()
            .position(|n| !n.is_empty() && *n == name)?;
        u16::try_from(id).ok().map(Self)
    }

    pub const fn id(self) -> u16 {
        self.0
    }

    pub fn name(self) -> &'static str {
        FIELD_NAMES.get(usize::from(self.0)).copied().unwrap_or("")
    }
}

impl std::fmt::Debug for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Field::{}", self.name())
    }
}

include!(concat!(env!("OUT_DIR"), "/kinds.rs"));

#[cfg(test)]
mod tests {
    #[test]
    fn test_can_load_grammar() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&super::LANGUAGE.into())
            .expect("Error loading Postgres Language Server parser");
    }

    /// The generated constants have the language's ids: every visible kind of the language has
    /// its name at its id, and every field too.
    #[test]
    fn kinds_and_fields_have_the_ids_of_the_language() {
        let language: tree_sitter::Language = super::LANGUAGE.into();

        let mut kinds = 0;
        for id in 0..u16::try_from(language.node_kind_count()).unwrap() {
            if !language.node_kind_is_visible(id) {
                continue;
            }
            let name = language.node_kind_for_id(id).unwrap();
            let public = language.id_for_node_kind(name, language.node_kind_is_named(id));
            assert_eq!(super::Kind(public).name(), name, "kind {id}");
            kinds += 1;
        }
        assert!(kinds > 0);
        assert_eq!(super::Kind::ERROR.name(), "ERROR");

        for id in 1..=u16::try_from(language.field_count()).unwrap() {
            let name = language.field_name_for_id(id).unwrap();
            assert_eq!(super::Field(id).name(), name);
            assert_eq!(super::Field::named(name), Some(super::Field(id)));
        }
    }

    #[test]
    fn kinds_of_nodes_are_constants() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&super::LANGUAGE.into()).unwrap();
        let tree = parser.parse("select (a) from t;", None).unwrap();
        let root = tree.root_node();

        assert_eq!(super::Kind::of(root), super::Kind::PROGRAM);
        let mut cursor = root.walk();
        let mut kinds = Vec::new();
        loop {
            kinds.push(super::Kind::of(cursor.node()));
            if cursor.goto_first_child() || cursor.goto_next_sibling() {
                continue;
            }
            while !cursor.goto_next_sibling() {
                if !cursor.goto_parent() {
                    break;
                }
            }
            if cursor.node() == root {
                break;
            }
        }
        assert!(kinds.contains(&super::Kind::KEYWORD_SELECT));
        assert!(kinds.contains(&super::Kind::ANON_LPAREN));
        assert!(kinds.contains(&super::Kind::ANON_SEMI));
    }
}
