mod group_kind;
mod keywords;
mod proto_analyser;
mod token_kind;

use std::path;

use proto_analyser::ProtoAnalyzer;
use token_kind::token_kind_mod;

#[proc_macro]
pub fn token_kind_codegen(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    token_kind_mod().into()
}

#[proc_macro]
pub fn group_kind_codegen(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let analyser = ProtoAnalyzer::from(&proto_file_path()).unwrap();
    group_kind::group_kind_mod(analyser).into()
}

fn proto_file_path() -> path::PathBuf {
    path::PathBuf::from(env!("PG_QUERY_PROTO_PATH"))
}
