use syn::{
    Error, Ident, Meta, MetaNameValue, Path, Token, ext::IdentExt, parse::ParseStream,
    punctuated::Punctuated,
};

/// Calls `consume` for each item of the list `meta`, e.g. `derive(Eq)` and `skip_derive(Debug)`
/// in `#[partial(derive(Eq), skip_derive(Debug))]`.
pub(crate) fn parse_meta_list(
    meta: &Meta,
    mut consume: impl FnMut(&Meta) -> Result<(), Error>,
) -> Result<(), Error> {
    // syn 2 doesn't read a keyword as a path, but `#[partial(type)]` and
    // `#[partial(type = "...")]` name the type of a field.
    let item = |input: ParseStream| -> syn::Result<Meta> {
        if !input.peek(Token![type]) {
            return input.parse();
        }
        let path = Path::from(Ident::parse_any(input)?);
        if !input.peek(Token![=]) {
            return Ok(Meta::Path(path));
        }
        Ok(Meta::NameValue(MetaNameValue {
            path,
            eq_token: input.parse()?,
            value: input.parse()?,
        }))
    };
    let list = meta.require_list()?;
    for meta in list.parse_args_with(|input: ParseStream| {
        Punctuated::<Meta, Token![,]>::parse_terminated_with(input, item)
    })? {
        consume(&meta)?;
    }
    Ok(())
}
