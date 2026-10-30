use pgls_fs::PgLSPath;
use pgls_text_size::TextRange;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PullSemanticTokensParams {
    pub project_key: crate::workspace::ProjectKey,
    pub path: PgLSPath,
    /// The tokens of the statements that overlap this range are returned. `None` for the whole
    /// file.
    pub range: Option<TextRange>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PullSemanticTokensResult {
    /// The tokens, ordered by their start. Tokens don't overlap.
    pub tokens: Vec<SemanticToken>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SemanticToken {
    /// The range in the file.
    pub range: TextRange,
    pub kind: SemanticTokenKind,
    /// Whether the object is built into Postgres: it is in `pg_catalog` or `information_schema`.
    pub default_library: bool,
}

/// What a token refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum SemanticTokenKind {
    /// A schema.
    Namespace,
    /// A table, view, or other relation, or a common table expression.
    Class,
    /// A column.
    Property,
    Function,
    Type,
    /// A parameter of a SQL function, in its body.
    Parameter,
    /// A string literal that is coerced to an enum type.
    EnumMember,
    /// The alias of a FROM item, or a role.
    Variable,
}

impl From<pgls_semantic_tokens::SemanticTokenKind> for SemanticTokenKind {
    fn from(kind: pgls_semantic_tokens::SemanticTokenKind) -> Self {
        use pgls_semantic_tokens::SemanticTokenKind as Kind;
        match kind {
            Kind::Namespace => Self::Namespace,
            Kind::Class => Self::Class,
            Kind::Property => Self::Property,
            Kind::Function => Self::Function,
            Kind::Type => Self::Type,
            Kind::Parameter => Self::Parameter,
            Kind::EnumMember => Self::EnumMember,
            Kind::Variable => Self::Variable,
        }
    }
}
