use actix_web::{web, HttpResponse, Responder, post};
use mongodb::{
    bson::{doc, Document, Bson},
    options::UpdateOptions,
    Database,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
pub struct CartItem {
    pub product_id: String,
    pub quantity: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddToCartRequest {
    pub email: String,
    pub products: Vec<CartItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserCartInfo {
    pub email: String,
}

#[post("/carts")]
pub async fn add_to_cart(
    db: web::Data<Database>,
    cart_request: web::Json<AddToCartRequest>,
) -> impl Responder {
    let collection: mongodb::Collection<Document> = db.collection("carts");
    let cart_request = cart_request.into_inner();

    // Convert CartItem to BSON Document
    let products_bson: Vec<Document> = cart_request.products.iter().map(|item| {
        doc! {
            "product_id": &item.product_id,
            "quantity": item.quantity
        }
    }).collect();

    // Define the filter to find the user's cart by email
    let filter = doc! { "email": &cart_request.email };

    // Fetch the user's cart
    let cart_doc = match collection.find_one(filter.clone()).await {
        Ok(Some(cart)) => cart,
        Ok(None) => doc! { "email": &cart_request.email, "products": [] },
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    // Check for existing products in the cart
    let existing_products = match cart_doc.get_array("products") {
        Ok(arr) => arr.iter().filter_map(|bson| bson.as_document()).cloned().collect::<Vec<Document>>(),
        Err(_) => vec![],
    };

    let mut new_products: Vec<Document> = vec![];

    for product in products_bson {
        let product_id = product.get_str("product_id").unwrap();
        if existing_products.iter().any(|existing| existing.get_str("product_id").map_or(false, |id| id == product_id)) {
            continue;
        }
        new_products.push(product);
    }

    if new_products.is_empty() {
        // No new products to add
        return HttpResponse::Ok().json(json!({ "success": true, "message": "No new products added." }));
    }

    // Define the update operation
    let update = doc! { "$push": { "products": { "$each": new_products.clone() } } };

    // Attempt to update the cart
    let update_result = collection.update_one(filter.clone(), update).await;

    match update_result {
        Ok(result) if result.matched_count > 0 => {
            // If the cart exists and was updated
            HttpResponse::Created().json(json!({ "success": true }))
        }
        Ok(_) => {
            // If the cart does not exist, create a new cart with the provided products
            let new_cart = doc! {
                "email": &cart_request.email,
                "products": &new_products
            };
            match collection.insert_one(new_cart).await {
                Ok(_) => HttpResponse::Created().json(json!({ "success": true })),
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateQuantityRequest {
    pub email: String,
    pub product_id: String,
    pub quantity: u32,
}

#[post("/update-quantity")]
pub async fn update_quantity(db: web::Data<Database>, update_request: web::Json<UpdateQuantityRequest>) -> impl Responder {
    let collection: mongodb::Collection<Document> = db.collection("carts");
    let update_request = update_request.into_inner();

    // Define the filter to find the user's cart by email
    let filter = doc! {"email": &update_request.email};

    // Define the update to set the new quantity for the specific product
    let update = doc! {
        "$set": {
            "products.$[elem].quantity": update_request.quantity
        }
    };

    // Define the array filter to target the specific product by product_id
    let array_filters = vec![
        doc! {"elem.product_id": &update_request.product_id}
    ];

    // Define the update options with array filters
    let options = UpdateOptions::builder()
        .array_filters(array_filters)
        .build();

    // Attempt to update the quantity of the product in the user's cart
    match collection.update_one(filter, update).await {
        Ok(update_result) => {
            if update_result.matched_count > 0 {
                HttpResponse::Ok().json(json!({
                    "success": true,
                    "message": "Quantity updated"
                }))
            } else {
                HttpResponse::NotFound().json(json!({
                    "success": false,
                    "message": "Cart or product not found"
                }))
            }
        }
        Err(err) => {
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "message": err.to_string()
            }))
        }
    }
}


#[post("/fetch-cart")]
pub async fn fetch_cart_details(
    db: web::Data<Database>,
    user_cart: web::Json<UserCartInfo>,
) -> impl Responder {
    let collection: mongodb::Collection<Document> = db.collection("carts");
    let user_cart = user_cart.into_inner();

    // Define the filter to find the user's cart by email
    let filter = doc! { "email": &user_cart.email };

    // Attempt to find the cart
    match collection.find_one(filter).await {
        Ok(Some(cart)) => HttpResponse::Ok().json(json!({
            "success": true,
            "cart": cart
        })),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "success": false,
            "message": "Cart not found"
        })),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteProductRequest {
    pub email: String,
    pub product_id: String,
}

#[post("/delete-product")]
pub async fn delete_cart_product(
    db: web::Data<Database>,
    delete_request: web::Json<DeleteProductRequest>,
) -> impl Responder {

    let collection: mongodb::Collection<Document> = db.collection("carts");
    let delete_request = delete_request.into_inner();

    // Define the filter to find the user's cart by email
    let filter = doc! { "email": &delete_request.email };

    // Define the update to pull (remove) the specific product from the products array
    let update = doc! {
        "$pull": {
            "products": { "product_id": &delete_request.product_id }
        }
};

    // Attempt to update the cart by removing the specified product
    match collection.update_one(filter, update).await {
        Ok(update_result) => {
            if update_result.modified_count > 0 {
                HttpResponse::Ok().json(json!({
                    "success": true,
                    "message": "Product removed from cart"
                }))
            } else {
                HttpResponse::NotFound().json(json!({
                    "success": false,
                    "message": "Product not found in cart"
                }))
            }
        }
        Err(err) => {
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "message": err.to_string()
            }))
        }
    }
}