use crate::context::{ClauseType, CompletionContext};

#[derive(Debug)]
pub(crate) enum CompletionRelevanceData<'a> {
    Table(&'a pglt_schema_cache::Table),
    Function(&'a pglt_schema_cache::Function),
    Column(&'a pglt_schema_cache::Column),
}

impl CompletionRelevanceData<'_> {
    pub fn get_score(self, ctx: &CompletionContext) -> i32 {
        CompletionRelevance::from(self).into_score(ctx)
    }
}

impl<'a> From<CompletionRelevanceData<'a>> for CompletionRelevance<'a> {
    fn from(value: CompletionRelevanceData<'a>) -> Self {
        Self {
            score: 0,
            data: value,
        }
    }
}

#[derive(Debug)]
pub(crate) struct CompletionRelevance<'a> {
    score: i32,
    data: CompletionRelevanceData<'a>,
}

impl CompletionRelevance<'_> {
    pub fn into_score(mut self, ctx: &CompletionContext) -> i32 {
        self.check_matches_schema(ctx);
        self.check_matches_query_input(ctx);
        self.check_if_catalog(ctx);
        self.check_is_invocation(ctx);
        self.check_matching_clause_type(ctx);
        self.check_relations_in_stmt(ctx);

        self.score
    }

    fn check_matches_query_input(&mut self, ctx: &CompletionContext) {
        let node = ctx.ts_node.unwrap();

        let content = match ctx.get_ts_node_content(node) {
            Some(c) => c,
            None => return,
        };

        let name = match self.data {
            CompletionRelevanceData::Function(f) => f.name.as_str(),
            CompletionRelevanceData::Table(t) => t.name.as_str(),
            CompletionRelevanceData::Column(c) => c.name.as_str(),
        };

        if name.starts_with(content) {
            let len: i32 = content
                .len()
                .try_into()
                .expect("The length of the input exceeds i32 capacity");

            self.score += len * 5;
        };
    }

    fn check_matching_clause_type(&mut self, ctx: &CompletionContext) {
        let clause_type = match ctx.wrapping_clause_type.as_ref() {
            None => return,
            Some(ct) => ct,
        };

        let has_mentioned_tables = !ctx.mentioned_relations.is_empty();

        self.score += match self.data {
            CompletionRelevanceData::Table(_) => match clause_type {
                ClauseType::From => 5,
                ClauseType::Update => 15,
                ClauseType::Delete => 15,
                _ => -50,
            },
            CompletionRelevanceData::Function(_) => match clause_type {
                ClauseType::Select if !has_mentioned_tables => 15,
                ClauseType::Select if has_mentioned_tables => 0,
                ClauseType::From => 0,
                _ => -50,
            },
            CompletionRelevanceData::Column(_) => match clause_type {
                ClauseType::Select if has_mentioned_tables => 10,
                ClauseType::Select if !has_mentioned_tables => 0,
                ClauseType::Where => 10,
                _ => -15,
            },
        }
    }

    fn check_is_invocation(&mut self, ctx: &CompletionContext) {
        self.score += match self.data {
            CompletionRelevanceData::Function(_) if ctx.is_invocation => 30,
            CompletionRelevanceData::Function(_) if !ctx.is_invocation => -10,
            _ if ctx.is_invocation => -10,
            _ => 0,
        };
    }

    fn check_matches_schema(&mut self, ctx: &CompletionContext) {
        let schema_name = match ctx.schema_name.as_ref() {
            None => return,
            Some(n) => n,
        };

        let data_schema = self.get_schema_name();

        if schema_name == data_schema {
            self.score += 25;
        } else {
            self.score -= 10;
        }
    }

    fn get_schema_name(&self) -> &str {
        match self.data {
            CompletionRelevanceData::Function(f) => f.schema.as_str(),
            CompletionRelevanceData::Table(t) => t.schema.as_str(),
            CompletionRelevanceData::Column(c) => c.schema_name.as_str(),
        }
    }

    fn get_table_name(&self) -> Option<&str> {
        match self.data {
            CompletionRelevanceData::Column(c) => Some(c.table_name.as_str()),
            CompletionRelevanceData::Table(t) => Some(t.name.as_str()),
            _ => None,
        }
    }

    fn check_if_catalog(&mut self, ctx: &CompletionContext) {
        if ctx.schema_name.as_ref().is_some_and(|n| n == "pg_catalog") {
            return;
        }

        self.score -= 5; // unlikely that the user wants schema data
    }

    fn check_relations_in_stmt(&mut self, ctx: &CompletionContext) {
        match self.data {
            CompletionRelevanceData::Table(_) | CompletionRelevanceData::Function(_) => return,
            _ => {}
        }

        let schema = self.get_schema_name().to_string();
        let table_name = match self.get_table_name() {
            Some(t) => t,
            None => return,
        };

        if ctx
            .mentioned_relations
            .get(&Some(schema.to_string()))
            .is_some_and(|tables| tables.contains(table_name))
        {
            self.score += 45;
        } else if ctx
            .mentioned_relations
            .get(&None)
            .is_some_and(|tables| tables.contains(table_name))
        {
            self.score += 30;
        }
    }
}
