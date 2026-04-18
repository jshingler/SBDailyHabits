
use std::collections::HashMap;
use std::fmt::format;
use std::error;
use crate::config::{Config, CONFIG};
use log::{error, warn, info, debug, trace};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
// use serde_json::{json, Value};
use serde_json::{json, Result, Value};

use chrono::{Datelike, Local};
use once_cell::sync::Lazy;
use crate::ApiResponse;

pub fn get_hmd() -> Vec<(String, String)> {
    info!("Get Habits Master Data -- Start");

    let query = json!( {
    "filter": {
        "and": [
            {
                "property": "Status",
                "select": {
                     "equals": "Active"
                 }
            }
        ]
    },
    "sorts": [
        {
            "property": "Name",
            "direction": "ascending"
        }
    ]
});

    // info!("Filter: {}", query);

    let url = &CONFIG.notion.url;
    let notion_database_url = format!("{}/databases/{}/query", &url, &CONFIG.notion.habits_master_database_id);

    let http_client = Client::new();
    let http_result = http_client.post(notion_database_url )
        .header("Authorization", &CONFIG.notion.token)
        .header("Notion-Version", &CONFIG.notion.version)
        .header("Content-Type", "application/json")
        .json(&query)
        .send();

    let mut id = String::from("");
    let mut name = String::from("");
    let mut ids_and_names:Vec<(String, String)> = vec![];

    if http_result.is_ok() {
        let result_json = http_result.ok().unwrap().text().unwrap();
        // let parsed: ApiResponse = serde_json::from_str(&result_json).unwrap();
        // info!("Response UnParsed: {:#?}", result_json);

        let v: Value = serde_json::from_str(&result_json).unwrap();

        // Extract the array and collect the ids and names into a Vec<(String, String)>
        ids_and_names = v["results"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|result| {
                // Extract "id" and "name" safely
                let id : String = result["id"].to_string();
                let name = result["properties"]["Name"]["title"][0]["plain_text"]
                    .as_str()?
                    .to_string();

                Some((id, name))
            })
            .collect();
        
        info!("ID: {:#?}",v["results"][0]["properties"]["Name"]);
        id = v["results"][0]["id"].to_string();
        name = v["results"][0]["properties"]["Name"]["title"][0]["plain_text"].to_string();
    }

    info!("Get Habits Master Data -- End");
    
    ids_and_names
    
}