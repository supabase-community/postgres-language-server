use bpaf::Bpaf;
use pgls_configuration_macros::{Merge, Partial};
use serde::{Deserialize, Serialize};

/// The configuration for plpgsql_check.
#[derive(Clone, Debug, Deserialize, Eq, Partial, PartialEq, Serialize)]
#[partial(derive(Bpaf, Clone, Eq, PartialEq, Merge))]
#[partial(cfg_attr(feature = "schema", derive(schemars::JsonSchema)))]
#[partial(serde(rename_all = "camelCase", default, deny_unknown_fields))]
pub struct PlPgSqlCheckConfiguration {
    /// if `false`, it disables the feature and pglpgsql_check won't be executed. `true` by default
    #[partial(bpaf(hide))]
    pub enabled: bool,

    /// Stop processing at the first error. `true` by default.
    #[partial(bpaf(hide))]
    pub fatal_errors: bool,

    /// Show warnings about attribute count mismatches, variable overlaps, unused variables, and
    /// unwanted casting. `true` by default.
    #[partial(bpaf(hide))]
    pub other_warnings: bool,

    /// Show warnings regarding missing RETURN statements, shadowed variables, dead code, and
    /// unused parameters. `true` by default.
    #[partial(bpaf(hide))]
    pub extra_warnings: bool,

    /// Flag performance issues like declared types with modifiers and implicit casts that may
    /// prevent index usage. `false` by default.
    #[partial(bpaf(hide))]
    pub performance_warnings: bool,

    /// Identify potential SQL injection vulnerabilities in dynamic statements. `false` by default.
    #[partial(bpaf(hide))]
    pub security_warnings: bool,

    /// Detect deprecated patterns like explicit cursor name assignments in refcursor variables.
    /// `false` by default.
    #[partial(bpaf(hide))]
    pub compatibility_warnings: bool,

    /// Disable all warnings, overriding individual warning parameters. `false` by default.
    #[partial(bpaf(hide))]
    pub without_warnings: bool,

    /// Enable all warnings, overriding individual warning parameters. `false` by default.
    #[partial(bpaf(hide))]
    pub all_warnings: bool,

    /// Activate in-comment options embedded in function source code. `true` by default.
    #[partial(bpaf(hide))]
    pub use_incomment_options: bool,

    /// Raise warnings when in-comment options are utilized. `false` by default.
    #[partial(bpaf(hide))]
    pub incomment_options_usage_warning: bool,

    /// Permit variables holding constant values to be used like constants. `true` by default.
    #[partial(bpaf(hide))]
    pub constant_tracing: bool,
}

impl Default for PlPgSqlCheckConfiguration {
    fn default() -> Self {
        Self {
            enabled: true,
            fatal_errors: true,
            other_warnings: true,
            extra_warnings: true,
            performance_warnings: false,
            security_warnings: false,
            compatibility_warnings: false,
            without_warnings: false,
            all_warnings: false,
            use_incomment_options: true,
            incomment_options_usage_warning: false,
            constant_tracing: true,
        }
    }
}
