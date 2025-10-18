use quote::{format_ident, quote};

use crate::proto_analyser::ProtoAnalyzer;

pub fn group_kind_mod(analyser: ProtoAnalyzer) -> proc_macro2::TokenStream {
    let node_variants = analyser.enum_variants();

    let mut node_enum_variants = Vec::new();

    for variant in &node_variants {
        let variant_ident = format_ident!("{}", &variant.name);

        node_enum_variants.push(quote! {
            #variant_ident
        });
    }

    quote! {
        #[derive(Clone, PartialEq, Debug)]
        pub enum GroupKind {
            #(#node_enum_variants),*,
        }
    }
}
