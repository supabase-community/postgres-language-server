//! The state of name resolution: the catalog, the names in scope, and the findings so far.

use pgls_catalog::Catalog;
use std::{cell::OnceCell, mem::Discriminant};

use pgls_text_size::{TextRange, TextSize};
use rustc_hash::{FxHashMap, FxHashSet};

use pgls_query::{
    NodeEnum,
    protobuf::{self, CoercionForm, FuncCall},
};

use super::{
    Finding, FindingKind, FunctionContext, FunctionParam, Reference, Resolution,
    ResolvedCallArgument,
    column_name::target_name_with,
    lookups::{LookupKind, SearchPathLookup},
    output::QueryOutput,
    scope::{Cte, Level, OutputNames},
    span::Tokens,
};
use pgls_catalog::TypeKind;
use pgls_catalog::typing::{ResolvedFunction, Type, TypeId};
use pgls_catalog::{Lookup, Origin};

pub(super) struct Resolver<'a> {
    pub catalog: &'a Catalog,
    pub search_path: &'a [String],
    pub function: Option<&'a FunctionContext>,
    sql: Option<&'a str>,
    /// The tokens of `sql`, scanned when the first span is needed.
    tokens: OnceCell<Option<Tokens<'a>>>,
    findings: Vec<Finding>,
    /// Cleared as soon as the statement references something that doesn't come unchanged from
    /// the database, or something the resolver doesn't understand.
    database_only: bool,
    /// The FROM items of the enclosing query levels, innermost last. See
    /// [`Resolver::in_level`].
    levels: Vec<Level>,
    /// The common table expressions in scope, innermost last. See [`Resolver::with_ctes`].
    ctes: Vec<Cte>,
    /// The references resolved so far, when they are being collected.
    references: Option<Vec<Reference>>,
    /// The location and kind of each reference in `references`.
    recorded: FxHashSet<(i32, Discriminant<Reference>)>,
    /// The names looked up through the search path so far, when they are being collected.
    lookups: Option<Vec<SearchPathLookup>>,
    /// The location and kind of each lookup in `lookups`.
    recorded_lookups: FxHashSet<(i32, LookupKind)>,
    /// What the clause being resolved allows. See [`Resolver::with_context`].
    context: Context,
    /// How many queries and data-modifying statements the resolver is in. See
    /// [`Resolver::in_query`].
    query_depth: usize,
    /// The output columns of the subqueries of sublinks resolved so far, by the address of the
    /// subquery's node, so that typing a sublink doesn't resolve its subquery again. Resolving
    /// it again at every level of nested subqueries took time exponential in their depth.
    subquery_outputs: FxHashMap<usize, Option<QueryOutput>>,
}

/// What the clause being resolved allows, including in its subqueries.
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct Context {
    /// Set while resolving a clause whose unqualified names can only be columns of one relation,
    /// like the index predicate of `ON CONFLICT`, so they are not recorded as column lookups.
    pub single_relation: bool,
    /// Set while resolving the RETURNING list of MERGE, including its subqueries, where
    /// `merge_action()` is allowed.
    pub merge_returning: bool,
}

impl<'a> Resolver<'a> {
    pub fn new(
        catalog: &'a Catalog,
        search_path: &'a [String],
        function: Option<&'a FunctionContext>,
        sql: Option<&'a str>,
    ) -> Self {
        Self {
            catalog,
            search_path,
            function,
            sql,
            tokens: OnceCell::new(),
            findings: Vec::new(),
            database_only: true,
            levels: Vec::new(),
            ctes: Vec::new(),
            references: None,
            recorded: FxHashSet::default(),
            lookups: None,
            recorded_lookups: FxHashSet::default(),
            context: Context::default(),
            query_depth: 0,
            subquery_outputs: FxHashMap::default(),
        }
    }

    /// Collects the references and the search path lookups of the statement too.
    pub fn collecting(mut self, references: bool, lookups: bool) -> Self {
        self.references = references.then(Vec::new);
        self.lookups = lookups.then(Vec::new);
        self
    }

    /// The resolution of a statement: what the resolver found, with the output columns of its
    /// query. `checks_database` is whether the database can check the statement at all.
    pub fn into_resolution(self, output: Option<QueryOutput>, checks_database: bool) -> Resolution {
        let mut lookups = self.lookups.unwrap_or_default();
        lookups.sort_by_key(|lookup| lookup.location);

        Resolution {
            // `Resolver::report_with_span` keeps each finding once.
            findings: self.findings,
            database_only: checks_database && self.database_only,
            references: self.references.unwrap_or_default(),
            lookups,
            output,
        }
    }

    /// Whether the resolver found anything Postgres would reject.
    pub fn has_findings(&self) -> bool {
        !self.findings.is_empty()
    }

    /// Runs `f`, and drops the findings it reports.
    pub fn without_findings<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let findings = self.findings.len();
        let result = f(self);
        self.findings.truncate(findings);
        result
    }

    /// Remembers the output columns of `query`, the subquery of a sublink.
    pub fn remember_subquery_output(
        &mut self,
        query: &protobuf::Node,
        output: Option<QueryOutput>,
    ) {
        self.subquery_outputs
            .insert(std::ptr::from_ref(query) as usize, output);
    }

    /// The output columns of `query`, the subquery of a sublink, if it was resolved before.
    pub fn subquery_output(&self, query: &protobuf::Node) -> Option<&Option<QueryOutput>> {
        self.subquery_outputs
            .get(&(std::ptr::from_ref(query) as usize))
    }

    /// The FROM items of the enclosing query levels, innermost last.
    pub fn levels(&self) -> &[Level] {
        &self.levels
    }

    /// Runs `f` with the FROM items of `level` visible, innermost, and returns its result and
    /// the level, with what `f` changed in it.
    pub fn in_level<T>(&mut self, level: Level, f: impl FnOnce(&mut Self) -> T) -> (T, Level) {
        let depth = self.levels.len();
        self.levels.push(level);
        let result = f(self);
        // Only this method pushes and pops levels, so `f` leaves them as it found them.
        debug_assert_eq!(self.levels.len(), depth + 1, "unbalanced query levels");
        let level = self.levels.pop().unwrap_or_default();
        (result, level)
    }

    /// The common table expressions in scope, innermost last.
    pub fn ctes(&self) -> &[Cte] {
        &self.ctes
    }

    /// Runs `f`, and forgets the common table expressions it defines when it returns, like the
    /// end of the statement whose `WITH` defines them.
    pub fn with_ctes<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let ctes = self.ctes.len();
        let result = f(self);
        self.ctes.truncate(ctes);
        result
    }

    /// Makes `cte` visible, innermost.
    pub fn define_cte(&mut self, cte: Cte) {
        self.ctes.push(cte);
    }

    /// The innermost visible common table expression with this name.
    pub fn cte_mut(&mut self, name: &str) -> Option<&mut Cte> {
        self.ctes.iter_mut().rev().find(|cte| cte.name == name)
    }

    /// Runs `f` inside a query or data-modifying statement.
    pub fn in_query<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        self.query_depth += 1;
        let result = f(self);
        self.query_depth -= 1;
        result
    }

    /// Whether the resolver is inside a query or data-modifying statement.
    pub fn in_any_query(&self) -> bool {
        self.query_depth > 0
    }

    /// What the clause being resolved allows.
    pub fn context(&self) -> Context {
        self.context
    }

    /// Runs `f` with `context`, and restores the one before when it returns.
    pub fn with_context<T>(&mut self, context: Context, f: impl FnOnce(&mut Self) -> T) -> T {
        let outer = std::mem::replace(&mut self.context, context);
        let result = f(self);
        self.context = outer;
        result
    }

    /// Names the output columns of the innermost level's select list, which `ORDER BY` and
    /// `GROUP BY` can reference by bare name.
    pub fn set_output_names(&mut self, names: Vec<String>) {
        if let Some(level) = self.levels.last_mut() {
            level.output_names = names;
        }
    }

    /// Whether bare column names can refer to the output columns of the innermost level, as in
    /// `ORDER BY` and `GROUP BY`.
    pub fn set_output_names_visible(&mut self, visible: OutputNames) {
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

    /// The name of a select list entry: its alias, or else the name Postgres chooses for it
    /// ([`FigureColname`]). Scalar subqueries the resolver has resolved are named after their
    /// first output column, which is also known when it is a `*`, as in `(select * from t)`.
    ///
    /// [`FigureColname`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1713
    pub fn target_name(&self, target: &protobuf::ResTarget) -> Option<String> {
        target_name_with(target, &|link| {
            let output = self.subquery_output(link.subselect.as_deref()?)?;
            Some(output.as_ref()?.first_column()?.name.clone())
        })
    }

    /// The statement text, if known.
    pub fn sql(&self) -> Option<&'a str> {
        self.sql
    }

    /// The tokens of the statement. It is scanned once, the first time they are needed, which for
    /// most statements is never: spans are only needed for findings and references.
    pub fn tokens(&self) -> Option<&Tokens<'a>> {
        self.tokens
            .get_or_init(|| self.sql.and_then(Tokens::scan))
            .as_ref()
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

    /// Whether the names looked up through the search path are being collected.
    pub fn collects_lookups(&self) -> bool {
        self.lookups.is_some()
    }

    /// Records a name looked up through the search path, when lookups are being collected.
    pub fn record_lookup(
        &mut self,
        kind: LookupKind,
        name: &str,
        location: i32,
        span: Option<TextRange>,
    ) {
        let Some(lookups) = self.lookups.as_mut() else {
            return;
        };
        // Some expressions are resolved more than once, e.g. VALUES rows in INSERT.
        if self.recorded_lookups.insert((location, kind)) {
            lookups.push(SearchPathLookup {
                kind,
                name: name.to_owned(),
                location,
                span,
            });
        }
    }

    /// Records that `call` resolves to `resolved`, when references are being collected.
    pub fn record_call(&mut self, call: &FuncCall, resolved: &ResolvedFunction) {
        let function = &resolved.function;
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
        // The type each argument is coerced to, with polymorphic parameters like `anyelement`
        // resolved from the other arguments.
        for (arg, target) in call.args.iter().zip(&resolved.arguments) {
            let value = match arg.node.as_ref() {
                Some(NodeEnum::NamedArgExpr(named)) => {
                    named.arg.as_deref().and_then(|arg| arg.node.as_ref())
                }
                value => value,
            };
            if let Some(Type::Named(target)) = target {
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
        self.span_to(location, Tokens::name_end)
    }

    /// The span of the (possibly qualified) type name starting at `location`, with all the words
    /// of multi-word types like `double precision`.
    pub fn type_name_span(&self, location: i32) -> Option<TextRange> {
        self.span_to(location, Tokens::type_name_end)
    }

    /// The span of the `*` or `t.*` starting at `location`.
    pub fn star_span(&self, location: i32) -> Option<TextRange> {
        self.span_to(location, Tokens::star_end)
    }

    /// The span of the select list entry starting at `location`, with its alias.
    pub fn target_span(&self, location: i32) -> Option<TextRange> {
        self.span_to(location, Tokens::target_end)
    }

    /// The span of the token starting at `location`.
    pub fn token_span(&self, location: i32) -> Option<TextRange> {
        self.span_to(location, Tokens::token_end)
    }

    /// The span of the keywords of the SQL construct starting at `location` that end with `last`,
    /// like `NOT IN`.
    pub fn construct_span(
        &self,
        location: i32,
        last: pgls_query::protobuf::Token,
    ) -> Option<TextRange> {
        let start = usize::try_from(location).ok()?;
        let end = self.tokens()?.construct_end(start, last)?;
        debug_assert!(
            self.sql
                .is_some_and(|sql| start < end && sql.get(start..end).is_some()),
            "span {start}..{end} outside the statement or not on character boundaries"
        );
        Some(TextRange::new(
            TextSize::try_from(start).ok()?,
            TextSize::try_from(end).ok()?,
        ))
    }

    /// The span of the operator token starting at `location`.
    pub fn operator_span(&self, location: i32) -> Option<TextRange> {
        self.span_to(location, Tokens::operator_end)
    }

    pub fn cast_span(&self, location: i32) -> Option<TextRange> {
        self.span_to(location, Tokens::cast_end)
    }

    /// The span of the string constant starting at `location`.
    pub fn literal_span(&self, location: i32) -> Option<TextRange> {
        self.span_to(location, Tokens::literal_end)
    }

    /// The alias `alias` after the (possibly qualified) name starting at `location`: its location
    /// and span, if it is written there.
    pub fn alias_location(&self, location: i32, alias: &str) -> Option<(i32, TextRange)> {
        let sql = self.sql?;
        let (start, end) = self.tokens()?.alias_span(usize::try_from(location).ok()?)?;
        if pgls_catalog::identifier_name(sql.get(start..end)?) != alias {
            return None;
        }
        Some((
            i32::try_from(start).ok()?,
            TextRange::new(
                TextSize::try_from(start).ok()?,
                TextSize::try_from(end).ok()?,
            ),
        ))
    }

    /// Records the alias of the FROM item named at `location`, when references are being
    /// collected.
    pub fn record_alias(&mut self, location: i32, alias: Option<&pgls_query::protobuf::Alias>) {
        if let Some(alias) = alias {
            self.record_alias_name(location, &alias.aliasname);
        }
    }

    /// Records the alias `name` that follows the name at `location`, like `o` in `OLD AS o`,
    /// when references are being collected.
    pub fn record_alias_name(&mut self, location: i32, name: &str) {
        if !self.collects_references() {
            return;
        }
        if let Some((location, span)) = self.alias_location(location, name) {
            self.record(Reference::Alias {
                location,
                span: Some(span),
                name: name.to_owned(),
            });
        }
    }

    /// The span of the first token of an expression starting at `location`.
    pub fn expression_span(&self, location: i32) -> Option<TextRange> {
        self.span_to(location, Tokens::expression_end)
    }

    fn span_to(
        &self,
        location: i32,
        end_fn: fn(&Tokens<'a>, usize) -> Option<usize>,
    ) -> Option<TextRange> {
        let start = usize::try_from(location).ok()?;
        let end = end_fn(self.tokens()?, start)?;
        debug_assert!(
            self.sql
                .is_some_and(|sql| start < end && sql.get(start..end).is_some()),
            "span {start}..{end} outside the statement or not on character boundaries"
        );
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
