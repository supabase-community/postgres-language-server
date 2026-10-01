//! The state of name resolution: the catalog, the names in scope, and the findings so far.

use pgls_text_size::{TextRange, TextSize};

use super::{
    Finding, FindingKind, FunctionContext, FunctionParam,
    scope::{Cte, Level},
    span,
};
use crate::lookup::{CatalogView, Lookup, Origin};

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
        }
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
        self.function?
            .params
            .iter()
            .find(|param| param.name.as_deref() == Some(name))
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
        let span = self.span(location);
        self.database_only = false;
        self.findings.push(Finding { kind, span });
    }

    /// The span of the (possibly qualified) name starting at `location`.
    fn span(&self, location: i32) -> Option<TextRange> {
        let start = usize::try_from(location).ok()?;
        let sql = self.sql?;
        let end = span::reference_end(sql, start)?;
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
