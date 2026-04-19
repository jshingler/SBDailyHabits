// `NotionClient` holds the credentials and IDs needed to make Notion API calls.
// Passing it as a parameter instead of reading from the global CONFIG singleton
// means tests can inject their own values (pointing at a mock server) without
// touching the filesystem or environment.
pub struct NotionClient {
    pub base_url: String,
    pub token: String,
    pub version: String,
    pub daily_database_id: String,
    pub habits_master_database_id: String,
    pub habits_database_id: String,
    pub daily_stats_page_id: String,
}

impl NotionClient {
    // Convenience constructor so tests can build one in a single expression.
    pub fn new(
        base_url: impl Into<String>,
        token: impl Into<String>,
        version: impl Into<String>,
        daily_database_id: impl Into<String>,
        habits_master_database_id: impl Into<String>,
        habits_database_id: impl Into<String>,
        daily_stats_page_id: impl Into<String>,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            token: token.into(),
            version: version.into(),
            daily_database_id: daily_database_id.into(),
            habits_master_database_id: habits_master_database_id.into(),
            habits_database_id: habits_database_id.into(),
            daily_stats_page_id: daily_stats_page_id.into(),
        }
    }
}
