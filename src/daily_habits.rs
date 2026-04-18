use log::info;
use reqwest::blocking::Client;
use serde_json::{json, Value};
use crate::config::CONFIG;

pub fn create_daily_habit(habit_id: & String, today_id: & String, habit_name: & String, daily_stats_id: & String ) {
    info!("create_daily_habit(habit_id: {}, today_id: {}, habit_name: {}, daily_stats_id: {} )", habit_id, today_id, habit_name, daily_stats_id);


    let query = json!({
   "parent": {
        "database_id": &CONFIG.notion.habits_database_id
    },
    "properties": {
        "Name": {
            "title": [
                {
                    "text": {
                        "content": habit_name
                    }
                }
            ]
        },
        "Habit":{
            "relation":[
               {
                  "id": habit_id.trim_matches('"')
               }
            ]
         },
        "Day":{
            "relation":[
               {
                  "id": today_id
               }
            ]
         },

        "stats":{
            "relation":[
               {
                  "id": daily_stats_id
               }
            ]
         }
    }
});

    // info!("Filter: {}", query);

    let url = &CONFIG.notion.url;
    let notion_database_url = format!("{}/pages", &url);

    let http_client = Client::new();
    let http_result = http_client.post(notion_database_url )
        .header("Authorization", &CONFIG.notion.token)
        .header("Notion-Version", &CONFIG.notion.version)
        .header("Content-Type", "application/json")
        .json(&query)
        .send();

    if http_result.is_ok() {
        let result_json = http_result.ok().unwrap().text().unwrap();
        // let parsed: ApiResponse = serde_json::from_str(&result_json).unwrap();
        info!("Response UnParsed: {:#?}", result_json);
    }

    info!("Get Habits Master Data -- End");
    
}