
use log::{info, error};
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::json;
use crate::config::CONFIG;

use chrono::{Datelike, Local};

pub fn get_today_id() -> String {
    // Get the current date
    let today = Local::now().with_timezone(&Local);

    // Format the date as a string
    let formatted_date = format!("{}-{:02}-{:02}", today.year(), today.month(), today.day());

    // info!("Formatted date: {}", &formatted_date);

    let query = json!({
  "filter": {
    "and": [
      {
        "timestamp": "created_time",
        "created_time": {
          "equals": formatted_date
        }
      }
    ]
  },
  "sorts": [
    {
      "property": "Date",
      "direction": "descending"
    }
  ]
});

    // info!("Filter: {}", query);

    let url = &CONFIG.notion.url;
    let notion_database_url = format!("{}/databases/{}/query", &url, &CONFIG.notion.daily_database_id);

    let http_client = Client::new();
    let http_result = http_client.post(notion_database_url )
        .header("Authorization", &CONFIG.notion.token)
        .header("Notion-Version", &CONFIG.notion.version)
        .header("Content-Type", "application/json")
        .json(&query)
        .send();

    let mut today_id = String::from("");
    if http_result.is_ok() {
        let result_json = http_result.ok().unwrap().text().unwrap();
        let parsed: ApiResponse = serde_json::from_str(&result_json).unwrap();
        // Observation:  http_result can be okay and have 0 results in array.
        // this results in an panic with array bounds problem.
        // Investigate:  How to retry this method after waiting.
        today_id = String::from(&parsed.results[0].id);
        // info!("Response: {:#?}", parsed);
        // today_id = today_id;
        info!("ID: {:#?}",today_id);
    } else if let Err(e) = http_result {
        // Log the error from the HTTP request
        error!("HTTP request failed: {:?}", e);
    }


    today_id
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    object: String,
    results: Vec<crate::Page>,
}

#[derive(Deserialize, Debug)]
struct Page {
    object: String,
    id: String,
}