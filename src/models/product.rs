use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]

pub struct Product{
    pub title:String,
    pub description:String,
    pub price:u32,
    pub quantity:u8,
    pub image: String
}