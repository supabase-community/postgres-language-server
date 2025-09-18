use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use convert_case::{Case, Casing};
use prost_reflect::{
    DescriptorError, DescriptorPool, FieldDescriptor, MessageDescriptor,
    prost_types::{
        FieldDescriptorProto,
        field_descriptor_proto::{Label, Type},
    },
};

pub(crate) struct ProtoAnalyzer {
    pool: DescriptorPool,
    message_graph: HashMap<String, Vec<String>>,
}

pub(crate) struct EnumVariant {
    pub name: String,
    pub type_name: String,
    pub boxed: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum FieldType {
    Node(Option<String>),
    Enum(String),
    Literal,
}

pub(crate) struct Field {
    pub name: String,
    pub r#type: FieldType,
    pub repeated: bool,
    pub is_one_of: bool,
}

pub(crate) struct Node {
    #[allow(dead_code)]
    pub name: String,
    pub enum_variant_name: String,
    pub fields: Vec<Field>,
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

        let mut analyzer = ProtoAnalyzer {
            pool,
            message_graph: HashMap::new(),
        };

        // Build the message graph
        analyzer.build_message_graph();

        Ok(analyzer)
    }

    pub fn nodes(&self) -> Vec<Node> {
        let mut nodes = Vec::new();

        for msg in self.pool.all_messages() {
            if ["ParseResult", "ScanResult", "Node", "ScanToken"].contains(&msg.name()) {
                continue;
            }
            let fields = msg
                .fields()
                .map(|f| {
                    let field_type = match f.field_descriptor_proto().r#type() {
                        Type::Message => match f.field_descriptor_proto().type_name() {
                            ".pg_query.Node" => FieldType::Node(None),
                            name => {
                                FieldType::Node(Some(name.to_string().replace(".pg_query.", "")))
                            }
                        },
                        Type::Enum => FieldType::Enum(
                            f.field_descriptor_proto()
                                .type_name()
                                .to_string()
                                .replace(".pg_query.", ""),
                        ),
                        _ => FieldType::Literal,
                    };

                    Field {
                        name: f.name().to_string(),
                        r#type: field_type,
                        repeated: f.is_list(),
                        is_one_of: f.containing_oneof().is_some(),
                    }
                })
                .collect();

            nodes.push(Node {
                name: msg.name().to_string(),
                enum_variant_name: msg.name().to_case(Case::Pascal), // Convert to PascalCase for enum variant name
                fields,
            });
        }

        nodes
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

            // Get the actual proto type name (the message type)
            let proto_type_name = field
                .field_descriptor_proto()
                .type_name()
                .split('.')
                .next_back()
                .unwrap_or(&variant_name);

            // The Rust type name is the proto type name converted to PascalCase
            // For example: CTESearchClause -> CteSearchClause
            let type_name = proto_type_name.to_case(Case::Pascal);

            let boxed = self.is_field_boxed(&field, &node);

            variants.push(EnumVariant {
                name: variant_name,
                type_name,
                boxed,
            });
        }

        variants
    }

    /// Build a graph of message dependencies for cycle detection
    fn build_message_graph(&mut self) {
        // Collect all messages first to avoid borrowing issues
        let mut all_messages = Vec::new();
        for file in self.pool.files() {
            for message in file.messages() {
                all_messages.push(message);
            }
        }

        // Now add them to the graph
        for message in all_messages {
            self.add_message_to_graph(&message);
        }
    }

    /// Add a message and its dependencies to the graph
    fn add_message_to_graph(&mut self, message: &MessageDescriptor) {
        let msg_fq_name = format!(".{}", message.full_name());
        let mut dependencies = Vec::new();

        // Check all fields for message type dependencies
        for field in message.fields() {
            if let Some(field_message) = field.kind().as_message() {
                // Only add non-repeated message fields as dependencies
                // since repeated fields are already heap allocated in Vec
                if !field.is_list() {
                    let field_fq_name = format!(".{}", field_message.full_name());
                    dependencies.push(field_fq_name);
                }
            }
        }

        self.message_graph.insert(msg_fq_name, dependencies);

        // Recursively add nested messages
        for nested in message.child_messages() {
            self.add_message_to_graph(&nested);
        }
    }

    /// Detect if a field will be boxed by prost due to recursive nesting
    fn is_field_boxed(&self, field: &FieldDescriptor, parent_message: &MessageDescriptor) -> bool {
        // Check if this is a message field that should be boxed
        let parent_fq_name = format!(".{}", parent_message.full_name());
        self.is_boxed(&parent_fq_name, field.field_descriptor_proto())
    }

    /// Check if there's a path from parent_message to field_type in the message graph
    /// This indicates that field_type is transitively contained within parent_message
    fn is_nested(&self, parent_message_name: &str, field_type_name: &str) -> bool {
        self.has_path(parent_message_name, field_type_name, &mut HashSet::new())
    }

    /// Recursive helper to find if there's a path from 'from' to 'to' in the message graph
    fn has_path(&self, from: &str, to: &str, visited: &mut HashSet<String>) -> bool {
        // If we've already visited this node, return false to avoid cycles
        if visited.contains(from) {
            return false;
        }

        // If we've reached the target, we found a path
        if from == to {
            return true;
        }

        visited.insert(from.to_string());

        // Check all dependencies of the current message
        if let Some(dependencies) = self.message_graph.get(from) {
            for dep in dependencies {
                if self.has_path(dep, to, visited) {
                    return true;
                }
            }
        }

        false
    }

    /// Returns whether the Rust type for this message field is `Box<_>`.
    fn is_boxed(&self, fq_message_name: &str, field: &FieldDescriptorProto) -> bool {
        if field.label() == Label::Repeated {
            // Repeated field are stored in Vec, therefore it is already heap allocated
            return false;
        }
        let fd_type = field.r#type();
        if fd_type == Type::Message || fd_type == Type::Group {
            // The field should be boxed if the field type transitively contains the parent message
            // This prevents infinitely sized type definitions
            if let Some(field_type_name) = field.type_name.as_ref() {
                // IMPORTANT: Check if field_type_name contains fq_message_name (not the other way around)
                return self.is_nested(field_type_name, fq_message_name);
            }
        }
        false
    }
}
