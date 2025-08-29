use std::path::Path;

use convert_case::{Case, Casing};
use prost_reflect::{DescriptorError, DescriptorPool};

pub(crate) struct ProtoAnalyzer {
    pool: DescriptorPool,
}

pub(crate) struct EnumVariant {
    pub name: String,
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

            variants.push(EnumVariant { name: variant_name });
        }

        variants
    }
}
