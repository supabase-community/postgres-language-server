use std::fmt::Write;

use humansize::DECIMAL;
use pgls_schema_cache::{SchemaCache, Table};
use pgls_treesitter::TreesitterContext;

use crate::{contextual_priority::ContextualPriority, to_markdown::ToHoverMarkdown};

const MAX_COLUMNS_IN_HOVER: usize = 20;

impl ToHoverMarkdown for Table {
    fn hover_headline<W: Write>(
        &self,
        writer: &mut W,
        _schema_cache: &SchemaCache,
    ) -> Result<(), std::fmt::Error> {
        write!(writer, "`{}.{}`", self.schema, self.name)?;

        let table_kind = match self.table_kind {
            pgls_schema_cache::TableKind::View => " (View)",
            pgls_schema_cache::TableKind::MaterializedView => " (M.View)",
            pgls_schema_cache::TableKind::Partitioned => " (Partitioned)",
            pgls_schema_cache::TableKind::Ordinary => "",
        };

        write!(writer, "{table_kind}")?;

        let locked_txt = if self.rls_enabled {
            " - 🔒 RLS enabled"
        } else {
            " - 🔓 RLS disabled"
        };

        write!(writer, "{locked_txt}")?;

        Ok(())
    }

    fn hover_body<W: Write>(
        &self,
        writer: &mut W,
        schema_cache: &SchemaCache,
    ) -> Result<bool, std::fmt::Error> {
        if let Some(comment) = &self.comment {
            write!(writer, "Comment: '{comment}'")?;
            writeln!(writer)?;
        }

        let mut columns: Vec<_> = schema_cache
            .columns
            .iter()
            .filter(|column| column.schema_name == self.schema && column.table_name == self.name)
            .collect();
        columns.sort_by_key(|column| column.number);

        writeln!(writer, "Columns:")?;

        for column in columns.iter().take(MAX_COLUMNS_IN_HOVER) {
            write!(writer, "- {}: ", column.name)?;

            if let Some(type_name) = &column.type_name {
                write!(writer, "{type_name}")?;

                if let Some(varchar_length) = column.varchar_length {
                    write!(writer, "({varchar_length})")?;
                }
            } else {
                write!(writer, "typeid:{}", column.type_id)?;
            }

            if column.is_nullable {
                write!(writer, " - nullable")?;
            } else {
                write!(writer, " - not null")?;
            }

            if let Some(default_expr) = column
                .default_expr
                .as_deref()
                .and_then(extract_basic_default_literal)
            {
                write!(writer, " - default: {default_expr}")?;
            }

            writeln!(writer)?;
        }

        if columns.len() > MAX_COLUMNS_IN_HOVER {
            writeln!(
                writer,
                "... +{} more columns",
                columns.len() - MAX_COLUMNS_IN_HOVER
            )?;
        }

        Ok(true)
    }

    fn hover_footer<W: Write>(
        &self,
        writer: &mut W,
        _schema_cache: &SchemaCache,
    ) -> Result<bool, std::fmt::Error> {
        writeln!(writer)?;
        write!(
            writer,
            "~{} rows, ~{} dead rows, {}",
            self.live_rows_estimate,
            self.dead_rows_estimate,
            humansize::format_size(self.bytes as u64, DECIMAL)
        )?;
        Ok(true)
    }
}

fn extract_basic_default_literal(default_expr: &str) -> Option<String> {
    let mut candidate = default_expr.trim();

    loop {
        let mut changed = false;

        if let Some(unwrapped) = strip_outer_parentheses(candidate) {
            candidate = unwrapped.trim();
            changed = true;
        }

        if let Some(without_cast) = strip_trailing_top_level_casts(candidate) {
            candidate = without_cast.trim();
            changed = true;
        }

        if !changed {
            break;
        }
    }

    if is_basic_literal(candidate) {
        Some(candidate.to_string())
    } else {
        None
    }
}

fn strip_outer_parentheses(value: &str) -> Option<&str> {
    let value = value.trim();

    if !value.starts_with('(') || !value.ends_with(')') {
        return None;
    }

    let mut depth = 0_i32;
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let bytes = value.as_bytes();
    let mut idx = 0_usize;

    while idx < bytes.len() {
        let ch = bytes[idx] as char;

        if in_single_quote {
            if ch == '\'' {
                if idx + 1 < bytes.len() && bytes[idx + 1] as char == '\'' {
                    idx += 2;
                    continue;
                }

                in_single_quote = false;
            }

            idx += 1;
            continue;
        }

        if in_double_quote {
            if ch == '"' {
                in_double_quote = false;
            }

            idx += 1;
            continue;
        }

        match ch {
            '\'' => in_single_quote = true,
            '"' => in_double_quote = true,
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 && idx != bytes.len() - 1 {
                    return None;
                }
            }
            _ => {}
        }

        idx += 1;
    }

    if depth != 0 || in_single_quote || in_double_quote {
        return None;
    }

    Some(&value[1..value.len() - 1])
}

fn strip_trailing_top_level_casts(value: &str) -> Option<&str> {
    let bytes = value.as_bytes();
    let mut depth = 0_i32;
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut idx = 0_usize;

    while idx + 1 < bytes.len() {
        let ch = bytes[idx] as char;

        if in_single_quote {
            if ch == '\'' {
                if idx + 1 < bytes.len() && bytes[idx + 1] as char == '\'' {
                    idx += 2;
                    continue;
                }
                in_single_quote = false;
            }
            idx += 1;
            continue;
        }

        if in_double_quote {
            if ch == '"' {
                in_double_quote = false;
            }
            idx += 1;
            continue;
        }

        match ch {
            '\'' => {
                in_single_quote = true;
                idx += 1;
                continue;
            }
            '"' => {
                in_double_quote = true;
                idx += 1;
                continue;
            }
            '(' => {
                depth += 1;
                idx += 1;
                continue;
            }
            ')' => {
                depth -= 1;
                idx += 1;
                continue;
            }
            ':' if depth == 0 && bytes[idx + 1] as char == ':' => {
                let suffix = &value[idx..];
                if suffix
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || ":_\".()[] ,\t".contains(c))
                {
                    return Some(value[..idx].trim_end());
                }
                return None;
            }
            _ => {
                idx += 1;
                continue;
            }
        }
    }

    None
}

fn is_basic_literal(value: &str) -> bool {
    let value = value.trim();

    if value.eq_ignore_ascii_case("true")
        || value.eq_ignore_ascii_case("false")
        || value.eq_ignore_ascii_case("null")
    {
        return true;
    }

    is_numeric_literal(value) || is_single_quoted_literal(value)
}

fn is_single_quoted_literal(value: &str) -> bool {
    let bytes = value.as_bytes();

    if bytes.len() < 2 || bytes.first() != Some(&b'\'') || bytes.last() != Some(&b'\'') {
        return false;
    }

    let mut idx = 1_usize;
    let end = bytes.len() - 1;

    while idx < end {
        if bytes[idx] == b'\'' {
            if idx + 1 < end && bytes[idx + 1] == b'\'' {
                idx += 2;
            } else {
                return false;
            }
        } else {
            idx += 1;
        }
    }

    true
}

fn is_numeric_literal(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.is_empty() {
        return false;
    }

    let mut idx = 0_usize;

    if matches!(bytes[idx], b'+' | b'-') {
        idx += 1;
        if idx >= bytes.len() {
            return false;
        }
    }

    let integer_start = idx;
    while idx < bytes.len() && bytes[idx].is_ascii_digit() {
        idx += 1;
    }
    let has_integer_digits = idx > integer_start;

    if idx < bytes.len() && bytes[idx] == b'.' {
        idx += 1;
        let fractional_start = idx;
        while idx < bytes.len() && bytes[idx].is_ascii_digit() {
            idx += 1;
        }
        if !has_integer_digits && idx == fractional_start {
            return false;
        }
    } else if !has_integer_digits {
        return false;
    }

    if idx < bytes.len() && matches!(bytes[idx], b'e' | b'E') {
        idx += 1;
        if idx < bytes.len() && matches!(bytes[idx], b'+' | b'-') {
            idx += 1;
        }

        let exponent_start = idx;
        while idx < bytes.len() && bytes[idx].is_ascii_digit() {
            idx += 1;
        }

        if idx == exponent_start {
            return false;
        }
    }

    idx == bytes.len()
}

impl ContextualPriority for Table {
    fn relevance_score(&self, ctx: &TreesitterContext) -> f32 {
        let mut score = 0.0;

        if ctx
            .get_mentioned_relations(&Some(self.schema.clone()))
            .is_some_and(|t| t.contains(&self.name))
        {
            score += 200.0;
        } else if ctx
            .get_mentioned_relations(&None)
            .is_some_and(|t| t.contains(&self.name))
        {
            score += 150.0;
        } else if ctx
            .get_mentioned_relations(&Some(self.schema.clone()))
            .is_some()
        {
            score += 50.0;
        }

        if self.schema == "public" && score == 0.0 {
            score += 10.0;
        }

        score
    }
}

#[cfg(test)]
mod tests {
    use super::extract_basic_default_literal;

    #[test]
    fn extracts_basic_defaults_with_optional_casts() {
        assert_eq!(
            extract_basic_default_literal("'anonymous'::text"),
            Some("'anonymous'".to_string())
        );
        assert_eq!(extract_basic_default_literal("(42)::int8"), Some("42".to_string()));
        assert_eq!(
            extract_basic_default_literal("NULL::character varying"),
            Some("NULL".to_string())
        );
        assert_eq!(
            extract_basic_default_literal("false::boolean"),
            Some("false".to_string())
        );
    }

    #[test]
    fn ignores_non_basic_defaults() {
        assert_eq!(
            extract_basic_default_literal("nextval('users_id_seq'::regclass)"),
            None
        );
        assert_eq!(extract_basic_default_literal("now()"), None);
        assert_eq!(
            extract_basic_default_literal("'a'::text || 'b'::text"),
            None
        );
    }
}
