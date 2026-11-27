use bpaf::Bpaf;
use pgls_configuration_macros::{Merge, Partial};
pub use pgls_templating::Template;
use serde::{Deserialize, Serialize};

use crate::StringSet;

/// The configuration of SQL files that are format strings of a host language.
#[derive(Clone, Debug, Default, Deserialize, Eq, Partial, PartialEq, Serialize)]
#[partial(derive(Bpaf, Clone, Eq, PartialEq, Merge))]
#[partial(cfg_attr(feature = "schema", derive(schemars::JsonSchema)))]
#[partial(serde(rename_all = "camelCase", default, deny_unknown_fields))]
pub struct TemplatingConfiguration {
    /// The format strings that the files are: "go-printf" for Go's `fmt.Sprintf` (`%s`, `%d`,
    /// `%[1]v`, `%%`), "rust-fmt" for Rust's `format!` (`{}`, `{0}`, `{name}`, `{{`), or "none".
    /// The analysis reads the SQL around the placeholders. Default: "none".
    #[partial(bpaf(long("template-syntax"), argument("go-printf|rust-fmt|none")))]
    pub syntax: Template,

    /// A list of Unix shell style patterns. Only the files that match them are templates.
    #[partial(bpaf(hide))]
    pub include: StringSet,

    /// A list of Unix shell style patterns. The files that match them are no templates.
    #[partial(bpaf(hide))]
    pub ignore: StringSet,
}

/// Like the `Merge` derive of an enum: the syntax of the other configuration wins.
impl crate::Merge for Template {
    fn merge_with(&mut self, other: Self) {
        *self = other;
    }
}
