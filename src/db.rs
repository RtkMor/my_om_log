use mongodb::{Client, Database};
use std::env;
use dotenv::dotenv;

pub async fn get_database() -> Database {
    dotenv().ok();
    println!("Connected to database!");
    let client_uri = env::var("MONGODB_URI").expect("You must set the MONGODB_URI environment variable!");
    let client = Client::with_uri_str(&client_uri).await.expect("Failed to initialize client.");
    client.database("om_logistics")
}