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

/// Comments of a statement, indexed by the node they precede.
#[derive(Debug, Default)]
pub struct AttachedComments {
    pub by_location: HashMap<i32, Vec<Comment>>,
    /// Comments that no node follows. The caller must not reformat a statement that has any:
    /// emitting it would drop them.
    pub unattached: Vec<Comment>,
}

/// Attaches every comment of `sql` to the node that starts right after it.
///
/// Attachment is positional because libpg_query drops comments from the AST, and positional is
/// enough: nodes carrying an i32 location field preserve the byte offset they were parsed from.
pub fn attach_comments(sql: &str, ast: &NodeEnum) -> AttachedComments {
    let mut attached = AttachedComments::default();

    let comments = collect_comments(sql);
    if comments.is_empty() {
        return attached;
    }

    let mut locations = collect_node_locations(ast);
    locations.sort_unstable();

    for (end, comment) in comments {
        match locations.iter().find(|location| **location as usize >= end) {
            Some(location) => attached
                .by_location
                .entry(*location)
                .or_default()
                .push(comment),
            None => attached.unattached.push(comment),
        }
    }

    attached
}

/// Every comment of the statement, as (end offset, comment), in source order.
fn collect_comments(sql: &str) -> Vec<(usize, Comment)> {
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

            Some((end, Comment { text, line_comment }))
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
        assert_eq!(attached.by_location.len(), 1);

        let comments = attached.by_location.values().next().expect("one entry");
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].text, "-- pick the magic value");
        assert!(comments[0].line_comment);
    }

    #[test]
    fn a_block_comment_is_not_a_line_comment() {
        let sql = "SELECT /* inline */ 1 FROM s.t";
        let attached = attach_comments(sql, &parse(sql));

        let comments = attached.by_location.values().next().expect("one entry");
        assert_eq!(comments[0].text, "/* inline */");
        assert!(!comments[0].line_comment);
    }

    #[test]
    fn a_comment_with_no_node_after_it_is_reported_as_unattached() {
        let sql = "SELECT 1 FROM s.t -- trailing";
        let attached = attach_comments(sql, &parse(sql));

        assert_eq!(attached.unattached.len(), 1);
        assert_eq!(attached.unattached[0].text, "-- trailing");
    }

    #[test]
    fn a_statement_without_comments_produces_an_empty_map() {
        let sql = "SELECT 1 FROM s.t";
        let attached = attach_comments(sql, &parse(sql));

        assert!(attached.by_location.is_empty());
        assert!(attached.unattached.is_empty());
    }
}
