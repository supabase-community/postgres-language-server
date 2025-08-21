mod keywords;
mod token_kind;

use token_kind::token_kind_mod;

#[proc_macro]
pub fn token_kind_codegen(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    token_kind_mod().into()
}
