use pgt_text_size::TextSize;

use crate::OnHoverParams;

pub(crate) enum NodeIdentification {
    Name(String),
    SchemaAndName((String, String)),
    SchemaAndTableAndName((String, String, String)),
}

pub(crate) enum HoveredNode {
    Schema(NodeIdentification),
    Table(NodeIdentification),
    Function(NodeIdentification),
    Column(NodeIdentification),
    Policy(NodeIdentification),
    Trigger(NodeIdentification),
    Role(NodeIdentification),
}

impl HoveredNode {
    pub(crate) fn get(position: TextSize, cst: &tree_sitter::Tree) -> Self {
        todo!()
    }
}
