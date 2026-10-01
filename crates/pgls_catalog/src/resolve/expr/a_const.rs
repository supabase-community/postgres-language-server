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
            Val::Fval(f) => {
                let text = &f.fval;
                if text.contains('.') || text.contains('e') || text.contains('E') {
                    named(r, "numeric")
                } else if text.parse::<i64>().is_ok() {
                    named(r, "int8")
                } else {
                    named(r, "numeric")
                }
            }
            Val::Boolval(_) => named(r, "bool"),
            Val::Sval(_) => Some(Type::UnknownLiteral),
            Val::Bsval(_) => named(r, "bit"),
        }
    }
}
