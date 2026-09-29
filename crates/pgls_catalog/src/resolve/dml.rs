//! INSERT, UPDATE, and DELETE.

use pgls_query::{NodeEnum, protobuf};

use super::{
    Ctes, FindingKind, Resolver,
    expr::ExprOptions,
    scope::{Columns, Item, Level},
};

impl Resolver<'_> {
    /// Resolves an INSERT and returns the columns of its RETURNING list.
    pub(super) fn insert(&mut self, insert: &protobuf::InsertStmt, ctes: Ctes) -> Columns {
        let with_ctes;
        let ctes = match &insert.with_clause {
            Some(with) => {
                with_ctes = self.with_clause(with, &[], ctes);
                with_ctes.as_slice()
            }
            None => ctes,
        };
        let Some(relation) = &insert.relation else {
            self.depends_on_file();
            return None;
        };
        let target = self.relation_item(relation);

        let mut target_columns = Vec::new();
        for column in &insert.cols {
            let Some(NodeEnum::ResTarget(column)) = &column.node else {
                self.depends_on_file();
                continue;
            };
            self.check_target_column(&target, &column.name, column.location);
            target_columns.push(column.name.clone());
        }

        if let Some(source) = insert.select_stmt.as_deref() {
            self.query(source, &[], ctes);
            let found = match &source.node {
                Some(NodeEnum::SelectStmt(select)) => select_width(select),
                _ => None,
            };
            let expected = if target_columns.is_empty() {
                target.columns.as_ref().map(Vec::len)
            } else {
                Some(target_columns.len())
            };
            if let (Some(found), Some(expected)) = (found, expected) {
                // Without a column list, fewer values than columns are fine.
                let mismatch = if target_columns.is_empty() {
                    found > expected
                } else {
                    found != expected
                };
                if mismatch {
                    self.report(FindingKind::InsertColumnMismatch { expected, found }, -1);
                }
            }
        }

        if let Some(conflict) = &insert.on_conflict_clause {
            let excluded = Item::named(Some("excluded".into()), target.columns.clone());
            let level = Level {
                items: vec![target.clone(), excluded],
                ..Default::default()
            };
            let levels = [&level];
            if let Some(infer) = &conflict.infer {
                if let Some(where_clause) = infer.where_clause.as_deref() {
                    self.expr(where_clause, &levels, ctes, ExprOptions::default());
                }
            }
            for assignment in &conflict.target_list {
                if let Some(NodeEnum::ResTarget(assignment)) = &assignment.node {
                    self.check_target_column(&target, &assignment.name, assignment.location);
                }
                self.expr(assignment, &levels, ctes, ExprOptions::default());
            }
            if let Some(where_clause) = conflict.where_clause.as_deref() {
                self.expr(where_clause, &levels, ctes, ExprOptions::default());
            }
        }

        let level = Level {
            items: vec![target],
            ..Default::default()
        };
        self.returning(&insert.returning_list, &level, ctes)
    }

    /// Resolves an UPDATE and returns the columns of its RETURNING list.
    pub(super) fn update(&mut self, update: &protobuf::UpdateStmt, ctes: Ctes) -> Columns {
        let with_ctes;
        let ctes = match &update.with_clause {
            Some(with) => {
                with_ctes = self.with_clause(with, &[], ctes);
                with_ctes.as_slice()
            }
            None => ctes,
        };
        let Some(relation) = &update.relation else {
            self.depends_on_file();
            return None;
        };
        let target = self.relation_item(relation);

        let mut level = Level {
            items: vec![target.clone()],
            ..Default::default()
        };
        let from = self.resolve_from(&update.from_clause, &[], ctes);
        level.extend(from);
        let levels = [&level];

        for assignment in &update.target_list {
            if let Some(NodeEnum::ResTarget(assignment)) = &assignment.node {
                self.check_target_column(&target, &assignment.name, assignment.location);
            }
            self.expr(assignment, &levels, ctes, ExprOptions::default());
        }
        if let Some(where_clause) = update.where_clause.as_deref() {
            self.expr(where_clause, &levels, ctes, ExprOptions::default());
        }
        self.returning(&update.returning_list, &level, ctes)
    }

    /// Resolves a DELETE and returns the columns of its RETURNING list.
    pub(super) fn delete(&mut self, delete: &protobuf::DeleteStmt, ctes: Ctes) -> Columns {
        let with_ctes;
        let ctes = match &delete.with_clause {
            Some(with) => {
                with_ctes = self.with_clause(with, &[], ctes);
                with_ctes.as_slice()
            }
            None => ctes,
        };
        let Some(relation) = &delete.relation else {
            self.depends_on_file();
            return None;
        };
        let target = self.relation_item(relation);

        let mut level = Level {
            items: vec![target],
            ..Default::default()
        };
        let using = self.resolve_from(&delete.using_clause, &[], ctes);
        level.extend(using);
        let levels = [&level];

        if let Some(where_clause) = delete.where_clause.as_deref() {
            self.expr(where_clause, &levels, ctes, ExprOptions::default());
        }
        self.returning(&delete.returning_list, &level, ctes)
    }

    fn returning(&mut self, returning: &[protobuf::Node], level: &Level, ctes: Ctes) -> Columns {
        let levels = [level];
        for target in returning {
            self.expr(target, &levels, ctes, ExprOptions::default());
        }
        self.target_columns(returning, &levels)
    }

    /// Reports a column of the target relation that doesn't exist.
    fn check_target_column(&mut self, target: &Item, column: &str, location: i32) {
        if target.has_column(column) == Some(false) {
            self.report(
                FindingKind::UnknownColumn {
                    relation: target.name.clone(),
                    column: column.to_owned(),
                },
                location,
            );
        }
    }
}

/// The number of columns a query produces, if it is certain.
fn select_width(select: &protobuf::SelectStmt) -> Option<usize> {
    if select.op() != protobuf::SetOperation::SetopNone {
        return select_width(select.larg.as_deref()?);
    }
    if let Some(row) = select.values_lists.first() {
        return match &row.node {
            Some(NodeEnum::List(row)) => Some(row.items.len()),
            _ => None,
        };
    }
    let mut width = 0;
    for target in &select.target_list {
        let NodeEnum::ResTarget(target) = target.node.as_ref()? else {
            return None;
        };
        if expands_to_columns(target.val.as_deref()?.node.as_ref()?) {
            return None;
        }
        width += 1;
    }
    Some(width)
}

/// `*`, `t.*`, and `(row).*` expand to any number of columns.
fn expands_to_columns(value: &NodeEnum) -> bool {
    let last = match value {
        NodeEnum::ColumnRef(column) => column.fields.last(),
        NodeEnum::AIndirection(indirection) => indirection.indirection.last(),
        _ => return false,
    };
    last.is_some_and(|last| matches!(last.node, Some(NodeEnum::AStar(_))))
}
