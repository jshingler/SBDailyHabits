// Custom error type for SBDailyHabits.
//
// Why typed errors instead of Box<dyn Error>?
//   Box<dyn Error> is opaque — callers can't inspect what went wrong without
//   parsing the error message string. A typed enum lets callers match on the
//   specific variant and handle each case differently. It also self-documents
//   every failure mode a function can produce.
//
// `thiserror::Error` is a derive macro that generates the `std::error::Error`
// trait impl for you, plus the `Display` impl from the `#[error("...")]`
// attribute. Without thiserror you'd write ~30 lines of boilerplate by hand.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum HabitsError {
    // Notion returned a non-2xx HTTP status. We capture both the status code
    // and the response body so error messages are actionable.
    #[error("Notion API error {status}: {body}")]
    NotionApi { status: u16, body: String },

    // Today's "Day" page was not found in the Daily Tracking database.
    // This happens if you run the program before creating today's page in Notion.
    #[error("No day entry found in Notion for today. Did you create today's page?")]
    NoDayEntry,

    // The Notion response JSON was missing the expected "results" array.
    // Usually means the API shape changed or the wrong endpoint was called.
    #[error("Notion response missing 'results' array")]
    MissingResultsArray,

    // `#[from]` tells thiserror to generate a `From<reqwest::Error>` impl
    // automatically. This means the `?` operator on a reqwest call will
    // convert the error into HabitsError::Http without any extra code.
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    // Same pattern for serde_json parse errors — `?` on from_str() auto-converts.
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    // Config loading errors are wrapped in a Box because the underlying error
    // type (from serde-java-properties) isn't known at compile time.
    #[error("Failed to load configuration: {0}")]
    Config(String),
}
