pub struct AnalysedFileContext<'a> {
    pub stmts: &'a Vec<pgt_query::NodeEnum>,

    pos: usize,
}

impl<'a> AnalysedFileContext<'a> {
    pub fn new(stmts: &'a Vec<pgt_query::NodeEnum>) -> Self {
        Self { stmts, pos: 0 }
    }

    pub fn previous_stmts(&self) -> &[pgt_query::NodeEnum] {
        &self.stmts[0..self.pos]
    }

    pub fn stmt_count(&self) -> usize {
        self.stmts.len()
    }

    pub fn next(&mut self) {
        self.pos += 1;
    }
}
