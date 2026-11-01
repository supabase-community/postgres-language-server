//! The state of name resolution: the catalog, the names in scope, and the findings so far.

use std::mem::Discriminant;

use pgls_text_size::{TextRange, TextSize};
use rustc_hash::{FxHashMap, FxHashSet};

use pgls_query::{
    NodeEnum,
    protobuf::{self, CoercionForm, FuncCall},
};

use super::{
    Finding, FindingKind, FunctionContext, FunctionParam, Reference, ResolvedCallArgument,
    scope::{Cte, Level},
    span,
};
use crate::lookup::{CatalogView, Lookup, Origin};
use crate::typing::TypeId;
use crate::{FunctionInfo, TypeKind};

pub(super) struct Resolver<'a> {
    pub catalog: &'a dyn CatalogView,
    pub search_path: &'a [String],
    pub function: Option<&'a FunctionContext>,
    sql: Option<&'a str>,
    pub findings: Vec<Finding>,
    /// Cleared as soon as the statement references something that doesn't come unchanged from
    /// the database, or something the resolver doesn't understand.
    pub database_only: bool,
    /// The FROM items of the enclosing query levels, innermost last.
    pub levels: Vec<Level>,
    /// The common table expressions in scope, innermost last.
    pub ctes: Vec<Cte>,
    pub output: crate::typing::QueryColumns,
    /// The references resolved so far, when they are being collected.
    pub references: Option<Vec<Reference>>,
    /// The location and kind of each reference in `references`.
    recorded: FxHashSet<(i32, Discriminant<Reference>)>,
    /// The output columns of the subqueries of sublinks resolved so far, by the address of the
    /// subquery's node, so that typing a sublink doesn't resolve its subquery again. Resolving
    /// it again at every level of nested subqueries took time exponential in their depth.
    subquery_outputs: FxHashMap<usize, crate::typing::QueryColumns>,
}

impl<'a> Resolver<'a> {
    pub fn new(
        catalog: &'a dyn CatalogView,
        search_path: &'a [String],
        function: Option<&'a FunctionContext>,
        sql: Option<&'a str>,
    ) -> Self {
        Self {
            catalog,
            search_path,
            function,
            sql,
            findings: Vec::new(),
            database_only: true,
            levels: Vec::new(),
            ctes: Vec::new(),
            output: None,
            references: None,
            recorded: FxHashSet::default(),
            subquery_outputs: FxHashMap::default(),
        }
    }

    /// Remembers the output columns of `query`, the subquery of a sublink.
    pub fn remember_subquery_output(
        &mut self,
        query: &protobuf::Node,
        columns: crate::typing::QueryColumns,
    ) {
        self.subquery_outputs
            .insert(std::ptr::from_ref(query) as usize, columns);
    }

    /// The output columns of `query`, the subquery of a sublink, if it was resolved before.
    pub fn subquery_output(&self, query: &protobuf::Node) -> Option<&crate::typing::QueryColumns> {
        self.subquery_outputs
            .get(&(std::ptr::from_ref(query) as usize))
    }

    /// Makes the FROM items of `level` visible, until [`Resolver::exit_level`].
    pub fn enter_level(&mut self, level: Level) {
        self.levels.push(level);
    }

    pub fn exit_level(&mut self) -> Level {
        self.levels.pop().unwrap_or_default()
    }

    /// Whether bare column names can refer to the output columns of the innermost level, as in
    /// `ORDER BY` and `GROUP BY`.
    pub fn set_output_names_visible(&mut self, visible: bool) {
        if let Some(level) = self.levels.last_mut() {
            level.output_names_visible = visible;
        }
    }

    /// The parameter of the SQL function with this name.
    pub fn function_param(&self, name: &str) -> Option<&'a FunctionParam> {
        let index = self.function_param_index(name)?;
        self.function?.params.get(index)
    }

    /// The index of the parameter of the SQL function with this name.
    pub fn function_param_index(&self, name: &str) -> Option<usize> {
        self.function?
            .params
            .iter()
            .position(|param| param.name.as_deref() == Some(name))
    }

    /// SQL function bodies are only checked against the database if their parameters have
    /// scalar types from the database: composite parameters can't be replaced by literals.
    pub fn check_function_params(&mut self) {
        let Some(function) = self.function else {
            return;
        };
        for param in &function.params {
            let lookup = self.catalog.type_(
                param.type_schema.as_deref(),
                &param.type_name,
                self.search_path,
            );
            match lookup {
                Lookup::Found(type_info)
                    if type_info.origin == Origin::Database && type_info.attributes.is_none() => {}
                _ => self.database_only = false,
            }
        }
    }

    /// Whether references are being collected.
    pub fn collects_references(&self) -> bool {
        self.references.is_some()
    }

    /// Records a reference, when references are being collected.
    pub fn record(&mut self, reference: Reference) {
        let Some(references) = self.references.as_mut() else {
            return;
        };
        // Some expressions are resolved more than once, e.g. VALUES rows in INSERT.
        if self
            .recorded
            .insert((reference.location(), std::mem::discriminant(&reference)))
        {
            references.push(reference);
        }
    }

    /// Records that `call` resolves to `function`, when references are being collected.
    pub fn record_call(&mut self, call: &FuncCall, function: &FunctionInfo) {
        if !self.collects_references() {
            return;
        }
        // SQL syntax like EXTRACT or TRIM doesn't pass the arguments in order.
        if call.funcformat() == CoercionForm::CoerceSqlSyntax {
            return;
        }
        let Some(signature) = function.signature.as_ref() else {
            return;
        };
        let parameters = &signature.arguments;
        let has_names = call
            .args
            .iter()
            .any(|arg| matches!(arg.node, Some(NodeEnum::NamedArgExpr(_))));
        // Without `VARIADIC` and named arguments, the arguments from the variadic parameter on
        // are its elements.
        let expanded = !call.func_variadic && !has_names && signature.variadic_element.is_some();
        let mut arguments = Vec::with_capacity(call.args.len());
        for (i, arg) in call.args.iter().enumerate() {
            let (parameter, named) = match arg.node.as_ref() {
                Some(NodeEnum::NamedArgExpr(named)) => (
                    parameters
                        .iter()
                        .position(|param| param.name.as_deref() == Some(named.name.as_str())),
                    true,
                ),
                _ if expanded => (Some(i.min(parameters.len().saturating_sub(1))), false),
                _ => (Some(i), false),
            };
            let Some(parameter) = parameter.filter(|&parameter| parameter < parameters.len())
            else {
                return;
            };
            arguments.push(ResolvedCallArgument { parameter, named });
        }
        for (arg, argument) in call.args.iter().zip(&arguments) {
            let value = match arg.node.as_ref() {
                Some(NodeEnum::NamedArgExpr(named)) => {
                    named.arg.as_deref().and_then(|arg| arg.node.as_ref())
                }
                value => value,
            };
            let is_variadic = expanded && argument.parameter + 1 == parameters.len();
            let target = if is_variadic {
                signature.variadic_element.as_ref()
            } else {
                parameters[argument.parameter].ty.as_ref()
            };
            if let Some(target) = target {
                self.record_enum_literal(value, target);
            }
        }
        self.record(Reference::Function {
            location: call.location,
            span: self.name_span(call.location),
            function: function.clone(),
            arguments,
        });
    }

    /// Records that the parameter `index` of the SQL function is referenced at `location`, when
    /// references are being collected.
    pub fn record_parameter(&mut self, index: usize, location: i32) {
        if !self.collects_references() {
            return;
        }
        self.record(Reference::Parameter {
            location,
            span: self.name_span(location),
            index,
        });
    }

    /// Records `node` if it is a string literal that is coerced to `target`, and `target` is an
    /// enum type.
    pub fn record_enum_literal(&mut self, node: Option<&NodeEnum>, target: &TypeId) {
        if !self.collects_references() {
            return;
        }
        let Some(NodeEnum::AConst(constant)) = node else {
            return;
        };
        if !matches!(constant.val, Some(protobuf::a_const::Val::Sval(_))) {
            return;
        }
        let Lookup::Found(type_info) = self.catalog.type_by_id(target) else {
            return;
        };
        if type_info.kind != Some(TypeKind::Enum) {
            return;
        }
        self.record(Reference::EnumLiteral {
            location: constant.location,
            span: self.literal_span(constant.location),
            type_info,
        });
    }

    /// Marks the statement as depending on something that is not an unchanged database object.
    pub fn depends_on_file(&mut self) {
        self.database_only = false;
    }

    /// Records the origin of a found object.
    pub fn uses(&mut self, origin: Origin) {
        if origin != Origin::Database {
            self.database_only = false;
        }
    }

    pub fn report(&mut self, kind: FindingKind, location: i32) {
        let span = self.name_span(location);
        self.report_with_span(kind, span);
    }

    pub fn report_with_span(&mut self, kind: FindingKind, span: Option<TextRange>) {
        self.database_only = false;
        // Some expressions are typed more than once, e.g. VALUES rows in INSERT.
        let finding = Finding { kind, span };
        if !self.findings.contains(&finding) {
            self.findings.push(finding);
        }
    }

    /// The span of the (possibly qualified) name starting at `location`.
    pub fn name_span(&self, location: i32) -> Option<TextRange> {
        self.span_to(location, span::reference_end)
    }

    /// The span of the (possibly qualified) type name starting at `location`, with all the words
    /// of multi-word types like `double precision`.
    pub fn type_name_span(&self, location: i32) -> Option<TextRange> {
        self.span_to(location, span::type_name_end)
    }

    /// The span of the `*` or `t.*` starting at `location`.
    pub fn star_span(&self, location: i32) -> Option<TextRange> {
        self.span_to(location, span::star_end)
    }

    /// The span of the operator token starting at `location`.
    pub fn operator_span(&self, location: i32) -> Option<TextRange> {
        self.span_to(location, span::operator_end)
    }

    pub fn cast_span(&self, location: i32) -> Option<TextRange> {
        self.span_to(location, span::cast_end)
    }

    /// The span of the string constant starting at `location`.
    pub fn literal_span(&self, location: i32) -> Option<TextRange> {
        self.span_to(location, span::literal_end)
    }

    /// The span of the first token of an expression starting at `location`.
    pub fn expression_span(&self, location: i32) -> Option<TextRange> {
        self.span_to(location, |sql, start| {
            span::reference_end(sql, start).or_else(|| span::token_end(sql, start))
        })
    }

    fn span_to(
        &self,
        location: i32,
        end_fn: fn(&str, usize) -> Option<usize>,
    ) -> Option<TextRange> {
        let start = usize::try_from(location).ok()?;
        let end = end_fn(self.sql?, start)?;
        Some(TextRange::new(
            TextSize::try_from(start).ok()?,
            TextSize::try_from(end).ok()?,
        ))
    }

    /// Reports an unknown schema. Returns `true` if the schema is known to be missing.
    pub fn check_schema(&mut self, schema: &str, location: i32) -> bool {
        match self.catalog.schema(schema) {
            Lookup::Missing => {
                self.report(
                    FindingKind::UnknownSchema {
                        name: schema.to_owned(),
                    },
                    location,
                );
                true
            }
            Lookup::Unknown => {
                self.depends_on_file();
                false
            }
            Lookup::Found(()) => false,
        }
    }
}
