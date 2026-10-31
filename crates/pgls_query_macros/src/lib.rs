use iter_mut::iter_mut_mod;
use iter_ref::iter_ref_mod;
use node_enum::node_enum_mod;
use node_mut::node_mut_mod;
use node_ref::node_ref_mod;
use node_structs::node_structs_mod;
use proto_analyser::ProtoAnalyzer;
use quote::quote;
use std::path;

mod iter_mut;
mod iter_ref;
mod node_enum;
mod node_mut;
mod node_ref;
mod node_structs;
mod proto_analyser;

#[proc_macro]
pub fn node_ref_codegen(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let analyser = ProtoAnalyzer::from(&proto_file_path()).unwrap();

    let node_ref = node_ref_mod(analyser);

    quote! {
        use crate::*;

        #node_ref
    }
    .into()
}

#[proc_macro]
pub fn node_mut_codegen(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let analyser = ProtoAnalyzer::from(&proto_file_path()).unwrap();

    let node_mut = node_mut_mod(analyser);

    quote! {
        use crate::*;

        #node_mut
    }
    .into()
}

#[proc_macro]
pub fn node_structs_codegen(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let analyser = ProtoAnalyzer::from(&proto_file_path()).unwrap();

    let conversions = node_structs_mod(analyser);

    quote! {
        use crate::*;

        #conversions
    }
    .into()
}

#[proc_macro]
pub fn node_enum_codegen(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let analyser = ProtoAnalyzer::from(&proto_file_path()).unwrap();

    let node_enum = node_enum_mod(analyser);

    quote! {
        use crate::*;

        #node_enum
    }
    .into()
}

#[proc_macro]
pub fn iter_ref_codegen(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let analyser = ProtoAnalyzer::from(&proto_file_path()).unwrap();

    let iterator = iter_ref_mod(analyser);

    quote! {
        use crate::*;

        #iterator
    }
    .into()
}

#[proc_macro]
pub fn iter_mut_codegen(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let analyser = ProtoAnalyzer::from(&proto_file_path()).unwrap();

    let iterator = iter_mut_mod(analyser);

    quote! {
        use crate::*;

        #iterator
    }
    .into()
}

fn proto_file_path() -> path::PathBuf {
    // Use the path set by the build script
    path::PathBuf::from(env!("PG_QUERY_PROTO_PATH"))
}
