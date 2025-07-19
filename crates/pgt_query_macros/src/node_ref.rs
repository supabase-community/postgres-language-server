use quote::{format_ident, quote};

use crate::proto_analyser::ProtoAnalyzer;

pub fn node_ref_mod(analyser: ProtoAnalyzer) -> proc_macro2::TokenStream {
    let node_variants = analyser.enum_variants();

    let mut to_enum_matches = Vec::new();
    let mut node_enum_variants = Vec::new();

    for variant in &node_variants {
        let variant_ident = format_ident!("{}", &variant.name);
        let type_ident = format_ident!("{}", &variant.type_name);

        if variant.boxed {
            // For boxed variants, we need to box the cloned value
            to_enum_matches.push(quote! {
                NodeRef::#variant_ident(n) => NodeEnum::#variant_ident(::prost::alloc::boxed::Box::new((*n).clone()))
            });
        } else {
            // For non-boxed variants, clone directly
            to_enum_matches.push(quote! {
                NodeRef::#variant_ident(n) => NodeEnum::#variant_ident((*n).clone())
            });
        }

        node_enum_variants.push(quote! {
            #variant_ident(&'a protobuf::#type_ident)
        });
    }

    quote! {
        #[derive(Debug, Copy, Clone)]
        pub enum NodeRef<'a> {
            #(#node_enum_variants,)*
        }

        impl<'a> NodeRef<'a> {
            pub fn to_enum(self) -> NodeEnum {
                match self {
                    #(#to_enum_matches,)*
                }
            }
        }
    }
}
