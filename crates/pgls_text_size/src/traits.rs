use crate::TextSize;

use priv_in_pub::Sealed;
mod priv_in_pub {
    pub trait Sealed {}
}

/// Primitives with a textual length that can be passed to [`TextSize::of`].
pub trait TextLen: Copy + Sealed {
    /// The textual length of this primitive.
    fn text_len(self) -> TextSize;
}

impl Sealed for &'_ str {}
impl TextLen for &'_ str {
    #[inline]
    fn text_len(self) -> TextSize {
        // Texts are limited to 4 GiB (`files.maxSize` rejects much smaller files).
        debug_assert!(u32::try_from(self.len()).is_ok(), "text longer than 4 GiB");
        TextSize::new(u32::try_from(self.len()).unwrap_or(u32::MAX))
    }
}

impl Sealed for &'_ String {}
impl TextLen for &'_ String {
    #[inline]
    fn text_len(self) -> TextSize {
        self.as_str().text_len()
    }
}

impl Sealed for char {}
impl TextLen for char {
    #[inline]
    fn text_len(self) -> TextSize {
        (self.len_utf8() as u32).into()
    }
}
