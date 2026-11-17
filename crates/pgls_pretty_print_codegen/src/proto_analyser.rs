use std::path::Path;

use convert_case::{Case, Casing};
use prost_reflect::{DescriptorError, DescriptorPool, Kind, MessageDescriptor};

pub(crate) struct ProtoAnalyzer {
    /// The `Node` message, whose oneof lists every node type.
    node: MessageDescriptor,
}

pub(crate) struct EnumVariant {
    pub name: String,
    /// True when the message behind this variant carries a `location` field, which holds the byte
    /// offset of the node in the original statement.
    pub has_location: bool,
}

impl ProtoAnalyzer {
    /// Reads `pg_query.proto` at the path the build script sets. A failure is a `compile_error!`
    /// for the macro to return.
    pub(crate) fn load() -> Result<Self, proc_macro::TokenStream> {
        Self::from(Path::new(env!("PG_QUERY_PROTO_PATH"))).map_err(|error| {
            let message = format!("failed to read pg_query.proto: {error}");
            quote::quote! { compile_error!(#message); }.into()
        })
    }

    fn from(proto_file: &Path) -> Result<Self, String> {
        let include_path = proto_file
            .parent()
            .ok_or("the proto file has no parent directory")?;

        // protox::compile expects the proto file to be relative to the include path
        let file_name = proto_file
            .file_name()
            .ok_or("the proto file has no file name")?;

        let file_descriptor_set =
            protox::compile([file_name], [include_path]).map_err(|error| error.to_string())?;
        let pool = DescriptorPool::from_file_descriptor_set(file_descriptor_set)
            .map_err(|error: DescriptorError| error.to_string())?;
        let node = pool
            .get_message_by_name(".pg_query.Node")
            .ok_or("the Node message is missing")?;

        Ok(ProtoAnalyzer { node })
    }

    pub(crate) fn enum_variants(&self) -> Vec<EnumVariant> {
        let mut variants = Vec::new();
        for field in self.node.fields() {
            // The prost-generated variant name is derived from the field name using snake_case to PascalCase conversion
            // For example: ctesearch_clause -> CtesearchClause
            let field_name = field.name();
            let variant_name = field_name.to_case(Case::Pascal);

            let has_location = match field.kind() {
                Kind::Message(message) => message
                    .get_field_by_name("location")
                    .is_some_and(|location| matches!(location.kind(), Kind::Int32)),
                _ => false,
            };

            variants.push(EnumVariant {
                name: variant_name,
                has_location,
            });
        }

        variants
    }
}
