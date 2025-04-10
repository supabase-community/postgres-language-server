use std::sync::{Arc, Mutex};

use dashmap::DashMap;
use tree_sitter::InputEdit;

use super::{change::ModifiedStatement, statement_identifier::StatementId};

pub struct TreeSitterStore {
    db: DashMap<StatementId, Arc<tree_sitter::Tree>>,
    parser: Mutex<tree_sitter::Parser>,
}

impl TreeSitterStore {
    pub fn new() -> TreeSitterStore {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(tree_sitter_sql::language())
            .expect("Error loading sql language");

        TreeSitterStore {
            db: DashMap::new(),
            parser: Mutex::new(parser),
        }
    }

    pub fn load_parse(&self, statement: &StatementId, content: &str) -> Arc<tree_sitter::Tree> {
        if let Some(existing) = self.db.get(statement).map(|x| x.clone()) {
            return existing;
        }

        let mut parser = self.parser.lock().expect("Failed to lock parser");
        let tree = Arc::new(parser.parse(content, None).unwrap());
        self.db.insert(statement.clone(), tree.clone());

        tree
    }

    pub fn add_statement(&self, statement: &StatementId, content: &str) {
        let mut parser = self.parser.lock().expect("Failed to lock parser");
        let tree = parser.parse(content, None).unwrap();
        self.db.insert(statement.clone(), Arc::new(tree));
    }

    pub fn remove_statement(&self, id: &StatementId) {
        self.db.remove(id);

        if let Some(child_id) = id.as_child() {
            self.db.remove(&child_id);
        }
    }

    pub fn modify_statement(&self, change: &ModifiedStatement) {
        let old = self.db.remove(&change.old_stmt);

        if old.is_none() {
            self.add_statement(&change.new_stmt, &change.change_text);
            return;
        }

        // we clone the three for now, lets see if that is sufficient or if we need to mutate the
        // original tree instead but that will require some kind of locking
        let mut tree = old.unwrap().1.as_ref().clone();

        let edit = edit_from_change(
            change.old_stmt_text.as_str(),
            usize::from(change.change_range.start()),
            usize::from(change.change_range.end()),
            change.change_text.as_str(),
        );

        tree.edit(&edit);

        let mut parser = self.parser.lock().expect("Failed to lock parser");
        // todo handle error
        self.db.insert(
            change.new_stmt.clone(),
            Arc::new(parser.parse(&change.new_stmt_text, Some(&tree)).unwrap()),
        );
    }
}

// Converts character positions and replacement text into a tree-sitter InputEdit
pub(crate) fn edit_from_change(
    text: &str,
    start_char: usize,
    end_char: usize,
    replacement_text: &str,
) -> InputEdit {
    let mut start_byte = 0;
    let mut end_byte = 0;
    let mut chars_counted = 0;

    let mut line = 0;
    let mut current_line_char_start = 0; // Track start of the current line in characters
    let mut column_start = 0;
    let mut column_end = 0;

    // Find the byte positions corresponding to the character positions
    for (idx, c) in text.char_indices() {
        if chars_counted == start_char {
            start_byte = idx;
            column_start = chars_counted - current_line_char_start;
        }
        if chars_counted == end_char {
            end_byte = idx;
            column_end = chars_counted - current_line_char_start;
            break; // Found both start and end
        }
        if c == '\n' {
            line += 1;
            current_line_char_start = chars_counted + 1; // Next character starts a new line
        }
        chars_counted += 1;
    }

    // Handle case where end_char is at the end of the text
    if end_char == chars_counted && end_byte == 0 {
        end_byte = text.len();
        column_end = chars_counted - current_line_char_start;
    }

    let start_point = tree_sitter::Point::new(line, column_start);
    let old_end_point = tree_sitter::Point::new(line, column_end);

    // Calculate the new end byte after the edit
    let new_end_byte = start_byte + replacement_text.len();

    // Calculate the new end position
    let new_lines = replacement_text.matches('\n').count();
    let last_line_length = if new_lines > 0 {
        replacement_text
            .split('\n')
            .next_back()
            .unwrap_or("")
            .chars()
            .count()
    } else {
        replacement_text.chars().count()
    };

    let new_end_position = if new_lines > 0 {
        // If there are new lines, the row is offset by the number of new lines, and the column is the length of the last line
        tree_sitter::Point::new(start_point.row + new_lines, last_line_length)
    } else {
        // If there are no new lines, the row remains the same, and the column is offset by the length of the insertion
        tree_sitter::Point::new(start_point.row, start_point.column + last_line_length)
    };

    InputEdit {
        start_byte,
        old_end_byte: end_byte,
        new_end_byte,
        start_position: start_point,
        old_end_position: old_end_point,
        new_end_position,
    }
}
