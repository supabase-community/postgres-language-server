/// Compared in blocks first, which is much faster than byte by byte.
const COMPARE_BLOCK_BYTES: usize = 64;

/// The part of a text that an edit changed: the bytes `start..old_end` of the old text became
/// `start..new_end` of the new one, and the text before and after them is the same. The change is
/// the text between the longest common prefix and the longest common suffix that doesn't overlap
/// it, both ends on a char boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextChange {
    pub start: usize,
    pub old_end: usize,
    pub new_end: usize,
}

impl TextChange {
    pub fn between(old: &str, new: &str) -> Self {
        let (old_bytes, new_bytes) = (old.as_bytes(), new.as_bytes());
        let mut start = Self::common_prefix(old_bytes, new_bytes);
        while !old.is_char_boundary(start) || !new.is_char_boundary(start) {
            start -= 1;
        }
        // The suffix is searched only after the prefix, so the two don't overlap: in `aa` → `aaa`
        // both would otherwise cover the whole of `old`.
        let max_suffix = old.len().min(new.len()) - start;
        let mut suffix = Self::common_suffix(
            old_bytes.get(old.len() - max_suffix..).unwrap_or_default(),
            new_bytes.get(new.len() - max_suffix..).unwrap_or_default(),
        );
        while !old.is_char_boundary(old.len() - suffix) || !new.is_char_boundary(new.len() - suffix)
        {
            suffix -= 1;
        }
        Self {
            start,
            old_end: old.len() - suffix,
            new_end: new.len() - suffix,
        }
    }

    /// Whether the texts are the same.
    pub fn is_empty(&self) -> bool {
        self.old_end == self.start && self.new_end == self.start
    }

    /// The length of the longest common prefix of `a` and `b`.
    fn common_prefix(a: &[u8], b: &[u8]) -> usize {
        let equal = a
            .as_chunks::<COMPARE_BLOCK_BYTES>()
            .0
            .iter()
            .zip(b.as_chunks::<COMPARE_BLOCK_BYTES>().0)
            .take_while(|(a, b)| a == b)
            .count()
            * COMPARE_BLOCK_BYTES;
        equal
            + a.iter()
                .zip(b)
                .skip(equal)
                .take_while(|(a, b)| a == b)
                .count()
    }

    /// The length of the longest common suffix of `a` and `b`, which have the same length.
    fn common_suffix(a: &[u8], b: &[u8]) -> usize {
        // `as_rchunks` puts the remainder at the start; its chunks run to the end.
        let equal = a
            .as_rchunks::<COMPARE_BLOCK_BYTES>()
            .1
            .iter()
            .rev()
            .zip(b.as_rchunks::<COMPARE_BLOCK_BYTES>().1.iter().rev())
            .take_while(|(a, b)| a == b)
            .count()
            * COMPARE_BLOCK_BYTES;
        // Compare the rest on the bytes before the equal blocks: `skip(equal)` on the reversed,
        // zipped bytes would step through every byte of the blocks.
        let (Some(a), Some(b)) = (a.get(..a.len() - equal), b.get(..b.len() - equal)) else {
            return equal;
        };
        equal
            + a.iter()
                .rev()
                .zip(b.iter().rev())
                .take_while(|(a, b)| a == b)
                .count()
    }
}

#[cfg(test)]
mod tests {
    use super::TextChange;

    fn change(old: &str, new: &str) -> (usize, usize, usize) {
        let change = TextChange::between(old, new);
        (change.start, change.old_end, change.new_end)
    }

    #[test]
    fn between_finds_the_changed_bytes_on_char_boundaries() {
        let long = "select 1;\n".repeat(20);
        let n = long.len();
        assert!(TextChange::between("", "").is_empty());
        assert!(TextChange::between(&long, &long).is_empty());
        assert_eq!(change(&long, "select 2;"), (7, n, 9));
        assert_eq!(
            change(&format!("{long}x"), &format!("{long}y")),
            (n, n + 1, n + 1)
        );
        // The prefix comes first, so the suffix doesn't overlap it.
        assert_eq!(change("aa", "aaa"), (2, 2, 3));
        // `ä` and `ö` share their first byte, so the change starts at that byte.
        assert_eq!(
            change(&format!("{long}ä{long}"), &format!("{long}ö{long}")),
            (n, n + 2, n + 2)
        );
    }
}
