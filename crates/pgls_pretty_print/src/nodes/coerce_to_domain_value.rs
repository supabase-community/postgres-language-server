use pgls_query::protobuf::CoerceToDomainValue;

use super::prelude::*;

pub(super) fn emit_coerce_to_domain_value(e: &mut EventEmitter, _n: &CoerceToDomainValue) {
    e.group_start(GroupKind::CoerceToDomainValue);
    e.token(TokenKind::VALUE_KW);
    e.group_end();
}
