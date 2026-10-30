use proc_macro2::TokenStream;
use quote::quote;

use crate::proto_analyser::ProtoAnalyzer;

/// A table of the message-typed fields of every message, so that the nesting of a serialized
/// parse tree can be measured without decoding it.
pub fn nesting_mod(analyser: ProtoAnalyzer) -> TokenStream {
    let messages = analyser.message_fields();
    let ids: std::collections::HashMap<&str, u16> = messages
        .iter()
        .enumerate()
        .map(|(id, (name, _))| {
            let id = u16::try_from(id).expect("fewer than 65536 messages");
            (name.as_str(), id)
        })
        .collect();

    let parse_result = ids["ParseResult"];
    let arms: Vec<TokenStream> = messages
        .iter()
        .flat_map(|(name, fields)| {
            let message = ids[name.as_str()];
            fields
                .iter()
                .map(move |(number, type_name)| (message, number, type_name))
        })
        .map(|(message, number, type_name)| {
            let child = ids[type_name.as_str()];
            quote! { (#message, #number) => Some(#child), }
        })
        .collect();

    quote! {
        /// Identifies a message type of `pg_query.proto`.
        pub(crate) type MessageId = u16;

        /// The message type of the root of a parse result.
        pub(crate) const PARSE_RESULT: MessageId = #parse_result;

        /// The message type of field `field` of message type `message`, if it is a message.
        pub(crate) fn nested_message(message: MessageId, field: u32) -> Option<MessageId> {
            match (message, field) {
                #(#arms)*
                _ => None,
            }
        }
    }
}
