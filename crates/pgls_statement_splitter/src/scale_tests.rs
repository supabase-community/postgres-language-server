//! Operation counts of [`split_edited`] on a generated 20k-statement file. The budgets are those
//! of checkpoint 0 (S8): counts, not timings, so that they hold on every machine.

use pgls_test_utils::scale::Dump;

use crate::{COMPARE_BLOCK_BYTES, Operations, split, split_edited};

const STATEMENTS: usize = 20_000;

/// A keystroke that doesn't touch a quote or a comment splits only the statements around it.
#[test]
fn an_ordinary_keystroke_doesnt_split_the_whole_file() {
    let dump = Dump::new(STATEMENTS);
    let previous = split(&dump.sql);
    for at in dump.edit_offsets() {
        let mut edited = dump.sql.clone();
        edited.insert(at, ' ');
        let (result, operations) =
            Operations::during(|| split_edited(&dump.sql, &previous, &edited));
        assert_eq!(result.ranges.len(), STATEMENTS);
        assert_eq!(operations.full_splits, 0, "a space at {at}");
    }
}

/// A keystroke that types a quote can change every statement after it, so the file is split
/// again, but only once.
#[test]
fn a_quote_splits_the_whole_file_once() {
    let dump = Dump::new(STATEMENTS);
    let previous = split(&dump.sql);
    for typed in ['$', '\'', '"', '/', '*', '-', '\\'] {
        let mut edited = dump.sql.clone();
        edited.insert(dump.query(0.5).expect("the dump has queries") + 10, typed);
        let (_, operations) = Operations::during(|| split_edited(&dump.sql, &previous, &edited));
        assert!(
            operations.full_splits <= 1,
            "typing {typed:?} split the file {} times",
            operations.full_splits
        );
    }
}

/// Finding the change compares blocks, and steps through at most a block on each side of the
/// change one byte at a time.
///
/// Known failure: since #1257, `common_suffix` steps through the whole common suffix one byte at
/// a time (checkpoint 0, §4.2: `split_edited` 0.18 → 0.73 ms at 20k statements). The fix is on
/// its way; once it is merged this test stops panicking, which fails it, and `should_panic`
/// goes. The budget stays as it is.
#[test]
#[should_panic(expected = "bytes one at a time")]
fn split_edited_steps_through_the_change_only() {
    let dump = Dump::new(STATEMENTS);
    let previous = split(&dump.sql);
    for at in dump.edit_offsets() {
        let mut edited = dump.sql.clone();
        edited.insert(at, ' ');
        let changed = 1;
        let (_, operations) = Operations::during(|| split_edited(&dump.sql, &previous, &edited));
        let budget = 2 * COMPARE_BLOCK_BYTES + changed;
        assert!(
            operations.bytes_stepped <= budget,
            "a space at {at}: stepped through {} bytes one at a time, budget {budget}",
            operations.bytes_stepped
        );
    }
}
