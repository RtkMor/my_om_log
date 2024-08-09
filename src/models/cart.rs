use mongodb::bson::{self, doc, oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CartProduct {
    #[serde(rename = "_id")]
    pub id: Option<ObjectId>,
    pub user_id: Option<ObjectId>,
    pub products: Vec<CartItem>,

}

#[derive(Debug, Serialize, Deserialize)]
pub struct CartItem {
    pub product_id: ObjectId,
    pub quantity: u32,
}