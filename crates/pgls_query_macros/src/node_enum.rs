use quote::{format_ident, quote};

use crate::proto_analyser::ProtoAnalyzer;

pub fn node_enum_mod(analyser: ProtoAnalyzer) -> proc_macro2::TokenStream {
    let node_variants = analyser.enum_variants();

    let mut to_ref_matches = Vec::new();
    let mut to_mut_matches = Vec::new();

    for variant in &node_variants {
        let variant_ident = format_ident!("{}", &variant.name);

        to_ref_matches.push(quote! {
            NodeEnum::#variant_ident(n) => NodeRef::#variant_ident(&n)
        });

        if variant.boxed {
            to_mut_matches.push(quote! {
                NodeEnum::#variant_ident(n) => NodeMut::#variant_ident(&mut **n as *mut _)
            });
        } else {
            to_mut_matches.push(quote! {
                NodeEnum::#variant_ident(n) => NodeMut::#variant_ident(n as *mut _)
            });
        }
    }

    quote! {
        impl NodeEnum {
            pub fn to_ref(&self) -> NodeRef {
                match self {
                    #(#to_ref_matches,)*
                }
            }

            pub fn to_mut(&mut self) -> NodeMut {
                match self {
                    #(#to_mut_matches,)*
                }
            }
        }
    }
}
