// Integration tests using wiremock to simulate the Notion API.
//
// How wiremock works:
//   1. `MockServer::start()` spins up a real HTTP server on a random local port.
//   2. You register `Mock` rules — "when you see POST /foo, respond with this JSON".
//   3. Your code makes real HTTP requests to `mock_server.uri()`.
//   4. wiremock intercepts them and returns the canned responses.
//   5. After the test, wiremock verifies all expected calls were made.
//
// `#[tokio::test]` is needed because wiremock is async. However, the functions
// under test use reqwest::blocking (synchronous HTTP). Running blocking code
// directly inside an async context causes a runtime conflict, so we use
// `tokio::task::spawn_blocking` to move blocking calls onto a dedicated thread.

use sb_daily_habits::daily_tracking::get_today_id;
use sb_daily_habits::habits_md::get_hmd;
use sb_daily_habits::daily_habits::{create_daily_habit, habit_exists_today};
use sb_daily_habits::notion_client::NotionClient;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, header, body_partial_json};
use serde_json::json;

// Helper: builds a NotionClient pointed at the mock server.
// Using fixed dummy IDs means tests are self-contained — no real config needed.
fn test_client(mock_server: &MockServer) -> NotionClient {
    NotionClient::new(
        mock_server.uri(),       // points at wiremock, not api.notion.com
        "Bearer test-token",
        "2022-06-28",
        "daily-db-id",
        "habits-master-db-id",
        "habits-db-id",
        "stats-page-id",
    )
}

// ── get_today_id ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_today_id_returns_page_id() {
    // Verifies the happy path: mock returns one result, function returns its ID.
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/databases/daily-db-id/query"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [{ "id": "today-page-id-abc" }]
        })))
        .mount(&mock_server)
        .await;

    let notion = test_client(&mock_server);

    // `spawn_blocking` runs the blocking reqwest call on a dedicated thread,
    // keeping it off the async executor to avoid runtime conflicts.
    let id = tokio::task::spawn_blocking(move || get_today_id(&notion))
        .await.expect("task panicked")
        .expect("Should return today's ID");

    assert_eq!(id, "today-page-id-abc");
}

#[tokio::test]
async fn test_get_today_id_sends_authorization_header() {
    // Verifies the Authorization header is sent — without it Notion returns 401.
    // wiremock only matches this mock if the header is present; otherwise it
    // returns 404, which our code converts to an error.
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/databases/daily-db-id/query"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [{ "id": "page-id" }]
        })))
        .mount(&mock_server)
        .await;

    let notion = test_client(&mock_server);
    tokio::task::spawn_blocking(move || get_today_id(&notion))
        .await.expect("task panicked")
        .expect("Should succeed with correct auth header");
}

#[tokio::test]
async fn test_get_today_id_sends_notion_version_header() {
    // Verifies the Notion-Version header is sent — Notion requires it.
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/databases/daily-db-id/query"))
        .and(header("Notion-Version", "2022-06-28"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [{ "id": "page-id" }]
        })))
        .mount(&mock_server)
        .await;

    let notion = test_client(&mock_server);
    tokio::task::spawn_blocking(move || get_today_id(&notion))
        .await.expect("task panicked")
        .expect("Should succeed with correct Notion-Version header");
}

#[tokio::test]
async fn test_get_today_id_errors_on_empty_results() {
    // Verifies that a valid 200 response with no results produces a meaningful
    // error rather than a panic.
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/databases/daily-db-id/query"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": []
        })))
        .mount(&mock_server)
        .await;

    let notion = test_client(&mock_server);
    let result = tokio::task::spawn_blocking(move || get_today_id(&notion))
        .await.expect("task panicked");

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No day entry"));
}

#[tokio::test]
async fn test_get_today_id_errors_on_401() {
    // Verifies that a 401 Unauthorized response is treated as an error.
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/databases/daily-db-id/query"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "message": "API token is invalid."
        })))
        .mount(&mock_server)
        .await;

    let notion = test_client(&mock_server);
    let result = tokio::task::spawn_blocking(move || get_today_id(&notion))
        .await.expect("task panicked");

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("401"));
}

// ── get_hmd ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_hmd_returns_habits() {
    // Verifies the full round-trip: mock returns two active habits, function
    // returns a vec of (id, name) tuples.
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/databases/habits-master-db-id/query"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [
                {
                    "id": "habit-1",
                    "properties": {
                        "Name": { "title": [{ "plain_text": "Morning Run" }] }
                    }
                },
                {
                    "id": "habit-2",
                    "properties": {
                        "Name": { "title": [{ "plain_text": "Read 30 mins" }] }
                    }
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let notion = test_client(&mock_server);
    let habits = tokio::task::spawn_blocking(move || get_hmd(&notion))
        .await.expect("task panicked")
        .expect("Should return habits");

    assert_eq!(habits.len(), 2);
    assert_eq!(habits[0], ("habit-1".to_string(), "Morning Run".to_string()));
    assert_eq!(habits[1], ("habit-2".to_string(), "Read 30 mins".to_string()));
}

#[tokio::test]
async fn test_get_hmd_returns_empty_vec_when_no_active_habits() {
    // Verifies that zero active habits returns an empty vec, not an error.
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/databases/habits-master-db-id/query"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": []
        })))
        .mount(&mock_server)
        .await;

    let notion = test_client(&mock_server);
    let habits = tokio::task::spawn_blocking(move || get_hmd(&notion))
        .await.expect("task panicked")
        .expect("Should return empty vec");

    assert!(habits.is_empty());
}

#[tokio::test]
async fn test_get_hmd_errors_on_500() {
    // Verifies that a 500 server error from Notion surfaces as an error.
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/databases/habits-master-db-id/query"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    let notion = test_client(&mock_server);
    let result = tokio::task::spawn_blocking(move || get_hmd(&notion))
        .await.expect("task panicked");

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("500"));
}

// ── create_daily_habit ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_daily_habit_posts_to_pages_endpoint() {
    // Verifies the request goes to /pages — the Notion endpoint for creating pages.
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/pages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": "new-page-id"})))
        .mount(&mock_server)
        .await;

    let notion = test_client(&mock_server);
    tokio::task::spawn_blocking(move || {
        create_daily_habit(&notion, "habit-id", "today-id", "Morning Run")
    })
    .await.expect("task panicked")
    .expect("Should succeed");
}

#[tokio::test]
async fn test_create_daily_habit_sends_habit_name() {
    // Verifies the habit name is included in the request body sent to Notion.
    // `body_partial_json` checks that the given JSON is a subset of the body.
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/pages"))
        .and(body_partial_json(json!({
            "properties": {
                "Name": {
                    "title": [{ "text": { "content": "Morning Run" } }]
                }
            }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": "x"})))
        .mount(&mock_server)
        .await;

    let notion = test_client(&mock_server);
    tokio::task::spawn_blocking(move || {
        create_daily_habit(&notion, "habit-id", "today-id", "Morning Run")
    })
    .await.expect("task panicked")
    .expect("Should succeed with correct body");
}

#[tokio::test]
async fn test_create_daily_habit_errors_on_401() {
    // Verifies a 401 from Notion (e.g. expired token) surfaces as an error.
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/pages"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&mock_server)
        .await;

    let notion = test_client(&mock_server);
    let result = tokio::task::spawn_blocking(move || {
        create_daily_habit(&notion, "habit-id", "today-id", "Morning Run")
    })
    .await.expect("task panicked");

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("401"));
}

// ── habit_exists_today (idempotency) ──────────────────────────────────────────

#[tokio::test]
async fn test_habit_exists_today_returns_true_when_entry_found() {
    // Verifies that when Notion returns a result for this habit+day combo,
    // we correctly detect the existing entry and return true.
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/databases/habits-db-id/query"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [{ "id": "existing-entry-id" }]
        })))
        .mount(&mock_server)
        .await;

    let notion = test_client(&mock_server);
    let exists = tokio::task::spawn_blocking(move || {
        habit_exists_today(&notion, "habit-id-123", "today-id-456")
    })
    .await.expect("task panicked")
    .expect("Should succeed");

    assert!(exists, "Should return true when entry already exists");
}

#[tokio::test]
async fn test_habit_exists_today_returns_false_when_no_entry() {
    // Verifies that when Notion returns no results, we correctly return false,
    // meaning it is safe to create a new entry.
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/databases/habits-db-id/query"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": []
        })))
        .mount(&mock_server)
        .await;

    let notion = test_client(&mock_server);
    let exists = tokio::task::spawn_blocking(move || {
        habit_exists_today(&notion, "habit-id-123", "today-id-456")
    })
    .await.expect("task panicked")
    .expect("Should succeed");

    assert!(!exists, "Should return false when no entry exists");
}

#[tokio::test]
async fn test_habit_exists_today_sends_authorization_header() {
    // Verifies the Authorization header is sent on the existence check query.
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/databases/habits-db-id/query"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": []
        })))
        .mount(&mock_server)
        .await;

    let notion = test_client(&mock_server);
    tokio::task::spawn_blocking(move || {
        habit_exists_today(&notion, "habit-id", "today-id")
    })
    .await.expect("task panicked")
    .expect("Should succeed with correct auth header");
}

#[tokio::test]
async fn test_habit_exists_today_errors_on_401() {
    // Verifies that a 401 on the existence check surfaces as an error.
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/databases/habits-db-id/query"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&mock_server)
        .await;

    let notion = test_client(&mock_server);
    let result = tokio::task::spawn_blocking(move || {
        habit_exists_today(&notion, "habit-id", "today-id")
    })
    .await.expect("task panicked");

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("401"));
}

#[tokio::test]
async fn test_create_skipped_when_habit_already_exists() {
    // Verifies the full idempotency flow: when the existence check returns a
    // result, the /pages endpoint is never called. We register the /pages mock
    // with `.expect(0)` — wiremock will FAIL the test if /pages is ever called,
    // proving create was skipped.
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/databases/habits-db-id/query"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [{ "id": "existing-entry" }]
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/pages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": "x"})))
        .expect(0)
        .mount(&mock_server)
        .await;

    let notion = test_client(&mock_server);
    let exists = tokio::task::spawn_blocking(move || {
        habit_exists_today(&notion, "habit-id", "today-id")
    })
    .await.expect("task panicked")
    .expect("Should succeed");

    assert!(exists, "Existence check should return true");
}

#[tokio::test]
async fn test_create_called_when_habit_does_not_exist() {
    // Verifies the other side: when the existence check returns no results,
    // create_daily_habit posts to /pages exactly once.
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/databases/habits-db-id/query"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": []
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/pages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": "new-entry"})))
        .expect(1)
        .mount(&mock_server)
        .await;

    let notion_check = test_client(&mock_server);
    let notion_create = test_client(&mock_server);

    let exists = tokio::task::spawn_blocking(move || {
        habit_exists_today(&notion_check, "habit-id", "today-id")
    })
    .await.expect("task panicked")
    .expect("Check should succeed");

    assert!(!exists);

    tokio::task::spawn_blocking(move || {
        create_daily_habit(&notion_create, "habit-id", "today-id", "Morning Run")
    })
    .await.expect("task panicked")
    .expect("Create should succeed");
}
