mod keywords;
mod node_location;
mod proto_analyser;
mod token_kind;

use proto_analyser::ProtoAnalyzer;
use token_kind::token_kind_mod;

#[proc_macro]
pub fn token_kind_codegen(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    token_kind_mod().into()
}

#[proc_macro]
pub fn node_location_codegen(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match ProtoAnalyzer::load() {
        Ok(analyser) => node_location::node_location_mod(analyser).into(),
        Err(error) => error,
    }
}
