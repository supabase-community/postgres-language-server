mod analysed_file_context;
mod categories;
pub mod context;
mod filter;
pub mod macros;
pub mod options;
mod registry;
mod rule;

// Re-exported for use in the `declare_group` macro
pub use pgls_diagnostics::category_concat;

pub use crate::analysed_file_context::AnalysedFileContext;
pub use crate::categories::{
    ActionCategory, RefactorKind, RuleCategories, RuleCategoriesBuilder, RuleCategory,
    SUPPRESSION_ACTION_CATEGORY, SourceActionKind,
};
pub use crate::filter::{AnalysisFilter, GroupKey, RuleFilter, RuleKey};
pub use crate::options::{AnalyserOptions, AnalyserRules};
pub use crate::registry::{
    MetadataRegistry, RegistryRuleParams, RegistryVisitor, RuleRegistry, RuleRegistryBuilder,
};
pub use crate::rule::{
    GroupCategory, Rule, RuleDiagnostic, RuleGroup, RuleMeta, RuleMetadata, RuleSource,
};
