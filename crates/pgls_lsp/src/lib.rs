// The server handles data from the client, the database, the file system and config files, so a
// panic on unexpected input takes down the server for every connected editor.
#![warn(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

mod adapters;
mod capabilities;
mod configuration;
mod diagnostics;
mod documents;
mod handlers;
mod server;
mod session;
mod utils;

pub use crate::server::{LSPServer, ServerConnection, ServerFactory};
