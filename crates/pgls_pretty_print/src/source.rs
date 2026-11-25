use pgls_query::protobuf::ScanToken;

/// The text of a statement and its tokens, scanned once per formatting pass. Comment attachment
/// and the truncated identifiers read the same tokens.
pub(crate) struct Source<'a> {
    pub sql: &'a str,
    /// libpg_query's tokens of `sql`, comments included, in source order.
    pub tokens: Vec<ScanToken>,
}

impl<'a> Source<'a> {
    /// Scans `sql`. Text libpg_query can't scan has no tokens, so it has no comments and no
    /// truncated identifiers.
    pub(crate) fn scan(sql: &'a str) -> Self {
        let tokens = pgls_query::scan(sql)
            .map(|scan| scan.tokens)
            .unwrap_or_default();
        Self { sql, tokens }
    }
}
