//! Terminal search utilities for Termarx.

mod engine;
mod matcher;
mod state;

pub use engine::{SearchConfig, SearchEngine, SearchMode};
pub use matcher::{SearchMatch, SearchResults};
pub use state::SearchState;
