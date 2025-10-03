use crate::{TextRange, TextSize};

#[derive(Debug, PartialEq)]
enum AdjustmentDirection {
    Lengthened,
    Shortened,
}

#[derive(Debug)]
struct AdjustmentMarker {
    original_range: TextRange,
    adjusted_range: TextRange,
    replacement_txt: String,
    registered_offset_at_start: isize,
    adjustment_direction: AdjustmentDirection,
    range_size_difference: TextSize,
}

impl AdjustmentMarker {
    fn new(original_range: TextRange, replacement_txt: &str) -> Self {
        let og_range_len = usize::from(original_range.len());

        let (range_size_difference, adjustment_direction) = if og_range_len <= replacement_txt.len()
        {
            (
                replacement_txt.len() - og_range_len,
                AdjustmentDirection::Lengthened,
            )
        } else {
            (
                og_range_len - replacement_txt.len(),
                AdjustmentDirection::Shortened,
            )
        };

        AdjustmentMarker {
            original_range,
            adjustment_direction,
            replacement_txt: replacement_txt.into(),
            range_size_difference: TextSize::new(range_size_difference.try_into().unwrap()),

            // will be calculated during `.build()`
            adjusted_range: original_range,
            registered_offset_at_start: 0,
        }
    }

    /// If the original text is `select $1 from $2` and the adjusted text is `select email from auth.x`,
    /// and you index into the `x` in the adjusted string, this will "correct" the adjusted range
    /// as if it had the original length ('$2', so a length of 2).
    ///
    /// So, the resulting `TextSize` *will* be corrected "to the left" as though we indexed onto the `u`, since `$2` has a range
    /// of two characters.
    ///
    /// The TextSize *will still* consider the offsets of previous replacements (3 to the right, since `email` is longer than `$1`).
    fn adjusted_end_within_clamped_range(&self, position: TextSize) -> TextSize {
        let clamped_end = self.adjusted_range.end() - self.range_size_difference;
        std::cmp::min(position, clamped_end - TextSize::new(1))
    }
}

/// Builder for creating a `TextRangeReplacement` that tracks text range adjustments.
///
/// This builder allows you to register multiple text replacements and their effects on ranges,
/// then build a tracker that can map positions between the original and adjusted text.
#[derive(Debug)]
pub struct TextRangeReplacementBuilder {
    markers: Vec<AdjustmentMarker>,
    text: String,
}

impl TextRangeReplacementBuilder {
    /// Creates a new empty builder for range adjustments tracking.
    pub fn new(text: &str) -> Self {
        Self {
            markers: vec![],
            text: text.to_string(),
        }
    }

    /// Registers a text replacement that affects range positions.
    ///
    /// #### Arguments
    ///
    /// * `original_range` - The range in the original text that will be replaced
    /// * `replacement_text` - The text that will replace the content in the original range
    pub fn replace_range(&mut self, original_range: TextRange, replacement_text: &str) {
        if usize::from(original_range.len()) == replacement_text.len() {
            // if the replacement text is the same length as the to-replace range,
            // we can just immediately apply the replacement.
            let range: std::ops::Range<usize> = original_range.into();
            self.text.replace_range(range, replacement_text);
            return;
        }

        self.markers
            .push(AdjustmentMarker::new(original_range, replacement_text));
    }

    /// Builds the range adjustments tracker from all registered adjustments.
    ///
    /// The adjustments are processed in order of their starting positions in the original text.
    /// Currently only supports lengthening adjustments (where replacement text is longer
    /// than the original range).
    pub fn build(mut self) -> TextRangeReplacement {
        self.markers.sort_by_key(|r| r.original_range.start());

        let mut total_offset: isize = 0;

        for marker in self.markers.iter_mut() {
            match marker.adjustment_direction {
                AdjustmentDirection::Lengthened => {
                    marker.adjusted_range = if total_offset >= 0 {
                        let to_add = TextSize::new(total_offset.abs().try_into().unwrap());
                        TextRange::new(
                            marker.original_range.start() + to_add,
                            marker.original_range.end() + to_add + marker.range_size_difference,
                        )
                    } else {
                        let to_sub = TextSize::new(total_offset.abs().try_into().unwrap());
                        TextRange::new(
                            marker.original_range.start().checked_sub(to_sub).unwrap(),
                            marker.original_range.end().checked_sub(to_sub).unwrap()
                                + marker.range_size_difference,
                        )
                    };

                    marker.registered_offset_at_start = total_offset;
                    total_offset += isize::from(marker.range_size_difference);
                }
                AdjustmentDirection::Shortened => {
                    marker.adjusted_range = if total_offset >= 0 {
                        let to_add = TextSize::new(total_offset.abs().try_into().unwrap());
                        TextRange::new(
                            marker.original_range.start() + to_add,
                            marker.original_range.end() + to_add - marker.range_size_difference,
                        )
                    } else {
                        let to_sub = TextSize::new(total_offset.abs().try_into().unwrap());
                        TextRange::new(
                            marker.original_range.start().checked_sub(to_sub).unwrap(),
                            marker.original_range.end().checked_sub(to_sub).unwrap()
                                - marker.range_size_difference,
                        )
                    };

                    marker.registered_offset_at_start = total_offset;
                    total_offset -= isize::from(marker.range_size_difference);
                }
            }
        }

        for marker in self.markers.iter().rev() {
            let std_range: std::ops::Range<usize> = marker.original_range.into();
            self.text
                .replace_range(std_range, marker.replacement_txt.as_str());
        }

        TextRangeReplacement {
            markers: self.markers,
            text: self.text,
        }
    }
}

/// Tracks text range adjustments and provides mapping between original and adjusted positions.
///
/// This struct maintains information about how text ranges have been modified (typically by
/// replacing placeholders with actual values) and can map positions from the adjusted text
/// back to their corresponding positions in the original text.
///
/// # Example
///
/// If you have original text `"select $1 from $2"` and replace `$1` with `email` and
/// `$2` with `auth.users`, this tracker can map positions in the adjusted text
/// `"select email from auth.users"` back to positions in the original text.
#[derive(Debug)]
pub struct TextRangeReplacement {
    markers: Vec<AdjustmentMarker>,
    text: String,
}

impl TextRangeReplacement {
    /// Returns the adjusted text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Maps a position in the adjusted text back to its corresponding position in the original text.
    ///
    ///
    /// #### Example
    ///
    /// If the original text was `"select $1 from $2"` and it was adjusted to
    /// `"select email from auth.users"`, then calling this method with the position
    /// of `'m'` in `"email"` would return the position of `'1'` in the original text, and using the position of `'e'` in
    /// `'email'` will give you the first `'$'`.
    ///
    /// The position tracker "clamps" positions, so if you call it with the position of `'l'` in `'email'` ,
    /// you'd still get the position of `'1'`.
    ///
    /// The position of `'f'` in `'from'` will give you the position of `'f'` in `'from'`.
    pub fn to_original_position(&self, adjusted_position: TextSize) -> TextSize {
        if let Some(marker) = self
            .markers
            .iter()
            .rev()
            .find(|marker| adjusted_position >= marker.adjusted_range.start())
        {
            if marker.adjustment_direction == AdjustmentDirection::Lengthened {
                if adjusted_position >= marker.adjusted_range.end() {
                    offset_reverted(adjusted_position, marker.registered_offset_at_start)
                        .checked_sub(marker.range_size_difference)
                        .unwrap()
                } else {
                    let clamped = marker.adjusted_end_within_clamped_range(adjusted_position);
                    offset_reverted(clamped, marker.registered_offset_at_start)
                }
            } else if adjusted_position < marker.adjusted_range.end() {
                offset_reverted(adjusted_position, marker.registered_offset_at_start)
            } else {
                offset_reverted(adjusted_position, marker.registered_offset_at_start)
                    .checked_add(marker.range_size_difference)
                    .unwrap()
            }
        } else {
            adjusted_position
        }
    }

    /// Maps a range in the adjusted text back to its corresponding range in the original text.
    #[allow(dead_code)]
    pub fn to_original_range(&self, adjusted_range: TextRange) -> TextRange {
        // todo(@juleswritescode): optimize with windows
        TextRange::new(
            self.to_original_position(adjusted_range.start()),
            self.to_original_position(adjusted_range.end()),
        )
    }
}

fn offset_reverted(position: TextSize, offset: isize) -> TextSize {
    let absolute = TextSize::new(offset.abs().try_into().unwrap());
    if offset >= 0 {
        position.checked_sub(absolute).unwrap()
    } else {
        position.checked_add(absolute).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::TextSize;

    use crate::text_range_replacement::TextRangeReplacementBuilder;

    #[test]
    fn tracks_lengthening_adjustments() {
        let sql = "select $1 from $2 where $3 = $4 limit 5;";

        let range_1: std::ops::Range<usize> = 7..9; // $1
        let range_2: std::ops::Range<usize> = 15..17; // $2
        let range_3: std::ops::Range<usize> = 24..26; // $3
        let range_4: std::ops::Range<usize> = 29..31; // $4
        let og_end = sql.len();

        let mut replacement_builder = TextRangeReplacementBuilder::new(sql);

        let replacement_4 = "'00000000-0000-0000-0000-000000000000'";
        let replacement_3 = "id";
        let replacement_2 = "auth.users";
        let replacement_1 = "email";

        // start in the middle – the builder can deal with unordered replacements
        replacement_builder.replace_range(range_2.clone().try_into().unwrap(), replacement_2);
        replacement_builder.replace_range(range_4.clone().try_into().unwrap(), replacement_4);
        replacement_builder.replace_range(range_1.clone().try_into().unwrap(), replacement_1);
        replacement_builder.replace_range(range_3.clone().try_into().unwrap(), replacement_3);

        let text_replacement = replacement_builder.build();

        assert_eq!(
            text_replacement.text(),
            "select email from auth.users where id = '00000000-0000-0000-0000-000000000000' limit 5;"
        );

        let repl_range_1 = 7..12; // email
        let repl_range_2 = 18..28; // auth.users
        let repl_range_3 = 35..37; // id
        let repl_range_4 = 40..78; // '00000000-0000-0000-0000-000000000000'

        // |select |email from auth.users where id = '00000000-0000-0000-0000-000000000000' limit 5;
        // maps to
        // |select |$1 from $2 where $3 = $4 limit 5;
        for i in 0..repl_range_1.clone().start {
            let between_og_0_1 = 0..range_1.start;
            let adjusted =
                text_replacement.to_original_position(TextSize::new(i.try_into().unwrap()));
            assert!(between_og_0_1.contains(&usize::from(adjusted)));
        }

        // select |email| from auth.users where id = '00000000-0000-0000-0000-000000000000' limit 5;
        // maps to
        // select |$1| from $2 where $3 = $4 limit 5;
        for i in repl_range_1.clone() {
            let pos = TextSize::new(i.try_into().unwrap());
            let og_pos = text_replacement.to_original_position(pos);
            assert!(range_1.contains(&usize::from(og_pos)));
        }

        // select email| from |auth.users where id = '00000000-0000-0000-0000-000000000000' limit 5;
        // maps to
        // select $1| from |$2 where $3 = $4 limit 5;
        for i in repl_range_1.end..repl_range_2.clone().start {
            let between_og_1_2 = range_1.end..range_2.start;
            let adjusted =
                text_replacement.to_original_position(TextSize::new(i.try_into().unwrap()));
            assert!(between_og_1_2.contains(&usize::from(adjusted)));
        }

        // select email from |auth.users| where id = '00000000-0000-0000-0000-000000000000' limit 5;
        // maps to
        // select $1 from |$2| where $3 = $4 limit 5;
        for i in repl_range_2.clone() {
            let pos = TextSize::new(i.try_into().unwrap());
            let og_pos = text_replacement.to_original_position(pos);
            assert!(range_2.contains(&usize::from(og_pos)));
        }

        // select email from auth.users| where |id = '00000000-0000-0000-0000-000000000000' limit 5;
        // maps to
        // select $1 from $2| where |$3 = $4 limit 5;
        for i in repl_range_2.end..repl_range_3.clone().start {
            let between_og_2_3 = range_2.end..range_3.start;
            let adjusted =
                text_replacement.to_original_position(TextSize::new(i.try_into().unwrap()));
            assert!(between_og_2_3.contains(&usize::from(adjusted)));
        }

        // select email from auth.users where |id| = '00000000-0000-0000-0000-000000000000' limit 5;
        // maps to
        // select $1 from $2 where |$3| = $4 limit 5;
        //
        // NOTE: this isn't even hanlded by the tracker, since `id` has the same length as `$3`
        for i in repl_range_3.clone() {
            let pos = TextSize::new(i.try_into().unwrap());
            let og_pos = text_replacement.to_original_position(pos);
            assert!(range_3.contains(&usize::from(og_pos)));
        }

        // select email from auth.users where id| = |'00000000-0000-0000-0000-000000000000' limit 5;
        // maps to
        // select $1 from $2 where $3| = |$4 limit 5;
        for i in repl_range_3.end..repl_range_4.clone().start {
            let between_og_3_4 = range_3.end..range_4.start;
            let adjusted =
                text_replacement.to_original_position(TextSize::new(i.try_into().unwrap()));
            assert!(between_og_3_4.contains(&usize::from(adjusted)));
        }

        // select email from auth.users where id = |'00000000-0000-0000-0000-000000000000'| limit 5;
        // maps to
        // select $1 from $2 where $3 = |$4| limit 5;
        for i in repl_range_4.clone() {
            let pos = TextSize::new(i.try_into().unwrap());
            let og_pos = text_replacement.to_original_position(pos);
            assert!(range_4.contains(&usize::from(og_pos)));
        }

        // select email from auth.users where id = '00000000-0000-0000-0000-000000000000'| limit 5;|
        // maps to
        // select $1 from $2 where $3 = $4| limit 5;|
        for i in repl_range_4.end..text_replacement.text.len() {
            let between_og_4_end = range_4.end..og_end;
            let adjusted =
                text_replacement.to_original_position(TextSize::new(i.try_into().unwrap()));
            assert!(between_og_4_end.contains(&usize::from(adjusted)));
        }
    }

    #[test]
    fn tracks_shortening_adjustments() {
        let sql = "select :my_param1 from :my_param2 where :my_param3 = :my_param4 limit 5;";

        let range_1: std::ops::Range<usize> = 7..17; // :my_param1
        let range_2: std::ops::Range<usize> = 23..33; // :my_param2
        let range_3: std::ops::Range<usize> = 40..50; // :my_param3
        let range_4: std::ops::Range<usize> = 53..63; // :my_param4
        let og_end = sql.len();

        let mut replacement_builder = TextRangeReplacementBuilder::new(sql);

        let replacement_1 = /* select */ "email";
        let replacement_2 = /* from */ "auth.users"; // (same length)
        let replacement_3 = /* where */ "name";
        let replacement_4 = /* = */ "timo"; /* limit 5; */

        // start in the middle – the builder can deal with unordered replacements
        replacement_builder.replace_range(range_2.clone().try_into().unwrap(), replacement_2);
        replacement_builder.replace_range(range_4.clone().try_into().unwrap(), replacement_4);
        replacement_builder.replace_range(range_1.clone().try_into().unwrap(), replacement_1);
        replacement_builder.replace_range(range_3.clone().try_into().unwrap(), replacement_3);

        let text_replacement = replacement_builder.build();

        assert_eq!(
            text_replacement.text(),
            "select email from auth.users where name = timo limit 5;"
        );

        let repl_range_1 = 7..12; // email
        let repl_range_2 = 18..28; // auth.users
        let repl_range_3 = 35..39; // name
        let repl_range_4 = 42..46; // timo

        // |select |email from auth.users where name = timo limit 5;
        // maps to
        // |select |:my_param1 from :my_param2 where :my_param3 = :my_param4 limit 5;
        for i in 0..repl_range_1.clone().start {
            let between_og_0_1 = 0..range_1.start;
            let adjusted =
                text_replacement.to_original_position(TextSize::new(i.try_into().unwrap()));
            assert!(between_og_0_1.contains(&usize::from(adjusted)));
        }

        // select |email| from auth.users where name = timo limit 5;
        // maps to
        // select |:my_param1| from :my_param2 where :my_param3 = :my_param4 limit 5;
        for i in repl_range_1.clone() {
            let pos = TextSize::new(i.try_into().unwrap());
            let og_pos = text_replacement.to_original_position(pos);
            assert!(range_1.contains(&usize::from(og_pos)));
        }

        // select email| from |auth.users where name = timo limit 5;
        // maps to
        // select :my_param1| from |:my_param2 where :my_param3 = :my_param4 limit 5;
        for i in repl_range_1.end..repl_range_2.clone().start {
            let between_og_1_2 = range_1.end..range_2.start;
            let adjusted =
                text_replacement.to_original_position(TextSize::new(i.try_into().unwrap()));
            assert!(between_og_1_2.contains(&usize::from(adjusted)));
        }

        // select email from |auth.users| where name = timo limit 5;
        // maps to
        // select :my_param1 from |:my_param2| where :my_param3 = :my_param4 limit 5;
        for i in repl_range_2.clone() {
            let pos = TextSize::new(i.try_into().unwrap());
            let og_pos = text_replacement.to_original_position(pos);
            assert!(range_2.contains(&usize::from(og_pos)));
        }

        // select email from auth.users| where |name = timo limit 5;
        // maps to
        // select :my_param1 from :my_param2| where |:my_param3 = :my_param4 limit 5;
        for i in repl_range_2.end..repl_range_3.clone().start {
            let between_og_2_3 = range_2.end..range_3.start;
            let adjusted =
                text_replacement.to_original_position(TextSize::new(i.try_into().unwrap()));
            assert!(between_og_2_3.contains(&usize::from(adjusted)));
        }

        // select email from auth.users where |name| = timo limit 5;
        // maps to
        // select :my_param1 from :my_param2 where |:my_param3| = :my_param4 limit 5;
        for i in repl_range_3.clone() {
            let pos = TextSize::new(i.try_into().unwrap());
            let og_pos = text_replacement.to_original_position(pos);
            assert!(range_3.contains(&usize::from(og_pos)));
        }

        // select email from auth.users where name| = |timo limit 5;
        // maps to
        // select :my_param1 from :my_param2 where :my_param3| = |:my_param4 limit 5;
        for i in repl_range_3.end..repl_range_4.clone().start {
            let between_og_3_4 = range_3.end..range_4.start;
            let adjusted =
                text_replacement.to_original_position(TextSize::new(i.try_into().unwrap()));
            assert!(between_og_3_4.contains(&usize::from(adjusted)));
        }

        // select email from auth.users where name = |timo| limit 5;
        // maps to
        // select :my_param1 from :my_param2 where :my_param3 = |:my_param4| limit 5;
        for i in repl_range_4.clone() {
            let pos = TextSize::new(i.try_into().unwrap());
            let og_pos = text_replacement.to_original_position(pos);
            assert!(range_4.contains(&usize::from(og_pos)));
        }

        // select email from auth.users where name = timo| limit 5;|
        // maps to
        // select :my_param1 from :my_param2 where :my_param3 = :my_param4| limit 5;|
        for i in repl_range_4.end..text_replacement.text.len() {
            let between_og_4_end = range_4.end..og_end;
            let adjusted =
                text_replacement.to_original_position(TextSize::new(i.try_into().unwrap()));
            assert!(between_og_4_end.contains(&usize::from(adjusted)));
        }
    }

    #[test]
    fn tracks_mixed_adjustments() {
        let sql = "select :my_param1 from $2 where $3 = :my_param4 limit 5;";

        let range_1: std::ops::Range<usize> = 7..17; // :my_param1
        let range_2: std::ops::Range<usize> = 23..25; // $2
        let range_3: std::ops::Range<usize> = 32..34; // $3
        let range_4: std::ops::Range<usize> = 37..47; // :my_param4
        let og_end = sql.len();

        let mut replacement_builder = TextRangeReplacementBuilder::new(sql);

        let replacement_1 = "email"; // replacement is shorter
        let replacement_2 = "auth.users"; // replacement is longer
        let replacement_3 = "id"; // replacement is same length
        let replacement_4 = "'00000000-0000-0000-0000-000000000000'"; // replacement is longer

        // start in the middle – the builder can deal with unordered replacements
        replacement_builder.replace_range(range_2.clone().try_into().unwrap(), replacement_2);
        replacement_builder.replace_range(range_4.clone().try_into().unwrap(), replacement_4);
        replacement_builder.replace_range(range_1.clone().try_into().unwrap(), replacement_1);
        replacement_builder.replace_range(range_3.clone().try_into().unwrap(), replacement_3);

        let text_replacement = replacement_builder.build();

        assert_eq!(
            text_replacement.text(),
            "select email from auth.users where id = '00000000-0000-0000-0000-000000000000' limit 5;"
        );

        let repl_range_1 = 7..12; // email
        let repl_range_2 = 18..28; // auth.users
        let repl_range_3 = 35..37; // id
        let repl_range_4 = 40..78; // '00000000-0000-0000-0000-000000000000'

        // |select |email from auth.users where id = '00000000-0000-0000-0000-000000000000' limit 5;
        // maps to
        // |select |:my_param1 from $2 where $3 = :my_param4 limit 5;
        for i in 0..repl_range_1.clone().start {
            let between_og_0_1 = 0..range_1.start;
            let adjusted =
                text_replacement.to_original_position(TextSize::new(i.try_into().unwrap()));
            assert!(between_og_0_1.contains(&usize::from(adjusted)));
        }

        // select |email| from auth.users where id = '00000000-0000-0000-0000-000000000000' limit 5;
        // maps to
        // select |:my_param1| from $2 where $3 = :my_param4 limit 5;
        for i in repl_range_1.clone() {
            let pos = TextSize::new(i.try_into().unwrap());
            let og_pos = text_replacement.to_original_position(pos);
            assert!(range_1.contains(&usize::from(og_pos)));
        }

        // select email| from |auth.users where id = '00000000-0000-0000-0000-000000000000' limit 5;
        // maps to
        // select :my_param1| from |$2 where $3 = :my_param4 limit 5;
        for i in repl_range_1.end..repl_range_2.clone().start {
            let between_og_1_2 = range_1.end..range_2.start;
            let adjusted =
                text_replacement.to_original_position(TextSize::new(i.try_into().unwrap()));
            assert!(between_og_1_2.contains(&usize::from(adjusted)));
        }

        // select email from |auth.users| where id = '00000000-0000-0000-0000-000000000000' limit 5;
        // maps to
        // select :my_param1 from |$2| where $3 = :my_param4 limit 5;
        for i in repl_range_2.clone() {
            let pos = TextSize::new(i.try_into().unwrap());
            let og_pos = text_replacement.to_original_position(pos);
            assert!(range_2.contains(&usize::from(og_pos)));
        }

        // select email from auth.users| where |id = '00000000-0000-0000-0000-000000000000' limit 5;
        // maps to
        // select :my_param1 from $2| where |$3 = :my_param4 limit 5;
        for i in repl_range_2.end..repl_range_3.clone().start {
            let between_og_2_3 = range_2.end..range_3.start;
            let adjusted =
                text_replacement.to_original_position(TextSize::new(i.try_into().unwrap()));
            assert!(between_og_2_3.contains(&usize::from(adjusted)));
        }

        // select email from auth.users where |id| = '00000000-0000-0000-0000-000000000000' limit 5;
        // maps to
        // select :my_param1 from $2 where |$3| = :my_param4 limit 5;
        for i in repl_range_3.clone() {
            let pos = TextSize::new(i.try_into().unwrap());
            let og_pos = text_replacement.to_original_position(pos);
            assert!(range_3.contains(&usize::from(og_pos)));
        }

        // select email from auth.users where id| = |'00000000-0000-0000-0000-000000000000' limit 5;
        // maps to
        // select :my_param1 from $2 where $3| = |:my_param4 limit 5;
        for i in repl_range_3.end..repl_range_4.clone().start {
            let between_og_3_4 = range_3.end..range_4.start;
            let adjusted =
                text_replacement.to_original_position(TextSize::new(i.try_into().unwrap()));
            assert!(between_og_3_4.contains(&usize::from(adjusted)));
        }

        // select email from auth.users where id = |'00000000-0000-0000-0000-000000000000'| limit 5;
        // maps to
        // select :my_param1 from $2 where $3 = |:my_param4| limit 5;
        for i in repl_range_4.clone() {
            let pos = TextSize::new(i.try_into().unwrap());
            let og_pos = text_replacement.to_original_position(pos);
            assert!(range_4.contains(&usize::from(og_pos)));
        }

        // select email from auth.users where id = '00000000-0000-0000-0000-000000000000'| limit 5;|
        // maps to
        // select :my_param1 from $2 where $3 = :my_param4| limit 5;|
        for i in repl_range_4.end..sql.len() {
            let between_og_4_end = range_4.end..og_end;
            let adjusted =
                text_replacement.to_original_position(TextSize::new(i.try_into().unwrap()));
            assert!(between_og_4_end.contains(&usize::from(adjusted)));
        }
    }
}
