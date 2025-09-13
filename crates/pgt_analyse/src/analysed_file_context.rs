pub struct AnalysedFileContext<'a> {
    pub all_stmts: &'a Vec<pgt_query::NodeEnum>,
    pub stmt_count: usize,
    pub previous_stmts: Vec<&'a pgt_query::NodeEnum>,
}

impl<'a> AnalysedFileContext<'a> {
    pub fn new(stmts: &'a Vec<pgt_query::NodeEnum>) -> Self {
        Self {
            all_stmts: stmts,
            stmt_count: stmts.len(),
            previous_stmts: Vec::new(),
        }
    }

    pub fn update_from(&mut self, stmt_root: &'a pgt_query::NodeEnum) {
        self.previous_stmts.push(stmt_root);
    }
}
