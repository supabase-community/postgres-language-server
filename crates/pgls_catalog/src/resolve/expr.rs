//! Expressions: column references, function calls, type names, and subqueries.

use pgls_query::{NodeEnum, protobuf};

use super::{
    Ctes, FindingKind, Resolver,
    scope::{ColumnLookup, Level, find_column, find_item, has_opaque_items},
    string_value, string_values,
};
use crate::view::Lookup;

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct ExprOptions {
    /// Whether bare column names can refer to the output columns of the select list, as in
    /// `ORDER BY` and `GROUP BY`.
    pub output_names: bool,
}

/// Types whose input is the name of a catalog object, which we don't resolve.
const OBJECT_NAME_TYPES: &[&str] = &[
    "regclass",
    "regcollation",
    "regconfig",
    "regdictionary",
    "regnamespace",
    "regoper",
    "regoperator",
    "regproc",
    "regprocedure",
    "regrole",
    "regtype",
];

impl Resolver<'_> {
    pub(super) fn expr(
        &mut self,
        node: &protobuf::Node,
        levels: &[&Level],
        ctes: Ctes,
        options: ExprOptions,
    ) {
        let Some(node) = node.node.as_ref() else {
            return;
        };
        let walk = |resolver: &mut Self, node: &Option<Box<protobuf::Node>>| {
            if let Some(node) = node.as_deref() {
                resolver.expr(node, levels, ctes, options);
            }
        };

        match node {
            NodeEnum::ColumnRef(column) => self.column_ref(column, levels, options),
            NodeEnum::FuncCall(call) => self.func_call(call, levels, ctes, options),
            NodeEnum::TypeCast(cast) => {
                walk(self, &cast.arg);
                if let Some(type_name) = &cast.type_name {
                    self.type_name(type_name);
                }
            }
            NodeEnum::SubLink(link) => {
                walk(self, &link.testexpr);
                if let Some(query) = link.subselect.as_deref() {
                    self.query(query, levels, ctes);
                }
            }
            NodeEnum::ResTarget(target) => walk(self, &target.val),
            NodeEnum::AExpr(expr) => {
                walk(self, &expr.lexpr);
                walk(self, &expr.rexpr);
            }
            NodeEnum::BoolExpr(expr) => self.exprs(&expr.args, levels, ctes, options),
            NodeEnum::NullTest(test) => walk(self, &test.arg),
            NodeEnum::BooleanTest(test) => walk(self, &test.arg),
            NodeEnum::CaseExpr(case) => {
                walk(self, &case.arg);
                self.exprs(&case.args, levels, ctes, options);
                walk(self, &case.defresult);
            }
            NodeEnum::CaseWhen(when) => {
                walk(self, &when.expr);
                walk(self, &when.result);
            }
            NodeEnum::CoalesceExpr(expr) => self.exprs(&expr.args, levels, ctes, options),
            NodeEnum::MinMaxExpr(expr) => self.exprs(&expr.args, levels, ctes, options),
            NodeEnum::RowExpr(expr) => self.exprs(&expr.args, levels, ctes, options),
            NodeEnum::AArrayExpr(expr) => self.exprs(&expr.elements, levels, ctes, options),
            NodeEnum::AIndirection(indirection) => {
                walk(self, &indirection.arg);
                self.exprs(&indirection.indirection, levels, ctes, options);
            }
            NodeEnum::AIndices(indices) => {
                walk(self, &indices.lidx);
                walk(self, &indices.uidx);
            }
            NodeEnum::CollateClause(collate) => walk(self, &collate.arg),
            NodeEnum::NamedArgExpr(arg) => walk(self, &arg.arg),
            NodeEnum::SortBy(sort) => walk(self, &sort.node),
            NodeEnum::GroupingSet(set) => self.exprs(&set.content, levels, ctes, options),
            NodeEnum::GroupingFunc(func) => self.exprs(&func.args, levels, ctes, options),
            NodeEnum::MultiAssignRef(assign) => walk(self, &assign.source),
            NodeEnum::WindowDef(window) => {
                self.exprs(&window.partition_clause, levels, ctes, options);
                self.exprs(&window.order_clause, levels, ctes, options);
                walk(self, &window.start_offset);
                walk(self, &window.end_offset);
            }
            NodeEnum::List(list) => self.exprs(&list.items, levels, ctes, options),
            // Field names in indirection (`(row).field`) and leaves.
            NodeEnum::String(_)
            | NodeEnum::AStar(_)
            | NodeEnum::AConst(_)
            | NodeEnum::ParamRef(_)
            | NodeEnum::SqlvalueFunction(_)
            | NodeEnum::SetToDefault(_)
            | NodeEnum::CurrentOfExpr(_) => {}
            // Anything else (XML and JSON expressions, ...) is not checked.
            _ => self.depends_on_file(),
        }
    }

    fn exprs(
        &mut self,
        nodes: &[protobuf::Node],
        levels: &[&Level],
        ctes: Ctes,
        options: ExprOptions,
    ) {
        for node in nodes {
            self.expr(node, levels, ctes, options);
        }
    }

    fn column_ref(
        &mut self,
        column: &protobuf::ColumnRef,
        levels: &[&Level],
        options: ExprOptions,
    ) {
        // `*` and `t.*`
        let Some(last) = column.fields.last() else {
            return;
        };
        if matches!(last.node, Some(NodeEnum::AStar(_))) {
            return;
        }
        let Some(names) = string_values(&column.fields) else {
            return;
        };

        match names.as_slice() {
            [name] => self.unqualified_column(name, levels, options, column.location),
            [qualifier, name] => self.qualified_column(qualifier, name, levels, column.location),
            [schema, relation, name] => {
                // `schema.table.column`, if `schema.table` is an unaliased item.
                let item = levels.iter().rev().find_map(|level| {
                    level.items.iter().find(|item| {
                        item.schema.as_deref() == Some(schema.as_str())
                            && item.name.as_deref() == Some(relation.as_str())
                    })
                });
                if let Some(item) = item {
                    if item.has_column(name) == Some(false) {
                        self.report(
                            FindingKind::UnknownColumn {
                                relation: Some(format!("{schema}.{relation}")),
                                column: name.clone(),
                            },
                            column.location,
                        );
                    }
                }
            }
            _ => {}
        }
    }

    fn unqualified_column(
        &mut self,
        name: &str,
        levels: &[&Level],
        options: ExprOptions,
        location: i32,
    ) {
        match find_column(levels, name, options.output_names) {
            ColumnLookup::Found | ColumnLookup::Unknown => {}
            ColumnLookup::Ambiguous(candidates) => self.report(
                FindingKind::AmbiguousColumn {
                    column: name.to_owned(),
                    candidates,
                },
                location,
            ),
            ColumnLookup::NotFound => {
                // A whole-row reference to a FROM item, or a parameter of the SQL function.
                let is_item = find_item(levels, name).is_some();
                if is_item || self.function_param(name).is_some() || has_opaque_items(levels) {
                    return;
                }
                self.report(
                    FindingKind::UnknownColumn {
                        relation: None,
                        column: name.to_owned(),
                    },
                    location,
                );
            }
        }
    }

    fn qualified_column(&mut self, qualifier: &str, name: &str, levels: &[&Level], location: i32) {
        if let Some(item) = find_item(levels, qualifier) {
            if item.has_column(name) == Some(false) {
                self.report(
                    FindingKind::UnknownColumn {
                        relation: Some(qualifier.to_owned()),
                        column: name.to_owned(),
                    },
                    location,
                );
            }
            return;
        }

        // Postgres also reads `a.b` as field `b` of the composite column `a`, or as the
        // function call `b(a)`.
        if find_column(levels, qualifier, false) != ColumnLookup::NotFound
            || has_opaque_items(levels)
        {
            return;
        }

        if let Some(function) = self.function {
            // `function_name.parameter`
            if function.function_name == qualifier {
                return;
            }
            // A field of a composite parameter.
            if let Some(param) = self.function_param(qualifier) {
                if param.is_array {
                    return;
                }
                let attributes = match self.catalog.type_(
                    param.type_schema.as_deref(),
                    &param.type_name,
                    self.search_path,
                ) {
                    Lookup::Found(type_info) => type_info.attributes,
                    _ => None,
                };
                if let Some(attributes) = attributes {
                    if !attributes.iter().any(|attribute| attribute.name == name) {
                        self.report(
                            FindingKind::UnknownColumn {
                                relation: Some(qualifier.to_owned()),
                                column: name.to_owned(),
                            },
                            location,
                        );
                    }
                }
                return;
            }
        }

        self.report(
            FindingKind::MissingFromClauseEntry {
                name: qualifier.to_owned(),
            },
            location,
        );
    }

    fn function_param(&self, name: &str) -> Option<&super::FunctionParam> {
        self.function?
            .params
            .iter()
            .find(|param| param.name.as_deref() == Some(name))
    }

    pub(super) fn func_call(
        &mut self,
        call: &protobuf::FuncCall,
        levels: &[&Level],
        ctes: Ctes,
        options: ExprOptions,
    ) {
        self.exprs(&call.args, levels, ctes, options);
        self.exprs(&call.agg_order, levels, ctes, options);
        if let Some(filter) = call.agg_filter.as_deref() {
            self.expr(filter, levels, ctes, options);
        }
        if let Some(window) = call.over.as_deref() {
            self.exprs(&window.partition_clause, levels, ctes, options);
            self.exprs(&window.order_clause, levels, ctes, options);
        }

        // SQL syntax like EXTRACT, TRIM, or AT TIME ZONE calls functions in pg_catalog.
        if call.funcformat() == protobuf::CoercionForm::CoerceSqlSyntax {
            return;
        }

        let Some(names) = string_values(&call.funcname) else {
            self.depends_on_file();
            return;
        };
        let (schema, name) = match names.as_slice() {
            [name] => (None, name.as_str()),
            [schema, name] => (Some(schema.as_str()), name.as_str()),
            _ => {
                self.depends_on_file();
                return;
            }
        };
        if let Some(schema) = schema {
            if self.check_schema(schema, call.location) {
                return;
            }
        }

        let arg_count = if call.agg_star { 0 } else { call.args.len() };
        match self.catalog.functions(schema, name, self.search_path) {
            Lookup::Found(overloads) => {
                for overload in &overloads {
                    self.uses(overload.origin);
                }
                // Ordered-set aggregates count their `WITHIN GROUP` arguments, and `VARIADIC`
                // passes an array for any number of arguments.
                if call.func_variadic || call.agg_within_group {
                    return;
                }
                let accepts = overloads.iter().any(|overload| {
                    overload.min_args <= arg_count
                        && overload.max_args.is_none_or(|max| arg_count <= max)
                });
                if !accepts && !self.may_be_field_access(call, levels) {
                    self.report(
                        FindingKind::UnknownFunction {
                            schema: schema.map(str::to_owned),
                            name: name.to_owned(),
                            arg_count,
                            name_exists: true,
                        },
                        call.location,
                    );
                }
            }
            Lookup::Missing => {
                if !self.may_be_field_access(call, levels) {
                    self.report(
                        FindingKind::UnknownFunction {
                            schema: schema.map(str::to_owned),
                            name: name.to_owned(),
                            arg_count,
                            name_exists: false,
                        },
                        call.location,
                    );
                }
            }
            Lookup::Unknown => self.depends_on_file(),
        }
    }

    /// Postgres reads `name(row)` as a field access if `row` is a whole row: `name(t)` is the
    /// same as `t.name`.
    fn may_be_field_access(&self, call: &protobuf::FuncCall, levels: &[&Level]) -> bool {
        let [argument] = call.args.as_slice() else {
            return false;
        };
        match argument.node.as_ref() {
            Some(NodeEnum::ColumnRef(column)) => match column.fields.as_slice() {
                [name] => string_value(name).is_some_and(|name| {
                    find_item(levels, name).is_some() || self.function_param(name).is_some()
                }),
                _ => false,
            },
            Some(NodeEnum::ParamRef(_) | NodeEnum::RowExpr(_) | NodeEnum::AIndirection(_)) => true,
            _ => false,
        }
    }

    pub(super) fn type_name(&mut self, type_name: &protobuf::TypeName) {
        if type_name.pct_type {
            self.depends_on_file();
            return;
        }
        let Some(names) = string_values(&type_name.names) else {
            self.depends_on_file();
            return;
        };
        let (schema, name) = match names.as_slice() {
            [] => return,
            [name] => (None, name.as_str()),
            [schema, name] => (Some(schema.as_str()), name.as_str()),
            _ => {
                self.depends_on_file();
                return;
            }
        };
        if let Some(schema) = schema {
            if self.check_schema(schema, type_name.location) {
                return;
            }
        }

        match self.catalog.type_(schema, name, self.search_path) {
            Lookup::Found(type_info) => {
                self.uses(type_info.origin);
                if OBJECT_NAME_TYPES.contains(&name) {
                    self.depends_on_file();
                }
            }
            Lookup::Missing => self.report(
                FindingKind::UnknownType {
                    schema: schema.map(str::to_owned),
                    name: name.to_owned(),
                },
                type_name.location,
            ),
            Lookup::Unknown => self.depends_on_file(),
        }
    }
}
