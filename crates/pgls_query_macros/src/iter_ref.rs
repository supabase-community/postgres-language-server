use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::proto_analyser::{FieldType, Node, ProtoAnalyzer};

pub fn iter_ref_mod(analyser: ProtoAnalyzer) -> proc_macro2::TokenStream {
    let enum_variants = analyser.enum_variants();
    let nodes = analyser.nodes();

    let mut node_variant_names = Vec::new();
    let mut node_property_handlers = Vec::new();

    let mut type_to_variant: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    for variant in &enum_variants {
        type_to_variant.insert(variant.type_name.clone(), variant.name.clone());
    }

    for node in &nodes {
        if let Some(variant_name) = type_to_variant.get(&node.enum_variant_name) {
            let variant_ident = format_ident!("{}", variant_name);
            node_variant_names.push(variant_ident);

            let property_handlers = property_handlers(node);
            node_property_handlers.push(quote! {
                #(#property_handlers)*
            });
        } else {
            panic!(
                "No enum variant found for node type: {}",
                node.enum_variant_name
            );
        }
    }

    quote! {
        use std::collections::VecDeque;

        pub struct NodeRefIterator<'a>{
            stack: VecDeque<NodeRef<'a>>,
        }

        impl<'a> NodeRefIterator<'a> {
            pub fn new(root: NodeRef<'a>) -> Self {
                Self {
                    stack: VecDeque::from([root]),
                }
            }
        }

        impl<'a> Iterator for NodeRefIterator<'a> {
            type Item = NodeRef<'a>;

            fn next(&mut self) -> Option<Self::Item> {
                if self.stack.is_empty() {
                    return None;
                }

                let node = self.stack.pop_front().unwrap();

                match &node {
                    #(NodeRef::#node_variant_names(n) => {#node_property_handlers}),*,
                    _ => {
                        // Some node types don't have any child nodes to traverse
                    }
                };

                Some(node)
            }
        }
    }
}

fn property_handlers(node: &Node) -> Vec<TokenStream> {
    node.fields
        .iter()
        .filter_map(|field| {
            let field_name = format_ident!("{}", field.name.as_str());
            if matches!(field.r#type, FieldType::Node(_)) && field.repeated {
                Some(quote! {
                    n.#field_name
                        .iter()
                        .for_each(|x| {
                            if let Some(n) = x.node.as_ref() {
                                self.stack.push_back(n.to_ref());
                            }
                        });
                })
            } else if matches!(field.r#type, FieldType::Node(_)) && !field.is_one_of {
                if field.r#type == FieldType::Node(None) {
                    Some(quote! {
                        if let Some(n) = &n.#field_name {
                            if let Some(n) = n.node.as_ref() {
                                self.stack.push_back(n.to_ref());
                            }
                        }
                    })
                } else {
                    Some(quote! {
                        if let Some(field_node) = &n.#field_name {
                            self.stack.push_back(field_node.to_ref());
                        }
                    })
                }
            } else {
                None // Filter out non-node fields
            }
        })
        .collect()
}
