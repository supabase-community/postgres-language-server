use std::ops::Deref;

use proc_macro::TokenStream;
use quote::quote;
use syn::{TypePath, TypeTuple, parse_macro_input};

struct IgnoredPath {
    path: syn::Expr,
}

impl syn::parse::Parse for IgnoredPath {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let arg_name: syn::Ident = input.parse()?;

        if arg_name != "path" {
            return Err(syn::Error::new_spanned(
                arg_name,
                "Expected 'path' argument.",
            ));
        }

        let _: syn::Token!(=) = input.parse()?;
        let path: syn::Expr = input.parse()?;

        Ok(Self { path })
    }
}

#[proc_macro_attribute]
/// You can use this on a workspace server function to return a default if the specified path
/// is ignored by the user's settings.
///
/// This will work for any function where &self is in scope and that returns `Result<T, E>`, `Result<(), E>`, or `T`, where `T: Default`.
/// `path` needs to point at a `&PgLSPath`.
///
/// ### Usage
///
/// ```ignore
/// impl WorkspaceServer {
///   #[ignore_path(path=&params.path)]
///   fn foo(&self, params: FooParams) -> Result<FooResult, WorkspaceError> {
///     ... codeblock
///   }
/// }
///
/// // …expands to…
///
/// impl WorkspaceServer {
///   fn foo(&self, params: FooParams) -> Result<FooResult, WorkspaceError> {
///     if self.is_ignored(&params.path) {
///       return Ok(FooResult::default());
///     }
///     ... codeblock
///   }
/// }
/// ```
pub fn ignored_path(args: TokenStream, input: TokenStream) -> TokenStream {
    let ignored_path = parse_macro_input!(args as IgnoredPath);
    let input_fn = parse_macro_input!(input as syn::ItemFn);

    let macro_specified_path = ignored_path.path;

    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let block = &input_fn.block;
    let attrs = &input_fn.attrs;

    // handles cases `fn foo() -> Result<T, E>` and `fn foo() -> Result<(), E>`
    // T needs to implement default
    if let syn::ReturnType::Type(_, ty) = &sig.output {
        if let syn::Type::Path(TypePath { path, .. }) = ty.deref() {
            if let Some(seg) = path.segments.last() {
                if seg.ident == "Result" {
                    if let syn::PathArguments::AngleBracketed(type_args) = &seg.arguments {
                        if let Some(syn::GenericArgument::Type(t)) = type_args.args.first() {
                            if let syn::Type::Tuple(TypeTuple { elems, .. }) = t {
                                // case: Result<(), E>
                                if elems.is_empty() {
                                    return TokenStream::from(quote! {
                                      #(#attrs)*
                                      #vis #sig {
                                        if self.is_ignored(#macro_specified_path) {
                                          return Ok(());
                                        };
                                        #block
                                      }
                                    });
                                }
                            }
                            if let syn::Type::Path(TypePath { path, .. }) = t {
                                if let Some(seg) = path.segments.first() {
                                    let ident = &seg.ident;
                                    return TokenStream::from(quote! {
                                      #(#attrs)*
                                      #vis #sig {
                                        if self.is_ignored(#macro_specified_path) {
                                          return Ok(#ident::default());
                                        };
                                        #block
                                      }
                                    });
                                }
                            }
                        };
                    };
                };
            };
        };
    };

    // case fn foo() -> T {}
    // handles all other T's
    // T needs to implement Default
    TokenStream::from(quote! {
      #(#attrs)*
      #vis #sig {
        if self.is_ignored(#macro_specified_path) {
          return Default::default();
        }
        #block
      }
    })
}
