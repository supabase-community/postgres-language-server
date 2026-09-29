//! FROM clauses: relations, CTEs, subqueries, functions, and joins.

use pgls_query::{NodeEnum, protobuf};

use super::{
    Ctes, FindingKind, Resolver, apply_column_aliases,
    expr::ExprOptions,
    scope::{Columns, Item, Level},
    string_value, string_values,
};
use crate::view::{Lookup, RelationKind};

impl Resolver<'_> {
    /// Resolves a FROM list into the items of a query level. `outer` are the levels of the
    /// enclosing queries.
    pub(super) fn resolve_from(
        &mut self,
        from: &[protobuf::Node],
        outer: &[&Level],
        ctes: Ctes,
    ) -> Level {
        let mut level = Level::default();
        for node in from {
            let item = self.resolve_from_item(node, outer, &level, ctes);
            level.extend(item);
        }
        level
    }

    /// Resolves one FROM item. `preceding` holds the items before it, which LATERAL items and
    /// functions can reference.
    fn resolve_from_item(
        &mut self,
        node: &protobuf::Node,
        outer: &[&Level],
        preceding: &Level,
        ctes: Ctes,
    ) -> Level {
        let item = match node.node.as_ref() {
            Some(NodeEnum::RangeVar(range_var)) => self.range_var_item(range_var, ctes),
            Some(NodeEnum::RangeSubselect(subselect)) => {
                let columns = subselect.subquery.as_deref().and_then(|query| {
                    if subselect.lateral {
                        let levels = with_level(outer, preceding);
                        self.query(query, &levels, ctes)
                    } else {
                        self.query(query, outer, ctes)
                    }
                });
                let alias = subselect.alias.as_ref();
                Item::named(
                    alias.map(|alias| alias.aliasname.clone()),
                    apply_column_aliases(columns, alias),
                )
            }
            Some(NodeEnum::RangeFunction(function)) => {
                let levels = with_level(outer, preceding);
                self.range_function(function, &levels, ctes)
            }
            Some(NodeEnum::JoinExpr(join)) => return self.join(join, outer, preceding, ctes),
            _ => {
                // TABLESAMPLE, XMLTABLE, JSON_TABLE, ...: we don't know what they expose.
                self.depends_on_file();
                return Level {
                    items: vec![Item::named(None, None)],
                    has_opaque_items: true,
                    ..Default::default()
                };
            }
        };
        Level {
            items: vec![item],
            ..Default::default()
        }
    }

    /// A relation or CTE in FROM.
    fn range_var_item(&mut self, range_var: &protobuf::RangeVar, ctes: Ctes) -> Item {
        let alias = range_var.alias.as_ref();
        let is_unqualified = range_var.schemaname.is_empty() && range_var.catalogname.is_empty();
        if is_unqualified {
            if let Some(cte) = ctes.iter().rev().find(|cte| cte.name == range_var.relname) {
                return Item::named(
                    Some(alias.map_or_else(|| cte.name.clone(), |alias| alias.aliasname.clone())),
                    apply_column_aliases(cte.columns.clone(), alias),
                );
            }
        }
        self.relation_item(range_var)
    }

    /// A relation that is never a CTE: the target of INSERT, UPDATE, and DELETE.
    pub(super) fn relation_item(&mut self, range_var: &protobuf::RangeVar) -> Item {
        let alias = range_var.alias.as_ref();
        let relation = self.relation(range_var);
        Item {
            name: Some(alias.map_or_else(
                || range_var.relname.clone(),
                |alias| alias.aliasname.clone(),
            )),
            schema: relation.schema.filter(|_| alias.is_none()),
            columns: apply_column_aliases(relation.columns, alias),
            has_system_columns: relation.has_system_columns,
        }
    }

    /// Looks up a relation. Reports it if it doesn't exist.
    pub(super) fn relation(&mut self, range_var: &protobuf::RangeVar) -> ResolvedRelation {
        let unknown = ResolvedRelation::default();
        if !range_var.catalogname.is_empty() {
            self.depends_on_file();
            return unknown;
        }
        let schema = (!range_var.schemaname.is_empty()).then_some(range_var.schemaname.as_str());
        if let Some(schema) = schema {
            if self.check_schema(schema, range_var.location) {
                return unknown;
            }
        }

        match self
            .catalog
            .relation(schema, &range_var.relname, self.search_path)
        {
            Lookup::Found(relation) => {
                self.uses(relation.origin);
                ResolvedRelation {
                    schema: Some(relation.schema),
                    columns: relation
                        .columns
                        .map(|columns| columns.into_iter().map(|column| column.name).collect()),
                    has_system_columns: relation.kind != RelationKind::View,
                }
            }
            Lookup::Missing => {
                self.report(
                    FindingKind::UnknownRelation {
                        schema: schema.map(str::to_owned),
                        name: range_var.relname.clone(),
                    },
                    range_var.location,
                );
                unknown
            }
            Lookup::Unknown => {
                self.depends_on_file();
                unknown
            }
        }
    }

    /// A function in FROM, including `ROWS FROM (...)` and `WITH ORDINALITY`.
    fn range_function(
        &mut self,
        function: &protobuf::RangeFunction,
        levels: &[&Level],
        ctes: Ctes,
    ) -> Item {
        let mut columns: Columns = Some(Vec::new());
        let mut function_name = None;

        for entry in &function.functions {
            // Each entry is a list of the function call and its column definitions.
            let Some(NodeEnum::List(entry)) = entry.node.as_ref() else {
                self.depends_on_file();
                columns = None;
                continue;
            };
            let call = entry.items.first();
            let column_definitions = entry.items.get(1).and_then(|node| match &node.node {
                Some(NodeEnum::List(list)) if !list.items.is_empty() => Some(&list.items),
                _ => None,
            });

            let mut function_columns = None;
            match call.and_then(|call| call.node.as_ref()) {
                Some(NodeEnum::FuncCall(call)) => {
                    self.func_call(call, levels, ctes, ExprOptions::default());
                    function_columns = self.function_columns(call);
                    if function.functions.len() == 1 {
                        function_name = call
                            .funcname
                            .last()
                            .and_then(string_value)
                            .map(str::to_owned);
                    }
                }
                Some(_) => {
                    if let Some(call) = call {
                        self.expr(call, levels, ctes, ExprOptions::default());
                    }
                }
                None => {}
            }
            if let Some(definitions) = column_definitions {
                function_columns = column_definition_names(definitions);
            }

            columns = match (columns, function_columns) {
                (Some(mut columns), Some(function_columns)) => {
                    columns.extend(function_columns);
                    Some(columns)
                }
                _ => None,
            };
        }

        if !function.coldeflist.is_empty() {
            columns = column_definition_names(&function.coldeflist);
        }
        if function.ordinality {
            if let Some(columns) = columns.as_mut() {
                columns.push("ordinality".into());
            }
        }

        let alias = function.alias.as_ref();
        Item::named(
            alias.map(|alias| alias.aliasname.clone()).or(function_name),
            apply_column_aliases(columns, alias),
        )
    }

    /// The output columns of a function call in FROM, if all its overloads agree on them.
    fn function_columns(&self, call: &protobuf::FuncCall) -> Columns {
        let names = string_values(&call.funcname)?;
        let (name, schema) = match names.as_slice() {
            [name] => (name, None),
            [schema, name] => (name, Some(schema.as_str())),
            _ => return None,
        };
        let Lookup::Found(overloads) = self.catalog.functions(schema, name, self.search_path)
        else {
            return None;
        };
        let first = overloads.first()?.return_columns.clone()?;
        overloads
            .iter()
            .all(|overload| overload.return_columns.as_ref() == Some(&first))
            .then(|| first.into_iter().map(|column| column.name).collect())
    }

    fn join(
        &mut self,
        join: &protobuf::JoinExpr,
        outer: &[&Level],
        preceding: &Level,
        ctes: Ctes,
    ) -> Level {
        let Some(left) = join.larg.as_deref() else {
            self.depends_on_file();
            return Level::default();
        };
        let Some(right) = join.rarg.as_deref() else {
            self.depends_on_file();
            return Level::default();
        };

        let left = self.resolve_from_item(left, outer, preceding, ctes);
        // LATERAL items on the right can reference the left side.
        let mut preceding_right = preceding.clone();
        preceding_right.extend(left.clone());
        let right = self.resolve_from_item(right, outer, &preceding_right, ctes);

        let using_columns = string_values(&join.using_clause).unwrap_or_default();
        let mut joined = left;
        joined.extend(right);
        joined.merged_columns.extend(using_columns.iter().cloned());
        joined.has_natural_join |= join.is_natural;

        if let Some(quals) = join.quals.as_deref() {
            let levels = with_level(outer, &joined);
            self.expr(quals, &levels, ctes, ExprOptions::default());
        }

        if let Some(alias) = &join.alias {
            // Only the alias of the join is visible outside of it.
            let columns = if joined.has_natural_join || !joined.merged_columns.is_empty() {
                None
            } else {
                joined.star_columns()
            };
            return Level {
                items: vec![Item::named(
                    Some(alias.aliasname.clone()),
                    apply_column_aliases(columns, Some(alias)),
                )],
                has_opaque_items: joined.has_opaque_items,
                ..Default::default()
            };
        }

        if let Some(alias) = &join.join_using_alias {
            joined.items.push(Item::named(
                Some(alias.aliasname.clone()),
                Some(using_columns),
            ));
        }
        joined
    }
}

/// A relation found in the catalog. Everything is `None` if it was not found.
#[derive(Debug, Default)]
pub(super) struct ResolvedRelation {
    pub schema: Option<String>,
    pub columns: Columns,
    pub has_system_columns: bool,
}

/// The enclosing levels plus one more.
pub(super) fn with_level<'a>(outer: &[&'a Level], level: &'a Level) -> Vec<&'a Level> {
    outer
        .iter()
        .copied()
        .chain(std::iter::once(level))
        .collect()
}

fn column_definition_names(definitions: &[protobuf::Node]) -> Columns {
    definitions
        .iter()
        .map(|definition| match &definition.node {
            Some(NodeEnum::ColumnDef(column)) => Some(column.colname.clone()),
            _ => None,
        })
        .collect()
}
