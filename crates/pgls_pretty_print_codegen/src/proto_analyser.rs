use std::path::Path;

use convert_case::{Case, Casing};
use prost_reflect::{DescriptorError, DescriptorPool, Kind};

pub(crate) struct ProtoAnalyzer {
    pool: DescriptorPool,
}

pub(crate) struct EnumVariant {
    pub name: String,
    /// True when the message behind this variant carries a `location` field, which holds the byte
    /// offset of the node in the original statement.
    pub has_location: bool,
}

impl ProtoAnalyzer {
    pub fn from(proto_file: &Path) -> Result<Self, DescriptorError> {
        let include_path = proto_file
            .parent()
            .expect("Proto file must have a parent directory");

        // protox::compile expects the proto file to be relative to the include path
        let file_name = proto_file
            .file_name()
            .expect("Proto file must have a file name");

        let pool = DescriptorPool::from_file_descriptor_set(
            protox::compile([file_name], [include_path]).expect("unable to parse"),
        )?;

        let analyzer = ProtoAnalyzer { pool };

        Ok(analyzer)
    }

    pub fn enum_variants(&self) -> Vec<EnumVariant> {
        let node = self
            .pool
            .get_message_by_name(".pg_query.Node")
            .expect("Node message not found");

        let mut variants = Vec::new();
        for field in node.fields() {
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
