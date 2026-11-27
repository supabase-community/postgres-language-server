use crate::{
    NodeEnum,
    protobuf::{AIndirection, ColumnRef, Node},
};

impl NodeEnum {
    /// Whether a select list entry is a `*` that expands to any number of columns, like `*`,
    /// `t.*` or `(t.c).*`.
    pub fn is_star(&self) -> bool {
        match self {
            NodeEnum::ColumnRef(column) => column.star_qualifier().is_some(),
            NodeEnum::AIndirection(indirection) => indirection.star_qualifier().is_some(),
            _ => false,
        }
    }
}

impl ColumnRef {
    /// The fields before the `*` of a column reference like `*`, `t.*` or `s.t.*`. `None` if the
    /// reference doesn't end in `*`.
    pub fn star_qualifier(&self) -> Option<&[Node]> {
        let (last, qualifier) = self.fields.split_last()?;
        matches!(last.node, Some(NodeEnum::AStar(_))).then_some(qualifier)
    }
}

impl AIndirection {
    /// The indirection before the `*` of a field selection like `(t.c).*` or `(t.c).f.*`. `None`
    /// if the indirection doesn't end in `*`.
    pub fn star_qualifier(&self) -> Option<&[Node]> {
        let (last, qualifier) = self.indirection.split_last()?;
        matches!(last.node, Some(NodeEnum::AStar(_))).then_some(qualifier)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Node, NodeEnum};

    fn select_values(sql: &str) -> Vec<NodeEnum> {
        let NodeEnum::SelectStmt(select) = crate::parse(sql).unwrap().into_root().unwrap() else {
            panic!("Expected SelectStmt");
        };
        select
            .target_list
            .iter()
            .map(|target| match target.node.as_ref().unwrap() {
                NodeEnum::ResTarget(target) => target.val.as_ref().unwrap().node.clone().unwrap(),
                _ => panic!("Expected ResTarget"),
            })
            .collect()
    }

    #[test]
    fn stars() {
        let values = select_values("select *, t.*, s.t.*, (t.c).*, t.c, (t.c).f from t");
        let stars: Vec<_> = values.iter().map(NodeEnum::is_star).collect();
        assert_eq!(stars, [true, true, true, true, false, false]);

        let qualifiers: Vec<_> = values
            .iter()
            .map(|value| {
                let qualifier = match value {
                    NodeEnum::ColumnRef(column) => column.star_qualifier()?,
                    NodeEnum::AIndirection(indirection) => indirection.star_qualifier()?,
                    _ => return None,
                };
                Node::string_values(qualifier)
            })
            .collect();
        let strings = |parts: &[&str]| Some(parts.iter().map(|&part| part.to_owned()).collect());
        assert_eq!(
            qualifiers,
            [
                strings(&[]),
                strings(&["t"]),
                strings(&["s", "t"]),
                strings(&[]),
                None,
                None
            ]
        );
    }
}
