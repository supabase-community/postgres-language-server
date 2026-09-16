use std::collections::HashMap;

use pgls_query::{NodeEnum, protobuf::Token};

use crate::codegen::node_location::node_location;

/// A comment found in the source of a statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    pub text: String,
    /// True for `--` comments, which run to the end of the line and therefore force a break.
    pub line_comment: bool,
}

/// Comments of a statement, indexed by the node they surround.
#[derive(Debug, Default)]
pub struct AttachedComments {
    /// Comments emitted before the node at this location.
    pub leading_by_location: HashMap<i32, Vec<Comment>>,
    /// Comments emitted after the node at this location.
    pub trailing_by_location: HashMap<i32, Vec<Comment>>,
    /// Comments that neither neighbour can hold, which today means a comment written after a
    /// statement terminator. The caller must not reformat a statement that has any: emitting it
    /// would drop them.
    pub unattached: Vec<Comment>,
}

/// Which side of a node a comment ends up on, once the node is known.
enum Placement {
    /// Emitted before the node at this location.
    Leading(i32),
    /// Emitted after the node at this location.
    Trailing(i32),
}

/// Attaches every comment of `sql` to a nearby AST node.
///
/// Attachment is positional because libpg_query drops comments from the AST, and positional is
/// enough: nodes carrying an i32 location field preserve the byte offset they were parsed from.
/// A comment preceded only by whitespace on its line is leading and belongs to the next node. A
/// comment following SQL on the same line is trailing and belongs to the previous node.
pub fn attach_comments(sql: &str, ast: &NodeEnum) -> AttachedComments {
    let mut attached = AttachedComments::default();

    let comments = collect_comments(sql);
    if comments.is_empty() {
        return attached;
    }

    let mut locations = collect_node_locations(ast);
    locations.sort_unstable();

    for source_comment in comments {
        let line_start = sql[..source_comment.start]
            .rfind('\n')
            .map_or(0, |offset| offset + 1);
        let line_prefix = &sql[line_start..source_comment.start];

        // A line comment written after a terminator documents the next statement, not this one.
        // Attaching it here would move it across a statement boundary.
        if source_comment.comment.line_comment && line_prefix.trim_end().ends_with(';') {
            attached.unattached.push(source_comment.comment);
            continue;
        }

        let leads = !source_comment.comment.line_comment
            || line_prefix.trim().is_empty()
            || ends_with_clause_header(line_prefix)
            || ends_with_structural_separator(line_prefix);

        let next = locations
            .iter()
            .find(|location| **location as usize >= source_comment.end)
            .copied();
        let previous = locations
            .iter()
            .rev()
            .find(|location| **location as usize <= source_comment.start)
            .copied();

        // The preferred side first, the other one as a fallback. A comment closing a list or a
        // statement has no node after it, and printing it after the node it already follows in the
        // source keeps it where its author wrote it, where refusing the statement keeps nothing.
        let placement = if leads {
            next.map(Placement::Leading)
                .or_else(|| previous.map(Placement::Trailing))
        } else {
            previous
                .map(Placement::Trailing)
                .or_else(|| next.map(Placement::Leading))
        };

        match placement {
            Some(Placement::Leading(location)) => attached
                .leading_by_location
                .entry(location)
                .or_default()
                .push(source_comment.comment),
            Some(Placement::Trailing(location)) => attached
                .trailing_by_location
                .entry(location)
                .or_default()
                .push(source_comment.comment),
            None => attached.unattached.push(source_comment.comment),
        }
    }

    attached
}

/// Returns whether `line_prefix` ends with a SQL clause or connective keyword.
///
/// Such a keyword is not represented by an AST node with its own source location. A line comment
/// immediately after it must therefore be emitted before the following expression, not after the
/// previously emitted AST node.
fn ends_with_clause_header(line_prefix: &str) -> bool {
    const HEADERS: &[&str] = &[
        "SELECT",
        "FROM",
        "WHERE",
        "GROUP BY",
        "HAVING",
        "WINDOW",
        "ORDER BY",
        "LIMIT",
        "OFFSET",
        "FETCH",
        "JOIN",
        "ON",
        "USING",
        "AND",
        "OR",
        "WHEN",
        "THEN",
        "ELSE",
        "VALUES",
        "SET",
        "RETURNING",
        "UNION",
        "INTERSECT",
        "EXCEPT",
    ];

    let normalized = line_prefix.trim_end().to_ascii_uppercase();

    HEADERS.iter().any(|header| {
        let Some(prefix) = normalized.strip_suffix(header) else {
            return false;
        };

        prefix.is_empty() || prefix.chars().last().is_some_and(char::is_whitespace)
    })
}

/// Returns whether `line_prefix` ends with punctuation that separates AST nodes.
///
/// Commas, brackets and braces are emitted by parent formatters rather than a dedicated AST node.
/// A comment after one of them must be emitted before the following node; otherwise it is
/// incorrectly attached to the last child inside the preceding expression on the next pass.
/// Parentheses are deliberately excluded: they can close a semantic expression, so a following
/// comment belongs to that expression rather than to the next node.
fn ends_with_structural_separator(line_prefix: &str) -> bool {
    matches!(
        line_prefix.trim_end().chars().last(),
        Some(',' | ']' | '}')
    )
}

struct SourceComment {
    start: usize,
    end: usize,
    comment: Comment,
}

/// Every comment of the statement, with its source range, in source order.
fn collect_comments(sql: &str) -> Vec<SourceComment> {
    let Ok(scan) = pgls_query::scan(sql) else {
        return Vec::new();
    };

    scan.tokens
        .iter()
        .filter_map(|token| {
            let kind = Token::try_from(token.token).ok()?;
            let line_comment = match kind {
                Token::SqlComment => true,
                Token::CComment => false,
                _ => return None,
            };

            let start = usize::try_from(token.start).ok()?;
            let end = usize::try_from(token.end).ok()?;
            let text = sql.get(start..end)?.trim_end().to_string();

            Some(SourceComment {
                start,
                end,
                comment: Comment { text, line_comment },
            })
        })
        .collect()
}

fn collect_node_locations(ast: &NodeEnum) -> Vec<i32> {
    ast.iter()
        .filter_map(|node| node_location(&node))
        .filter(|location| *location >= 0)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(sql: &str) -> pgls_query::NodeEnum {
        pgls_query::parse(sql)
            .expect("parse")
            .into_root()
            .expect("root")
    }

    #[test]
    fn a_comment_attaches_to_the_node_that_follows_it() {
        let sql = "SELECT\n-- pick the magic value\n1 FROM s.t";
        let attached = attach_comments(sql, &parse(sql));

        assert!(attached.unattached.is_empty());
        assert_eq!(attached.leading_by_location.len(), 1);
        assert!(attached.trailing_by_location.is_empty());

        let comments = attached
            .leading_by_location
            .values()
            .next()
            .expect("one entry");
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].text, "-- pick the magic value");
        assert!(comments[0].line_comment);
    }

    #[test]
    fn a_block_comment_is_not_a_line_comment() {
        let sql = "SELECT /* inline */ 1 FROM s.t";
        let attached = attach_comments(sql, &parse(sql));

        let comments = attached
            .leading_by_location
            .values()
            .next()
            .expect("one entry");
        assert_eq!(comments[0].text, "/* inline */");
        assert!(!comments[0].line_comment);
    }

    #[test]
    fn a_trailing_comment_attaches_to_the_node_that_precedes_it() {
        let sql = "SELECT * FROM t WHERE a = 1 -- context\nAND b = 2";
        let attached = attach_comments(sql, &parse(sql));

        assert!(attached.unattached.is_empty());
        assert!(attached.leading_by_location.is_empty());
        assert_eq!(attached.trailing_by_location.len(), 1);

        let comments = attached
            .trailing_by_location
            .values()
            .next()
            .expect("one entry");
        assert_eq!(comments[0].text, "-- context");
        assert!(comments[0].line_comment);
    }

    #[test]
    fn a_comment_after_a_clause_header_attaches_to_the_node_that_follows_it() {
        let sql = "SELECT * FROM t ORDER BY -- sort by name\nname";
        let attached = attach_comments(sql, &parse(sql));

        assert!(attached.unattached.is_empty());
        assert_eq!(attached.leading_by_location.len(), 1);
        assert!(attached.trailing_by_location.is_empty());

        let comments = attached
            .leading_by_location
            .values()
            .next()
            .expect("one entry");
        assert_eq!(comments[0].text, "-- sort by name");
        assert!(comments[0].line_comment);
    }

    #[test]
    fn a_comment_after_a_separator_attaches_to_the_node_that_follows_it() {
        let sql = "SELECT a, -- temporarily omit b\nb FROM t";
        let attached = attach_comments(sql, &parse(sql));

        assert!(attached.unattached.is_empty());
        assert_eq!(attached.leading_by_location.len(), 1);
        assert!(attached.trailing_by_location.is_empty());

        let comments = attached
            .leading_by_location
            .values()
            .next()
            .expect("one entry");
        assert_eq!(comments[0].text, "-- temporarily omit b");
        assert!(comments[0].line_comment);
    }

    #[test]
    fn a_comment_with_no_node_after_it_falls_back_to_the_previous_node() {
        let sql = "SELECT 1 FROM s.t -- trailing";
        let attached = attach_comments(sql, &parse(sql));

        assert!(attached.unattached.is_empty());
        assert_eq!(attached.trailing_by_location.len(), 1);
        let comments = attached
            .trailing_by_location
            .values()
            .next()
            .expect("one entry");
        assert_eq!(comments[0].text, "-- trailing");
    }

    #[test]
    fn a_comment_after_a_statement_terminator_stays_unattached() {
        let sql = "SELECT 1 FROM s.t; -- trailing";
        let attached = attach_comments(sql, &parse(sql));

        assert_eq!(attached.unattached.len(), 1);
        assert_eq!(attached.unattached[0].text, "-- trailing");
    }

    #[test]
    fn a_statement_without_comments_produces_an_empty_map() {
        let sql = "SELECT 1 FROM s.t";
        let attached = attach_comments(sql, &parse(sql));

        assert!(attached.leading_by_location.is_empty());
        assert!(attached.trailing_by_location.is_empty());
        assert!(attached.unattached.is_empty());
    }
}
