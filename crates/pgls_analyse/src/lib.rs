mod categories;
mod filter;
pub mod macros;
mod metadata;
mod registry;

// Re-exported for use in the `declare_group` macro
pub use pgls_diagnostics::category_concat;

pub use pgls_diagnostics::Applicability;

pub use crate::categories::{RuleCategories, RuleCategoriesBuilder, RuleCategory};
pub use crate::filter::{AnalysisFilter, RuleFilter, RuleKey};
pub use crate::metadata::{
    AppliesTo, FixKind, GroupCategory, RuleGroup, RuleMeta, RuleMetadata, RuleSource,
};
pub use crate::registry::{MetadataRegistry, RegistryVisitor};
