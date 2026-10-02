use super::*;

pub(super) fn infer_a_const(
    r: &mut Resolver<'_>,
    n: &pgls_query::protobuf::AConst,
) -> Option<Type> {
    {
        use protobuf::a_const::Val;
        if n.isnull {
            return Some(Type::UnknownLiteral);
        }
        match n.val.as_ref()? {
            Val::Ival(_) => named(r, "int4"),
            Val::Fval(f) => named(r, float_const_type(&f.fval)),
            Val::Boolval(_) => named(r, "bool"),
            Val::Sval(_) => Some(Type::UnknownLiteral),
            Val::Bsval(_) => named(r, "bit"),
        }
    }
}

/// The type of a `T_Float` constant, which is also an integer too large for `Ival`. Port of
/// `parse_node.c: make_const`: integers become int4 if they fit (only `-2147483648` does, after
/// `doNegate`), int8 if they fit that, and numeric otherwise.
fn float_const_type(text: &str) -> &'static str {
    match parse_integer(text) {
        Some(value) if i32::try_from(value).is_ok() => "int4",
        Some(_) => "int8",
        None => "numeric",
    }
}

/// Port of `numutils.c: pg_strtoint64_safe`: decimal, `0x`, `0o` and `0b` integers with
/// underscores between digits.
fn parse_integer(text: &str) -> Option<i64> {
    let (sign, digits) = match text.strip_prefix('-') {
        Some(digits) => ("-", digits),
        None => ("", text.strip_prefix('+').unwrap_or(text)),
    };
    let (radix, digits) = match digits.get(..2) {
        Some("0x" | "0X") => (16, &digits[2..]),
        Some("0o" | "0O") => (8, &digits[2..]),
        Some("0b" | "0B") => (2, &digits[2..]),
        _ => (10, digits),
    };
    if digits.is_empty()
        || digits.starts_with('_')
        || digits.ends_with('_')
        || digits.contains("__")
    {
        return None;
    }
    i64::from_str_radix(&format!("{sign}{}", digits.replace('_', "")), radix).ok()
}
