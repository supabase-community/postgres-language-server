mod keywords;
mod syntax_kind;

use syntax_kind::syntax_kind_mod;

#[proc_macro]
pub fn syntax_kind_codegen(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    syntax_kind_mod().into()
}
