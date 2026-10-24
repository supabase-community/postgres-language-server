use std::str::FromStr;

use bpaf::Bpaf;
use pgls_configuration_macros::{Merge, Partial};
use serde::{Deserialize, Serialize};

/// The configuration for inlay hints.
#[derive(Clone, Debug, Deserialize, Eq, Partial, PartialEq, Serialize)]
#[partial(derive(Bpaf, Clone, Eq, PartialEq, Merge))]
#[partial(cfg_attr(feature = "schema", derive(schemars::JsonSchema)))]
#[partial(serde(rename_all = "camelCase", default, deny_unknown_fields))]
pub struct InlayHintsConfiguration {
    /// if `false`, the language server sends no inlay hints. `true` by default
    #[partial(bpaf(hide))]
    pub enabled: bool,

    /// Which arguments of function calls show the name of their parameter: "all", "literals"
    /// (only constants like `1`, `'a'`, `true` or `null`) or "none". Default: "all".
    #[partial(bpaf(hide))]
    pub parameter_names: ParameterNameHints,
}

impl Default for InlayHintsConfiguration {
    fn default() -> Self {
        Self {
            enabled: true,
            parameter_names: ParameterNameHints::default(),
        }
    }
}

/// Which arguments of function calls show the name of their parameter.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Merge, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum ParameterNameHints {
    /// Every positional argument (default).
    #[default]
    All,
    /// Only constants, like `1`, `'a'`, `true` or `null`.
    Literals,
    /// No argument.
    None,
}

impl FromStr for ParameterNameHints {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "all" => Ok(Self::All),
            "literals" => Ok(Self::Literals),
            "none" => Ok(Self::None),
            _ => {
                Err("Value not supported for ParameterNameHints. Use 'all', 'literals' or 'none'.")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_parameter_names() {
        let config: PartialInlayHintsConfiguration =
            serde_json::from_str(r#"{ "parameterNames": "literals" }"#)
                .expect("inlay hints configuration should deserialize");
        assert_eq!(config.parameter_names, Some(ParameterNameHints::Literals));
        let config = InlayHintsConfiguration::from(PartialInlayHintsConfiguration::default());
        assert_eq!(config.parameter_names, ParameterNameHints::All);
        assert!(
            serde_json::from_str::<PartialInlayHintsConfiguration>(
                r#"{ "parameterNames": "some" }"#
            )
            .is_err()
        );
    }
}
