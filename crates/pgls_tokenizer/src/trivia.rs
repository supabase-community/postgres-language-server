use crate::TokenKind;
use crate::class::CharClass;
use crate::cursor::Cursor;

impl Cursor<'_> {
    // see: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L211-L213
    // non_newline		[^\n\r]
    // comment			("--"{non_newline}*)
    pub(crate) fn line_comment(&mut self) -> TokenKind {
        self.bump();

        self.eat_while(|c| !CharClass::Newline.contains(c));
        TokenKind::LineComment
    }

    // see: https://github.com/postgres/postgres/blob/db0c96cc18aec417101e37e59fcc53d4bf647915/src/backend/parser/scan.l#L324-L344
    pub(crate) fn block_comment(&mut self) -> TokenKind {
        self.bump();

        let mut depth = 1usize;
        while let Some(c) = self.bump() {
            match c {
                '/' if self.first() == '*' => {
                    self.bump();
                    depth += 1;
                }
                '*' if self.first() == '/' => {
                    self.bump();
                    depth -= 1;
                    if depth == 0 {
                        // This block comment is closed, so for a construction like "/* */ */"
                        // there will be a successfully parsed block comment "/* */"
                        // and " */" will be processed separately.
                        break;
                    }
                }
                _ => (),
            }
        }

        TokenKind::BlockComment {
            terminated: depth == 0,
        }
    }

    // invariant: we care about the number of consecutive newlines so we count them.
    //
    // Postgres considers a DOS-style \r\n sequence as two successive newlines, but we care about
    // logical line breaks and consider \r\n as one logical line break
    pub(crate) fn line_ending_sequence(&mut self, prev: char) -> TokenKind {
        // already consumed first line ending character (\n or \r)
        let mut line_breaks = 1;

        // started with \r, check if it's part of \r\n
        if prev == '\r' && self.first() == '\n' {
            // consume the \n - \r\n still counts as 1 logical line break
            self.bump();
        }

        // continue checking for more line endings
        loop {
            match self.first() {
                '\r' if self.second() == '\n' => {
                    self.bump(); // consume \r
                    self.bump(); // consume \n
                    line_breaks += 1;
                }
                '\n' => {
                    self.bump();
                    line_breaks += 1;
                }
                '\r' => {
                    self.bump();
                    line_breaks += 1;
                }
                _ => break,
            }
        }

        TokenKind::LineEnding { count: line_breaks }
    }
}
