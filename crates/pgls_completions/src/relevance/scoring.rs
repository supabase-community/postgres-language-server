use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};

use pgls_treesitter::context::{TreesitterContext, WrappingClause, WrappingNode};

use crate::sanitization;

use super::CompletionRelevanceData;

#[derive(Debug)]
pub(crate) struct CompletionScore<'a> {
    score: i32,
    skip: bool,
    data: CompletionRelevanceData<'a>,
}

impl<'a> From<CompletionRelevanceData<'a>> for CompletionScore<'a> {
    fn from(value: CompletionRelevanceData<'a>) -> Self {
        Self {
            score: 0,
            skip: false,
            data: value,
        }
    }
}

impl CompletionScore<'_> {
    pub fn should_skip(&self) -> bool {
        self.skip
    }

    pub fn get_score(&self) -> i32 {
        self.score
    }

    pub fn calc_score(&mut self, ctx: &TreesitterContext) {
        let case = (matches!(self.data, CompletionRelevanceData::Schema(_))
            && self.get_schema_name().is_some_and(|s| s == "pg_catalog"))
            || (matches!(self.data, CompletionRelevanceData::Table(_))
                && self.get_table_name().is_some_and(|s| s == "parameters"));

        if case {
            println!("checking {}", self.get_fully_qualified_name())
        }

        self.check_is_user_defined();
        if case {
            println!("{} after user-defined check", self.score);
        }

        self.check_matches_schema(ctx);
        if case {
            println!("{} after schema match check", self.score);
        }
        self.check_matches_query_input(ctx);
        if case {
            println!("{} after query input match check", self.score);
        }
        self.check_is_invocation(ctx);
        if case {
            println!("{} after invocation check", self.score);
        }
        self.check_matching_clause_type(ctx);
        if case {
            println!("{} after clause type check", self.score);
        }
        self.check_without_content(ctx);
        if case {
            println!("{} after wrapping node check", self.score);
        }
        self.check_relations_in_stmt(ctx);
        if case {
            println!("{} after relations in stmt check", self.score);
        }
        self.check_columns_in_stmt(ctx);
        if case {
            println!("{} after columns in stmt check", self.score);
        }
        self.check_is_not_wellknown_migration(ctx);
        if case {
            println!("{} after well-known migration check", self.score);
        }
    }

    fn check_matches_query_input(&mut self, ctx: &TreesitterContext) {
        let content = match ctx.get_node_under_cursor_content() {
            Some(c) if !sanitization::is_sanitized_token(c.as_str()) => c.replace('"', ""),
            _ => {
                return;
            }
        };

        let name = match self.data {
            CompletionRelevanceData::Function(f) => f.name.as_str().to_ascii_lowercase(),
            CompletionRelevanceData::Table(t) => t.name.as_str().to_ascii_lowercase(),
            CompletionRelevanceData::Column(c) => c.name.as_str().to_ascii_lowercase(),
            CompletionRelevanceData::Schema(s) => s.name.as_str().to_ascii_lowercase(),
            CompletionRelevanceData::Policy(p) => p.name.as_str().to_ascii_lowercase(),
            CompletionRelevanceData::Role(r) => r.name.as_str().to_ascii_lowercase(),
        };

        let fz_matcher = SkimMatcherV2::default();

        let check_against = match &ctx.identifier_qualifiers {
            // If both qualifiers are already written out, we must check the item's name itself.
            (Some(_), Some(_)) => name.clone(),

            // If only one qualifier is written out, we might look at a schema, a table, or an alias.
            (None, Some(qualifier)) => {
                if self.get_schema_name().is_some_and(|s| s == qualifier) {
                    self.get_table_name()
                        .map(|t| format!("{}.{}", t, name))
                        .unwrap_or(name.clone())
                } else if self.get_table_name().is_some_and(|t| t == qualifier) {
                    name.clone()
                } else if ctx
                    .get_mentioned_table_for_alias(&qualifier)
                    .is_some_and(|alias_tbl| {
                        self.get_table_name()
                            .is_some_and(|item_tbl| alias_tbl == item_tbl)
                    })
                {
                    name.clone()
                } else {
                    // the qualifier does not match schema, table, or alias.
                    // what the hell is it?
                    // probably a typo.
                    self.skip = true;
                    String::new()
                }
            }

            _ => match self.data {
                // for columns and functions, we fuzzy match with a possible alias.
                CompletionRelevanceData::Column(_) | CompletionRelevanceData::Policy(_) => self
                    .get_table_name()
                    .and_then(|tbl| ctx.get_used_alias_for_table(tbl))
                    .map(|t| format!("{}.{}", t, name))
                    .unwrap_or(name.clone()),

                // everything else is just fuzzy matched against its name.
                _ => name.clone(),
            },
        };

        match fz_matcher.fuzzy_match(
            check_against.to_ascii_lowercase().as_str(),
            content.to_ascii_lowercase().as_str(),
        ) {
            Some(score) => {
                let scorei32: i32 = score
                    .try_into()
                    .expect("The length of the input exceeds i32 capacity");

                // the scoring value isn't linear.
                // here are a couple of samples:
                // - item: bytea_string_agg_transfn, input: n, score: 15
                // - item: numeric_uplus, input: n, score: 31
                // - item: settings, input: sett, score: 91
                // - item: user_settings, input: sett, score: 82
                self.score += if check_against == name {
                    scorei32 / 2
                } else {
                    scorei32 / 3
                };

                if matches!(self.data, CompletionRelevanceData::Schema(_))
                    && self.get_schema_name().is_some_and(|s| s == "pg_catalog")
                {
                    println!("Debug: Schema pg_catalog match score {}", self.score);
                }

                if matches!(self.data, CompletionRelevanceData::Table(_))
                    && self.get_table_name().is_some_and(|s| s == "parameters")
                {
                    println!(
                        "Debug: Table information_schema.parameters match score {}",
                        self.score
                    );
                }
            }
            None => self.skip = true,
        }
    }

    fn check_matching_clause_type(&mut self, ctx: &TreesitterContext) {
        let clause_type = match ctx.wrapping_clause_type.as_ref() {
            None => return,
            Some(ct) => ct,
        };

        let has_mentioned_tables = ctx.has_any_mentioned_relations();
        let has_qualifier = ctx.has_any_qualifier();

        self.score += match self.data {
            CompletionRelevanceData::Table(_) => match clause_type {
                WrappingClause::Update => 10,
                WrappingClause::Delete => 10,
                WrappingClause::From => 5,
                WrappingClause::Join { on_node }
                    if on_node
                        .is_none_or(|on| ctx.node_under_cursor.end_byte() < on.start_byte()) =>
                {
                    5
                }
                _ => -50,
            },
            CompletionRelevanceData::Function(_) => match clause_type {
                WrappingClause::Select if !has_mentioned_tables => 15,
                WrappingClause::Select if has_mentioned_tables => 0,
                WrappingClause::From => 0,
                WrappingClause::CheckOrUsingClause => 0,
                _ => -50,
            },
            CompletionRelevanceData::Column(col) => match clause_type {
                WrappingClause::Select if has_mentioned_tables => 10,
                WrappingClause::Select if !has_mentioned_tables => 0,
                WrappingClause::Where => 10,
                WrappingClause::CheckOrUsingClause => 0,
                WrappingClause::Join { on_node }
                    if on_node
                        .is_some_and(|on| ctx.node_under_cursor.start_byte() > on.end_byte()) =>
                {
                    // Users will probably join on primary keys
                    if col.is_primary_key { 20 } else { 10 }
                }
                _ => -15,
            },
            CompletionRelevanceData::Schema(_) => match clause_type {
                WrappingClause::From if !has_qualifier => 15,
                WrappingClause::Join { .. } if !has_qualifier => 15,
                WrappingClause::Update if !has_qualifier => 15,
                WrappingClause::Delete if !has_qualifier => 15,
                WrappingClause::AlterPolicy if !has_qualifier => 15,
                WrappingClause::DropPolicy if !has_qualifier => 15,
                WrappingClause::CreatePolicy if !has_qualifier => 15,
                _ => -50,
            },
            CompletionRelevanceData::Policy(_) => match clause_type {
                WrappingClause::AlterPolicy | WrappingClause::DropPolicy => 25,
                _ => -50,
            },

            CompletionRelevanceData::Role(_) => match clause_type {
                WrappingClause::DropRole | WrappingClause::AlterRole => 25,
                _ => -50,
            },
        }
    }

    // ok i think we need a rule set first.
    // generally, we want to prefer columns that match a mentioned relation. that's already handled elsewhere. same with schema matches.

    // so, what here? we want to handle the *no content* case.
    // in that case, we want to check the current node_kind.

    fn check_without_content(&mut self, ctx: &TreesitterContext) {
        // the function is only concerned with cases where the user hasn't typed anything yet.
        if ctx
            .get_node_under_cursor_content()
            .is_some_and(|c| !sanitization::is_sanitized_token(c.as_str()))
        {
            return;
        }

        match ctx.node_under_cursor.kind() {
            "function_identifier" | "table_identifier"
                if self.get_schema_name().is_some_and(|s| s == "public") =>
            {
                self.score += 10;
            }

            "schema_identifier" if self.get_schema_name().is_some_and(|s| s != "public") => {
                self.score += 10;
            }

            "any_identifier" => match self.data {
                CompletionRelevanceData::Table(table) => {
                    if table.schema == "public" {
                        self.score += 20;
                    }
                }
                CompletionRelevanceData::Schema(schema) => {
                    if schema.name != "public" {
                        self.score += 10;
                    } else {
                        self.score -= 20;
                    }
                }
                _ => {}
            },

            _ => return,
        }
    }

    fn check_is_invocation(&mut self, ctx: &TreesitterContext) {
        self.score += match self.data {
            CompletionRelevanceData::Function(_) if ctx.is_invocation => 30,
            CompletionRelevanceData::Function(_) if !ctx.is_invocation => -10,
            _ if ctx.is_invocation => -10,
            _ => 0,
        };
    }

    fn check_matches_schema(&mut self, ctx: &TreesitterContext) {
        let schema_from_qualifier = match self.data {
            CompletionRelevanceData::Table(_) | CompletionRelevanceData::Function(_) => {
                ctx.tail_qualifier_sanitized()
            }

            CompletionRelevanceData::Column(_) | CompletionRelevanceData::Policy(_) => {
                ctx.head_qualifier_sanitized()
            }

            CompletionRelevanceData::Schema(_) | CompletionRelevanceData::Role(_) => None,
        };

        if schema_from_qualifier.is_none() {
            return;
        }

        let schema_from_qualifier = schema_from_qualifier.unwrap();

        let data_schema = match self.get_schema_name() {
            Some(s) => s,
            None => return,
        };

        if schema_from_qualifier == data_schema {
            self.score += 25;
        } else {
            self.score -= 10;
        }
    }

    fn get_item_name(&self) -> &str {
        match self.data {
            CompletionRelevanceData::Table(t) => t.name.as_str(),
            CompletionRelevanceData::Function(f) => f.name.as_str(),
            CompletionRelevanceData::Column(c) => c.name.as_str(),
            CompletionRelevanceData::Schema(s) => s.name.as_str(),
            CompletionRelevanceData::Policy(p) => p.name.as_str(),
            CompletionRelevanceData::Role(r) => r.name.as_str(),
        }
    }

    fn get_schema_name(&self) -> Option<&str> {
        match self.data {
            CompletionRelevanceData::Function(f) => Some(f.schema.as_str()),
            CompletionRelevanceData::Table(t) => Some(t.schema.as_str()),
            CompletionRelevanceData::Column(c) => Some(c.schema_name.as_str()),
            CompletionRelevanceData::Schema(s) => Some(s.name.as_str()),
            CompletionRelevanceData::Policy(p) => Some(p.schema_name.as_str()),
            CompletionRelevanceData::Role(_) => None,
        }
    }

    fn get_fully_qualified_name(&self) -> String {
        match self.data {
            CompletionRelevanceData::Schema(s) => s.name.clone(),
            CompletionRelevanceData::Column(c) => {
                format!("{}.{}.{}", c.schema_name, c.table_name, c.name)
            }
            CompletionRelevanceData::Table(t) => format!("{}.{}", t.schema, t.name),
            CompletionRelevanceData::Function(f) => format!("{}.{}", f.schema, f.name),
            CompletionRelevanceData::Policy(p) => {
                format!("{}.{}.{}", p.schema_name, p.table_name, p.name)
            }
            CompletionRelevanceData::Role(r) => r.name.clone(),
        }
    }

    fn get_table_name(&self) -> Option<&str> {
        match self.data {
            CompletionRelevanceData::Column(c) => Some(c.table_name.as_str()),
            CompletionRelevanceData::Table(t) => Some(t.name.as_str()),
            CompletionRelevanceData::Policy(p) => Some(p.table_name.as_str()),
            _ => None,
        }
    }

    fn check_relations_in_stmt(&mut self, ctx: &TreesitterContext) {
        match self.data {
            CompletionRelevanceData::Table(_) | CompletionRelevanceData::Function(_) => return,
            _ => {}
        }

        let schema = match self.get_schema_name() {
            Some(s) => s.to_string(),
            None => return,
        };
        let table_name = match self.get_table_name() {
            Some(t) => t,
            None => return,
        };

        if ctx
            .get_mentioned_relations(&Some(schema.to_string()))
            .is_some_and(|tables| tables.contains(table_name))
        {
            self.score += 45;
        } else if ctx
            .get_mentioned_relations(&None)
            .is_some_and(|tables| tables.contains(table_name))
        {
            self.score += 30;
        }
    }

    fn check_is_user_defined(&mut self) {
        if let CompletionRelevanceData::Role(r) = self.data {
            match r.name.as_str() {
                "pg_read_all_data"
                | "pg_write_all_data"
                | "pg_read_all_settings"
                | "pg_read_all_stats"
                | "pg_stat_scan_tables"
                | "pg_monitor"
                | "pg_database_owner"
                | "pg_signal_backend"
                | "pg_read_server_files"
                | "pg_write_server_files"
                | "pg_execute_server_program"
                | "pg_checkpoint"
                | "pg_maintain"
                | "pg_use_reserved_connections"
                | "pg_create_subscription"
                | "postgres" => self.score -= 20,
                _ => {}
            };

            return;
        }

        let schema_name = match self.get_schema_name() {
            Some(s) => s.to_string(),
            None => return,
        };

        let system_schemas = ["pg_catalog", "information_schema", "pg_toast"];

        if system_schemas.contains(&schema_name.as_str()) {
            self.score -= 20;
        }

        // "public" is the default postgres schema where users
        // create objects. Prefer it by a slight bit.
        if schema_name.as_str() == "public" {
            self.score += 2;
        }

        let item_name = self.get_item_name().to_string();
        let table_name = self.get_table_name();

        // migrations shouldn't pop up on top
        if item_name.contains("migrations")
            || table_name.is_some_and(|t| t.contains("migrations"))
            || schema_name.contains("migrations")
        {
            self.score -= 15;
        }
    }

    fn check_columns_in_stmt(&mut self, ctx: &TreesitterContext) {
        if let CompletionRelevanceData::Column(column) = self.data {
            /*
             * Columns can be mentioned in one of two ways:
             *
             * 1) With an alias: `select u.id`.
             * If the currently investigated suggestion item is "id" of the "users" table,
             * we want to check
             * a) whether the name of the column matches.
             * b) whether we know which table is aliased by "u" (if we don't, we ignore the alias).
             * c) whether the aliased table matches the currently investigated suggestion item's table.
             *
             * 2) Without an alias: `select id`.
             * In that case, we only check whether the mentioned column fits our currently investigated
             * suggestion item's name.
             *
             */
            if ctx
                .get_mentioned_columns(&ctx.wrapping_clause_type)
                .is_some_and(|set| {
                    set.iter().any(|mentioned| match mentioned.alias.as_ref() {
                        Some(als) => {
                            let aliased_table = ctx.get_mentioned_table_for_alias(als.as_str());
                            column.name == mentioned.column.replace('"', "")
                                && aliased_table.is_none_or(|t| t == &column.table_name)
                        }
                        None => mentioned.column == column.name,
                    })
                })
            {
                self.score -= 10;
            }
        }
    }

    fn check_is_not_wellknown_migration(&mut self, _ctx: &TreesitterContext) {
        if let Some(table_name) = self.get_table_name() {
            if ["_sqlx_migrations"].contains(&table_name) {
                self.score -= 10;
            }
        }

        if let Some(schema_name) = self.get_schema_name() {
            if ["supabase_migrations"].contains(&schema_name) {
                self.score -= 10;
            }
        }
    }
}
