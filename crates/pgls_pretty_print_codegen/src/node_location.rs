use quote::{format_ident, quote};

use crate::proto_analyser::ProtoAnalyzer;

pub fn node_location_mod(analyser: ProtoAnalyzer) -> proc_macro2::TokenStream {
    let arms = analyser
        .enum_variants()
        .into_iter()
        .filter(|variant| variant.has_location)
        .map(|variant| {
            let variant_ident = format_ident!("{}", &variant.name);
            quote! {
                pgls_query::NodeRef::#variant_ident(n) => Some(n.location)
            }
        });

    quote! {
        /// Byte offset of a node in the statement it was parsed from.
        ///
        /// Generated from the protobuf descriptor: a node kind reports its offset when its message
        /// carries a `location` field, and `None` otherwise. Comment attachment relies on it to
        /// find the node a comment sits in front of.
        #[allow(dead_code)] // Consumed by comment attachment in task 2.
        pub fn node_location(node: &pgls_query::NodeRef<'_>) -> Option<i32> {
            match node {
                #(#arms),*,
                _ => None,
            }
        }
    }
}
