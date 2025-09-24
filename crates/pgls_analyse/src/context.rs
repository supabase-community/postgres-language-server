use pgls_schema_cache::SchemaCache;

use crate::{
    AnalysedFileContext,
    categories::RuleCategory,
    rule::{GroupCategory, Rule, RuleGroup, RuleMetadata},
};

pub struct RuleContext<'a, R: Rule> {
    stmt: &'a pgls_query::NodeEnum,
    options: &'a R::Options,
    schema_cache: Option<&'a SchemaCache>,
    file_context: &'a AnalysedFileContext<'a>,
}

impl<'a, R> RuleContext<'a, R>
where
    R: Rule + Sized + 'static,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        stmt: &'a pgls_query::NodeEnum,
        options: &'a R::Options,
        schema_cache: Option<&'a SchemaCache>,
        file_context: &'a AnalysedFileContext,
    ) -> Self {
        Self {
            stmt,
            options,
            schema_cache,
            file_context,
        }
    }

    /// Returns the group that belongs to the current rule
    pub fn group(&self) -> &'static str {
        <R::Group as RuleGroup>::NAME
    }

    /// Returns the category that belongs to the current rule
    pub fn category(&self) -> RuleCategory {
        <<R::Group as RuleGroup>::Category as GroupCategory>::CATEGORY
    }

    /// Returns the AST root
    pub fn stmt(&self) -> &pgls_query::NodeEnum {
        self.stmt
    }

    pub fn file_context(&self) -> &AnalysedFileContext {
        self.file_context
    }

    pub fn schema_cache(&self) -> Option<&SchemaCache> {
        self.schema_cache
    }

    /// Returns the metadata of the rule
    ///
    /// The metadata contains information about the rule, such as the name, version, language, and whether it is recommended.
    ///
    /// ## Examples
    /// ```rust,ignore
    /// declare_lint_rule! {
    ///     /// Some doc
    ///     pub(crate) Foo {
    ///         version: "0.0.0",
    ///         name: "foo",
    ///         recommended: true,
    ///     }
    /// }
    ///
    /// impl Rule for Foo {
    ///     const CATEGORY: RuleCategory = RuleCategory::Lint;
    ///     type State = ();
    ///     type Signals = ();
    ///     type Options = ();
    ///
    ///     fn run(ctx: &RuleContext<Self>) -> Self::Signals {
    ///         assert_eq!(ctx.metadata().name, "foo");
    ///     }
    /// }
    /// ```
    pub fn metadata(&self) -> &RuleMetadata {
        &R::METADATA
    }

    /// It retrieves the options that belong to a rule, if they exist.
    ///
    /// In order to retrieve a typed data structure, you have to create a deserializable
    /// data structure and define it inside the generic type `type Options` of the [Rule]
    ///
    pub fn options(&self) -> &R::Options {
        self.options
    }
}
