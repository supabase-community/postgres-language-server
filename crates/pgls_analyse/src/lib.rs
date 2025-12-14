mod categories;
mod filter;
pub mod macros;
mod metadata;
mod registry;

// Re-exported for use in the `declare_group` macro
pub use pgls_diagnostics::category_concat;

pub use crate::categories::{
    ActionCategory, RefactorKind, RuleCategories, RuleCategoriesBuilder, RuleCategory,
    SUPPRESSION_ACTION_CATEGORY, SourceActionKind,
};
pub use crate::filter::{AnalysisFilter, GroupKey, RuleFilter, RuleKey};
pub use crate::metadata::{GroupCategory, RuleGroup, RuleMeta, RuleMetadata, RuleSource};
pub use crate::registry::{MetadataRegistry, RegistryVisitor};
