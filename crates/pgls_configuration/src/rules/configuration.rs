use biome_deserialize::Merge;
use biome_deserialize_macros::Deserializable;
use pgls_analyser::RuleOptions;
use pgls_diagnostics::Severity;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, untagged)]
pub enum RuleConfiguration<T: Default> {
    Plain(RulePlainConfiguration),
    WithOptions(RuleWithOptions<T>),
}
impl<T: Default> RuleConfiguration<T> {
    pub fn is_disabled(&self) -> bool {
        matches!(self.level(), RulePlainConfiguration::Off)
    }
    pub fn is_enabled(&self) -> bool {
        !self.is_disabled()
    }
    pub fn level(&self) -> RulePlainConfiguration {
        match self {
            Self::Plain(plain) => *plain,
            Self::WithOptions(options) => options.level,
        }
    }
    pub fn set_level(&mut self, level: RulePlainConfiguration) {
        match self {
            Self::Plain(plain) => *plain = level,
            Self::WithOptions(options) => options.level = level,
        }
    }
}
// Rule configuration has a custom [Merge] implementation so that overriding the
// severity doesn't override the options.
impl<T: Clone + Default> Merge for RuleConfiguration<T> {
    fn merge_with(&mut self, other: Self) {
        match self {
            Self::Plain(_) => *self = other,
            Self::WithOptions(this) => match other {
                Self::Plain(level) => {
                    this.level = level;
                }
                Self::WithOptions(other) => {
                    this.merge_with(other);
                }
            },
        }
    }
}
impl<T: Clone + Default + 'static> RuleConfiguration<T> {
    pub fn get_options(&self) -> Option<RuleOptions> {
        match self {
            Self::Plain(_) => None,
            Self::WithOptions(options) => Some(RuleOptions::new(options.options.clone())),
        }
    }
}
impl<T: Default> Default for RuleConfiguration<T> {
    fn default() -> Self {
        Self::Plain(RulePlainConfiguration::Error)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, untagged)]
pub enum RuleFixConfiguration<T: Default> {
    Plain(RulePlainConfiguration),
    WithOptions(RuleWithFixOptions<T>),
}
impl<T: Default> Default for RuleFixConfiguration<T> {
    fn default() -> Self {
        Self::Plain(RulePlainConfiguration::Error)
    }
}
impl<T: Default> RuleFixConfiguration<T> {
    pub fn is_disabled(&self) -> bool {
        matches!(self.level(), RulePlainConfiguration::Off)
    }
    pub fn is_enabled(&self) -> bool {
        !self.is_disabled()
    }
    pub fn level(&self) -> RulePlainConfiguration {
        match self {
            Self::Plain(plain) => *plain,
            Self::WithOptions(options) => options.level,
        }
    }
    pub fn set_level(&mut self, level: RulePlainConfiguration) {
        match self {
            Self::Plain(plain) => *plain = level,
            Self::WithOptions(options) => options.level = level,
        }
    }
}
// Rule configuration has a custom [Merge] implementation so that overriding the
// severity doesn't override the options.
impl<T: Clone + Default> Merge for RuleFixConfiguration<T> {
    fn merge_with(&mut self, other: Self) {
        match self {
            Self::Plain(_) => *self = other,
            Self::WithOptions(this) => match other {
                Self::Plain(level) => {
                    this.level = level;
                }
                Self::WithOptions(other) => {
                    this.merge_with(other);
                }
            },
        }
    }
}
impl<T: Clone + Default + 'static> RuleFixConfiguration<T> {
    pub fn get_options(&self) -> Option<RuleOptions> {
        match self {
            Self::Plain(_) => None,
            Self::WithOptions(options) => Some(RuleOptions::new(options.options.clone())),
        }
    }
}
impl<T: Default> From<&RuleConfiguration<T>> for Severity {
    fn from(conf: &RuleConfiguration<T>) -> Self {
        match conf {
            RuleConfiguration::Plain(p) => (*p).into(),
            RuleConfiguration::WithOptions(conf) => {
                let level = &conf.level;
                (*level).into()
            }
        }
    }
}
impl From<RulePlainConfiguration> for Severity {
    fn from(conf: RulePlainConfiguration) -> Self {
        match conf {
            RulePlainConfiguration::Warn => Severity::Warning,
            RulePlainConfiguration::Error => Severity::Error,
            RulePlainConfiguration::Info => Severity::Information,
            RulePlainConfiguration::Off => {
                unreachable!("the rule is turned off, it should not step in here")
            }
        }
    }
}
impl From<RuleAssistPlainConfiguration> for Severity {
    fn from(conf: RuleAssistPlainConfiguration) -> Self {
        match conf {
            RuleAssistPlainConfiguration::On => Severity::Hint,
            RuleAssistPlainConfiguration::Off => {
                unreachable!("the rule is turned off, it should not step in here")
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Deserializable, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum RulePlainConfiguration {
    #[default]
    Warn,
    Error,
    Info,
    Off,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, untagged)]
pub enum RuleAssistConfiguration<T: Default> {
    Plain(RuleAssistPlainConfiguration),
    WithOptions(RuleAssistWithOptions<T>),
}
impl<T: Default> RuleAssistConfiguration<T> {
    pub fn is_disabled(&self) -> bool {
        matches!(self.level(), RuleAssistPlainConfiguration::Off)
    }
    pub fn is_enabled(&self) -> bool {
        !self.is_disabled()
    }
    pub fn level(&self) -> RuleAssistPlainConfiguration {
        match self {
            Self::Plain(plain) => *plain,
            Self::WithOptions(options) => options.level,
        }
    }
    pub fn set_level(&mut self, level: RuleAssistPlainConfiguration) {
        match self {
            Self::Plain(plain) => *plain = level,
            Self::WithOptions(options) => options.level = level,
        }
    }
}
// Rule configuration has a custom [Merge] implementation so that overriding the
// severity doesn't override the options.
impl<T: Clone + Default> Merge for RuleAssistConfiguration<T> {
    fn merge_with(&mut self, other: Self) {
        match self {
            Self::Plain(_) => *self = other,
            Self::WithOptions(this) => match other {
                Self::Plain(level) => {
                    this.level = level;
                }
                Self::WithOptions(other) => {
                    this.merge_with(other);
                }
            },
        }
    }
}
impl<T: Clone + Default + 'static> RuleAssistConfiguration<T> {
    pub fn get_options(&self) -> Option<RuleOptions> {
        match self {
            Self::Plain(_) => None,
            Self::WithOptions(options) => Some(RuleOptions::new(options.options.clone())),
        }
    }
}
impl<T: Default> Default for RuleAssistConfiguration<T> {
    fn default() -> Self {
        Self::Plain(RuleAssistPlainConfiguration::Off)
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Deserializable, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum RuleAssistPlainConfiguration {
    #[default]
    On,
    Off,
}
impl RuleAssistPlainConfiguration {
    pub const fn is_enabled(&self) -> bool {
        matches!(self, Self::On)
    }

    pub const fn is_disabled(&self) -> bool {
        matches!(self, Self::Off)
    }
}
impl Merge for RuleAssistPlainConfiguration {
    fn merge_with(&mut self, other: Self) {
        *self = other;
    }
}

#[derive(Clone, Debug, Default, Deserialize, Deserializable, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuleAssistWithOptions<T: Default> {
    /// The severity of the emitted diagnostics by the rule
    pub level: RuleAssistPlainConfiguration,
    /// Rule's options
    pub options: T,
}
impl<T: Default> Merge for RuleAssistWithOptions<T> {
    fn merge_with(&mut self, other: Self) {
        self.level = other.level;
        self.options = other.options;
    }
}

#[derive(Clone, Debug, Default, Deserialize, Deserializable, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuleWithOptions<T: Default> {
    /// The severity of the emitted diagnostics by the rule
    pub level: RulePlainConfiguration,
    /// Rule's options
    pub options: T,
}
impl<T: Default> Merge for RuleWithOptions<T> {
    fn merge_with(&mut self, other: Self) {
        self.level = other.level;
        self.options = other.options;
    }
}

#[derive(Clone, Debug, Default, Deserialize, Deserializable, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuleWithFixOptions<T: Default> {
    /// The severity of the emitted diagnostics by the rule
    pub level: RulePlainConfiguration,
    /// Rule's options
    pub options: T,
}

impl<T: Default> Merge for RuleWithFixOptions<T> {
    fn merge_with(&mut self, other: Self) {
        self.level = other.level;
        self.options = other.options;
    }
}
