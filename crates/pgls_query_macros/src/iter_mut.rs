use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::proto_analyser::{FieldType, Node, ProtoAnalyzer};

pub fn iter_mut_mod(analyser: ProtoAnalyzer) -> proc_macro2::TokenStream {
    let enum_variants = analyser.enum_variants();
    let nodes = analyser.nodes();

    let mut node_variant_names = Vec::new();
    let mut node_property_handlers = Vec::new();

    // Create a map from type name to enum variant name
    let mut type_to_variant: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    for variant in &enum_variants {
        type_to_variant.insert(variant.type_name.clone(), variant.name.clone());
    }

    for node in &nodes {
        // Use the enum variant name from the Node enum
        if let Some(variant_name) = type_to_variant.get(&node.enum_variant_name) {
            let variant_ident = format_ident!("{}", variant_name);
            node_variant_names.push(variant_ident);

            let property_handlers = property_handlers(node);
            node_property_handlers.push(property_handlers);
        } else {
            panic!(
                "No enum variant found for node type: {}",
                node.enum_variant_name
            );
        }
    }

    quote! {
        use std::collections::VecDeque;

        /// An iterator that provides mutable access to all nodes in an AST tree.
        ///
        /// This iterator performs a depth-first traversal of the AST, yielding mutable
        /// references to each node. It uses unsafe operations internally to work with
        /// raw pointers in the AST structure.
        ///
        /// # Safety Requirements
        ///
        /// Users of this iterator must ensure:
        ///
        /// - The root `NodeMut` passed to `new()` must point to a valid, properly
        ///   constructed AST that remains alive for the iterator's lifetime
        /// - No other code concurrently accesses or modifies the AST while this
        ///   iterator is in use (exclusive access required)
        /// - The AST structure must not be modified through other means while
        ///   iterating (e.g., don't modify parent nodes while iterating children)
        ///
        /// # Panics
        ///
        /// This iterator may panic or cause undefined behavior if the safety
        /// requirements above are violated.
        /// ```
        pub struct NodeMutIterator {
            stack: VecDeque<NodeMut>,
        }

        impl NodeMutIterator {
            /// Creates a new iterator starting from the given root node.
            ///
            /// # Safety
            ///
            /// The caller must ensure that `roots` points to valid AST nodes
            /// and that the safety requirements documented on `NodeMutIterator`
            /// are met throughout the iterator's lifetime.
            pub fn new(root: NodeMut) -> Self {
                Self {
                    stack: VecDeque::from([root]),
                }
            }
        }

        impl Iterator for NodeMutIterator {
            type Item = NodeMut;

            fn next(&mut self) -> Option<Self::Item> {
                if self.stack.is_empty() {
                    return None;
                }

                let node = self.stack.pop_front().unwrap();

                unsafe {
                    match node {
                        #(NodeMut::#node_variant_names(n) => {#node_property_handlers}),*,
                        _ => {
                            // Some node types don't have any child nodes to traverse
                        }
                    };
                }

                Some(node)
            }
        }
    }
}

fn property_handlers(node: &Node) -> TokenStream {
    let handlers: Vec<TokenStream> = node
        .fields
        .iter()
        .filter_map(|field| {
            let field_name = format_ident!("{}", field.name.as_str());
            if matches!(field.r#type, FieldType::Node(_)) && field.repeated {
                Some(quote! {
                    n.#field_name
                        .iter_mut()
                        .for_each(|x| {
                            if let Some(n) = x.node.as_mut() {
                                self.stack.push_back(n.to_mut());
                            }
                        });
                })
            } else if matches!(field.r#type, FieldType::Node(_)) && !field.is_one_of {
                if field.r#type == FieldType::Node(None) {
                    Some(quote! {
                        if let Some(n) = n.#field_name.as_mut() {
                            if let Some(n) = n.node.as_mut() {
                                self.stack.push_back(n.to_mut());
                            }
                        }
                    })
                } else {
                    Some(quote! {
                        if let Some(field_node) = n.#field_name.as_mut() {
                            self.stack.push_back(field_node.to_mut());
                        }
                    })
                }
            } else {
                None // Filter out non-node fields
            }
        })
        .collect();

    quote! {
        let n = n.as_mut().unwrap();
        #(#handlers)*
    }
}
