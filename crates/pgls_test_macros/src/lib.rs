//! Test macros. `db_test` runs an async test against a fresh database.

use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

/// Runs an `async fn(PgPool)` test against a fresh database, like `#[sqlx::test]` with the
/// migrations of `pgls_test_utils`.
///
/// sqlx creates, migrates and drops the database as usual. On top of that, the test fails fast
/// with a clear message when the test database isn't running, and its pool is smaller and waits
/// longer for a connection, see `pgls_test_utils::db`.
///
/// ```ignore
/// #[pgls_test_utils::db_test]
/// async fn loads_the_snapshot(test_db: PgPool) { ... }
/// ```
#[proc_macro_attribute]
pub fn db_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return syn::Error::new(
            proc_macro2::TokenStream::from(attr).span(),
            "`db_test` takes no arguments",
        )
        .to_compile_error()
        .into();
    }

    let test_fn = syn::parse_macro_input!(item as syn::ItemFn);
    if test_fn.sig.asyncness.is_none() {
        return syn::Error::new(test_fn.sig.fn_token.span, "`db_test` needs an `async fn`")
            .to_compile_error()
            .into();
    }

    let attrs = &test_fn.attrs;
    let vis = &test_fn.vis;
    let name = &test_fn.sig.ident;
    let inputs = &test_fn.sig.inputs;
    let output = &test_fn.sig.output;
    let body = &test_fn.block;

    quote! {
        #(#attrs)*
        #[::core::prelude::v1::test]
        #vis fn #name() #output {
            async fn #name(#inputs) #output #body

            async fn with_test_pool(
                pool_options: ::sqlx::pool::PoolOptions<::sqlx::Postgres>,
                connect_options: ::sqlx::postgres::PgConnectOptions,
            ) #output {
                let pool = ::pgls_test_utils::db::connect_test_pool(pool_options, connect_options).await;
                let result = #name(pool.clone()).await;
                ::pgls_test_utils::db::close_test_pool(pool).await;
                result
            }

            ::pgls_test_utils::db::require_test_db();

            // The same test path as `#[sqlx::test]`, so the test keeps its database name.
            let mut args = ::sqlx::testing::TestArgs::new(concat!(module_path!(), "::", stringify!(#name)));
            args.migrator(&::pgls_test_utils::MIGRATIONS);

            let test: fn(_, _) -> _ = with_test_pool;
            ::sqlx::testing::TestFn::run_test(test, args)
        }
    }
    .into()
}
