mod config;
mod habits_md;
mod daily_tracking;
mod daily_habits;

use std::collections::HashMap;
use std::fmt::format;
use std::error;
use config::CONFIG;
// use nc::get_hmd();
use log::{error, warn, info, debug, trace};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
// use serde_json::{Result, Value};

use chrono::{Datelike, Local};

// A simple type alias  to DRY.
type Result<T> = std::result::Result<T, Box<dyn error::Error + Send + Sync>>;

// #[tokio::main]
// async
fn main()  -> Result<()>  {
    log4rs::init_file("config/log4rs.yaml", Default::default())?;
    // error!("This is an error message.");
    // warn!("This is a warning message.");
    // info!("This is an info message.");
    // debug!("This is a debug message.");
    // trace!("This is a trace message.");

    info!("Hello, world!");
    // Access the configuration values
    info!("App Name: {}", &CONFIG.app.name);
    info!("App Version: {}", &CONFIG.app.version);
    info!("Database User: {}", &CONFIG.database.user);
    info!("Database Host: {}", &CONFIG.database.host);
    info!("Notion Url: {}", &CONFIG.notion.url);
    info!("Habits Master DB ID: {}", &CONFIG.notion.habits_master_database_id);
    // hi();

    // let url = "http://jsonplaceholder.typicode.com/users".parse().unwrap();
    // let url = &CONFIG.notion.url;
    // get_url(&url);
    let todays_id = daily_tracking::get_today_id();
    // let Vec(habit_id,habit_name) = habits_md::get_hmd();

    // Assuming get_hmd() returns a Vec<(String, String)>
    let habits = habits_md::get_hmd();

    for (habit_id, habit_name) in habits {
        // println!("ID: {}, Name: {}", habit_id, habit_name);
        daily_habits::create_daily_habit( &habit_id, & todays_id, & habit_name, &CONFIG.notion.daily_stats_page_id);
    }
    
    
    // get_habits_md();

    Ok(())
}

fn get_url(url: & String) {
    let http_client = Client::new();
    let http_result = http_client.get(url).send();

    if http_result.is_ok() {
        println!("{:#?}", &http_result.ok().unwrap().text());
    }
    else if http_result.is_err() {
        let my_error = http_result.err().unwrap().to_string();
        println!("Error: {:#?}", &my_error);
        error!("{:#?}", &my_error);
    }

}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    object: String,
    results: Vec<Page>,
}

#[derive(Deserialize, Debug)]
struct Page {
    object: String,
    id: String,
    r#type: Option<String>,
    created_time: String,
    last_edited_time: String,
    has_children: Option<bool>,
    paragraph: Option<Paragraph>,
    to_do: Option<ToDo>,
    properties: Option<HashMap<String, Value>>,


}

#[derive(Debug, Serialize, Deserialize)]
struct Paragraph {
    text: Vec<Text>, // Assuming a Text struct represents the "text" object
    children: Option<Vec<Block>>, // Paragraph can contain children blocks
}

#[derive(Debug, Serialize, Deserialize)]
struct ToDo {
    text: Vec<Text>, // Assuming a Text struct represents the "text" object
    checked: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Text {
    // Placeholder struct for the details omitted from "text"
    content: String,
    // Add other necessary fields here, based on the actual text structure
}

#[derive(Debug, Serialize, Deserialize)]
struct Block {
    object: String,                 // Always "block"
    id: String,                     // UUIDv4
    parent: Parent,                 // Parent object
    r#type: BlockType,              // Enum for block type
    created_time: String,           // ISO 8601 date-time
    created_by: PartialUser,        // User object (partial)
    last_edited_time: String,       // ISO 8601 date-time
    last_edited_by: PartialUser,    // User object (partial)
    archived: bool,                 // Archived status
    in_trash: bool,                 // Deleted status
    has_children: bool,             // If block has children
    #[serde(flatten)]               // The dynamic field representing the type-specific block info
    block_type_data: BlockTypeData,
}

#[derive(Debug, Serialize, Deserialize)]
struct Parent {
    r#type: String,                 // e.g., "block_id"
    block_id: Option<String>,       // Parent block ID if available
}

#[derive(Debug, Serialize, Deserialize)]
struct PartialUser {
    object: String,                 // Always "user"
    id: String,                     // User ID (UUIDv4)
}

// Enum for different block types
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum BlockType {
    Bookmark,
    Breadcrumb,
    BulletedListItem,
    Callout,
    ChildDatabase,
    ChildPage,
    Column,
    ColumnList,
    Divider,
    Embed,
    Equation,
    File,
    Heading1,
    Heading2,
    Heading3,
    Image,
    LinkPreview,
    LinkToPage,
    NumberedListItem,
    Paragraph,
    Pdf,
    Quote,
    SyncedBlock,
    Table,
    TableOfContents,
    TableRow,
    Template,
    ToDo,
    Toggle,
    Unsupported,
    Video,
}

// BlockTypeData struct to handle dynamic block content based on block type
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)] // Allow dynamic content based on block type
enum BlockTypeData {
    Paragraph(ParagraphBlock),
    ToDo(ToDoBlock),
    Heading1(HeadingBlock),
    Heading2(HeadingBlock),
    Heading3(HeadingBlock),
    // Add other block types as necessary...
    Unsupported,                    // Fallback for unsupported block types
}

// Struct for paragraph block type
#[derive(Debug, Serialize, Deserialize)]
struct ParagraphBlock {
    text: Vec<Text>,                 // Text array
    children: Option<Vec<Block>>,    // Nested blocks as children
}

// Struct for to_do block type
#[derive(Debug, Serialize, Deserialize)]
struct ToDoBlock {
    text: Vec<Text>,                 // Text array
    checked: bool,                   // Whether the to-do is checked
}

// Struct for heading block types
#[derive(Debug, Serialize, Deserialize)]
struct HeadingBlock {
    text: Vec<Text>,                 // Text array
}
