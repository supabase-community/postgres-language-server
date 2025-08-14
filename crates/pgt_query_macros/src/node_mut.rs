use quote::{format_ident, quote};

use crate::proto_analyser::ProtoAnalyzer;

pub fn node_mut_mod(analyser: ProtoAnalyzer) -> proc_macro2::TokenStream {
    let node_variants = analyser.enum_variants();

    let mut to_enum_matches = Vec::new();
    let mut node_enum_variants = Vec::new();

    for variant in &node_variants {
        let variant_ident = format_ident!("{}", &variant.name);
        let type_ident = format_ident!("{}", &variant.type_name);

        if variant.boxed {
            // For boxed variants, we need to box the cloned value
            to_enum_matches.push(quote! {
                NodeMut::#variant_ident(n) => Ok(NodeEnum::#variant_ident(Box::new(n.as_ref().ok_or(err)?.clone())))
            });
        } else {
            // For non-boxed variants, clone directly
            to_enum_matches.push(quote! {
                NodeMut::#variant_ident(n) => Ok(NodeEnum::#variant_ident(n.as_ref().ok_or(err)?.clone()))
            });
        }

        node_enum_variants.push(quote! {
            #variant_ident(*mut protobuf::#type_ident)
        });
    }

    quote! {
        #[derive(Debug, Copy, Clone)]
        pub enum NodeMut {
            #(#node_enum_variants, )*
        }

        impl NodeMut {
            pub fn to_enum(self) -> Result<NodeEnum> {
                unsafe {
                    let err = Error::InvalidPointer;
                    match self {
                        #(#to_enum_matches,)*
                        _ => Err(Error::InvalidPointer),
                    }
                }
            }
        }
    }
}
