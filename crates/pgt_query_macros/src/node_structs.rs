use quote::{format_ident, quote};

use crate::proto_analyser::ProtoAnalyzer;

pub fn node_structs_mod(analyser: ProtoAnalyzer) -> proc_macro2::TokenStream {
    let node_variants = analyser.enum_variants();

    let mut impls = Vec::new();

    for variant in &node_variants {
        let node_ident = format_ident!("{}", &variant.name);
        let type_ident = format_ident!("{}", &variant.type_name);

        impls.push(quote! {
            impl protobuf::#type_ident {
                pub fn to_ref(&self) -> NodeRef {
                    NodeRef::#node_ident(self)
                }

                pub fn to_mut(&mut self) -> NodeMut {
                    NodeMut::#node_ident(self)
                }
            }
        });
    }

    quote! {
        #(#impls)*
    }
}
