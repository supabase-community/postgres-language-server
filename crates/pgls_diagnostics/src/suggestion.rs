use ::serde::{Deserialize, Serialize};

/// Whether a fix can be applied without reviewing it.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum Applicability {
    /// The fix is safe to apply automatically.
    Always,
    /// The fix may change the behaviour of the code and should be reviewed.
    MaybeIncorrect,
}
