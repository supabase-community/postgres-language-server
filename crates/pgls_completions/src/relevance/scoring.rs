use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};

use pgls_migrations::{History, MigrationTool};
use pgls_treesitter::context::{TreesitterContext, WrappingClause};

use crate::{providers::helper::word_at_cursor, sanitization};

use super::CompletionRelevanceData;

/// What scoring compares every candidate with, computed once per request: the typed text and
/// one fuzzy matcher, whose buffers are reused across the candidates.
pub(crate) struct TypedInput {
    /// The word at the cursor, sanitized, without quotes and in ASCII lowercase. Keywords are
    /// matched against it. `None` if nothing was typed.
    word: Option<String>,
    /// The text of the node under the cursor, in the same form. Every other candidate is matched
    /// against it.
    node: Option<String>,
    matcher: SkimMatcherV2,
}

impl TypedInput {
    pub(crate) fn new(ctx: &TreesitterContext) -> Self {
        let normalized = |content: Option<String>| {
            content
                .filter(|c| !sanitization::is_sanitized_token(c))
                .map(|c| {
                    let mut c = sanitization::remove_sanitized_token(&c).replace('"', "");
                    c.make_ascii_lowercase();
                    c
                })
        };

        Self {
            word: normalized(ctx.text.get(word_at_cursor(ctx)).map(str::to_string)),
            node: normalized(ctx.get_node_under_cursor_content()),
            matcher: SkimMatcherV2::default(),
        }
    }

    /// Whether the fuzzy matcher can match `typed` in the `candidate`'s bytes, checked on the
    /// borrowed bytes before anything is copied. The matcher needs every typed character in order
    /// and folds only ASCII case, and equal characters have equal bytes, so a candidate without
    /// the typed bytes in order can't match. `typed` is in ASCII lowercase.
    fn may_match(typed: &str, candidate: impl Iterator<Item = u8>) -> bool {
        let mut typed = typed.bytes().peekable();
        for byte in candidate {
            match typed.peek() {
                Some(t) if *t == byte.to_ascii_lowercase() => {
                    typed.next();
                }
                Some(_) => {}
                None => break,
            }
        }
        typed.peek().is_none()
    }
}

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
    pub(crate) fn should_skip(&self) -> bool {
        self.skip
    }

    pub(crate) fn get_score(&self) -> i32 {
        self.score
    }

    pub(crate) fn calc_score(&mut self, ctx: &TreesitterContext, input: &TypedInput) {
        self.check_is_keyword();
        self.check_is_user_defined();
        self.check_matches_schema(ctx);
        self.check_matches_query_input(ctx, input);
        self.check_is_invocation(ctx);
        self.check_matching_clause_type(ctx);
        self.check_without_content(ctx, input);
        self.check_relations_in_stmt(ctx);
        self.check_columns_in_stmt(ctx);
        self.check_is_not_wellknown_migration(ctx);
    }

    fn check_is_keyword(&mut self) {
        if matches!(self.data, CompletionRelevanceData::Keyword(_)) {
            self.score -= 10;
        }
    }

    fn check_matches_query_input(&mut self, ctx: &TreesitterContext, input: &TypedInput) {
        let typed = match self.data {
            CompletionRelevanceData::Keyword(_) => input.word.as_deref(),
            _ => input.node.as_deref(),
        };
        let Some(typed) = typed else {
            return;
        };

        let name = self.get_item_name();

        // What the text the user typed qualifies `name` with, if anything: `<qualifier>.<name>`.
        let qualifier = match &ctx.identifier_qualifiers {
            // If both qualifiers are already written out, we must check the item's name itself.
            (Some(_), Some(_)) => None,

            // If only one qualifier is written out, we might look at a schema, a table, or an alias.
            (None, Some(_)) => {
                let qualifier = ctx.tail_qualifier_sanitized().unwrap_or_default();
                let qualifier = qualifier.as_str();
                if self.get_schema_name().is_some_and(|s| s == qualifier) {
                    self.get_table_name().map(str::to_string)
                } else if self.get_table_name().is_some_and(|t| t == qualifier)
                    || ctx
                        .get_mentioned_table_for_alias(qualifier)
                        .is_some_and(|alias_tbl| {
                            self.get_table_name()
                                .is_some_and(|item_tbl| alias_tbl == item_tbl)
                        })
                {
                    None
                } else {
                    // the qualifier does not match schema, table, or alias.
                    // what the hell is it?
                    // probably a typo.
                    self.skip = true;
                    return;
                }
            }

            _ => match self.data {
                // for columns and functions, we fuzzy match with a possible alias.
                CompletionRelevanceData::Column(_) | CompletionRelevanceData::Policy(_) => self
                    .get_schema_name()
                    .zip(self.get_table_name())
                    .and_then(|(schema, tbl)| ctx.get_used_alias_for_table(schema, tbl)),

                // everything else is just fuzzy matched against its name.
                _ => None,
            },
        };

        let candidate = qualifier
            .iter()
            .flat_map(|q| q.bytes().chain(*b"."))
            .chain(name.bytes());
        if !TypedInput::may_match(typed, candidate) {
            self.skip = true;
            return;
        }

        let mut check_against = match &qualifier {
            Some(qualifier) => format!("{qualifier}.{name}"),
            None => name.to_string(),
        };
        check_against.make_ascii_lowercase();

        match input.matcher.fuzzy_match(check_against.as_str(), typed) {
            Some(score) => {
                let mut scorei32 = i32::try_from(score).unwrap_or(i32::MAX);

                // give a significant bonus for prefix matches since these are much more
                // likely what the user is looking for
                if check_against.starts_with(typed) {
                    scorei32 = scorei32.saturating_add(20);
                }

                // the scoring value isn't linear.
                // here are a couple of samples:
                // - item: bytea_string_agg_transfn, input: n, score: 15
                // - item: numeric_uplus, input: n, score: 31
                // - item: settings, input: sett, score: 91
                // - item: user_settings, input: sett, score: 82
                self.score += if qualifier.is_none() {
                    scorei32 / 2
                } else {
                    scorei32 / 3
                };
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
                WrappingClause::Merge => 10,
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
                WrappingClause::Returning => 0,
                WrappingClause::Merge | WrappingClause::MergeWhen => 0,
                WrappingClause::CheckOrUsingClause => 0,
                _ => -50,
            },
            CompletionRelevanceData::Column(col) => match clause_type {
                WrappingClause::Select if has_mentioned_tables => 10,
                WrappingClause::Select if !has_mentioned_tables => 0,
                WrappingClause::Where => 10,
                // RETURNING has the target table.
                WrappingClause::Returning => 10,
                // MERGE has the target and the source.
                WrappingClause::Merge | WrappingClause::MergeWhen => 10,
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
                WrappingClause::Merge if !has_qualifier => 15,
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

            CompletionRelevanceData::Keyword(_) => 0,
        }
    }

    // ok i think we need a rule set first.
    // generally, we want to prefer columns that match a mentioned relation. that's already handled elsewhere. same with schema matches.

    // so, what here? we want to handle the *no content* case.
    // in that case, we want to check the current node_kind.

    fn check_without_content(&mut self, ctx: &TreesitterContext, input: &TypedInput) {
        // the function is only concerned with cases where the user hasn't typed anything yet.
        if input.node.is_some() {
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

            _ => (),
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

            CompletionRelevanceData::Schema(_)
            | CompletionRelevanceData::Role(_)
            | CompletionRelevanceData::Keyword(_) => None,
        };

        let Some(schema_from_qualifier) = schema_from_qualifier else {
            return;
        };

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
            CompletionRelevanceData::Keyword(k) => k.name,
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
            CompletionRelevanceData::Keyword(_) => None,
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
            if r.is_predefined() || r.name == "postgres" {
                self.score -= 20;
            }

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
        // create objects. Prefer it by a slight bit, but do not suggest the literal one.
        if schema_name.as_str() == "public" {
            match self.data {
                CompletionRelevanceData::Schema(_) => {
                    self.score -= 2;
                }
                _ => {
                    self.score += 2;
                }
            }
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
                            column.name == mentioned.column
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

    /// Ranks the tables where migration tools record the migrations they ran low.
    fn check_is_not_wellknown_migration(&mut self, _ctx: &TreesitterContext) {
        let table_name = self.get_table_name();
        let schema_name = self.get_schema_name();
        let is_history = |history: &History| match history {
            History::Table(name) => table_name == Some(*name),
            History::Schema(name) => schema_name == Some(*name),
        };
        if MigrationTool::ALL
            .into_iter()
            .filter_map(|tool| tool.database().history)
            .any(|history| is_history(&history))
            // sqlx records its migrations here, but isn't a migration tool of ours yet.
            || table_name == Some("_sqlx_migrations")
        {
            self.score -= 10;
        }
    }
}

#[cfg(test)]
mod tests {
    use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};

    use super::TypedInput;

    #[test]
    fn the_byte_filter_rejects_only_what_the_matcher_rejects() {
        let matcher = SkimMatcherV2::default();
        let candidates = [
            "users",
            "User_Settings",
            "u.email",
            "public.users",
            "Счета",
            "счета",
            "café_id",
            "ü",
            "",
        ];
        let typed = [
            "", "us", "usr", "sett", "u.em", "pu.us", "сч", "сЧ", "é", "fé", "ü", "x", "ti",
        ];

        for candidate in candidates {
            let lower = candidate.to_ascii_lowercase();
            for typed in typed {
                let matches = matcher.fuzzy_match(&lower, typed).is_some();
                assert_eq!(
                    TypedInput::may_match(typed, candidate.bytes()),
                    matches,
                    "{typed:?} in {candidate:?}"
                );
            }
        }
    }
}
