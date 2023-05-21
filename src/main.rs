pub mod cli;
pub mod commands;
pub mod config;
pub mod constants;
pub mod models;

use cli::parser;
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    let _ = dotenv();
    // Parse the arguments passed in and forward it to the correct command
    parser::parse().await.unwrap();
}
