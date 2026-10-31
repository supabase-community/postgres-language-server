// The helpers read the parse trees and function bodies of the client's SQL, so a panic on
// unexpected input takes down the server for every connected editor.
#![warn(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

pub mod diagnostics;
pub mod utils;
