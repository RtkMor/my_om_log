use actix_web::{get, put, post, delete, web, HttpResponse, Responder};
use mongodb::{
    bson::{self, doc, oid::ObjectId, Document},
    Collection, Database
};
use serde::{Deserialize, Serialize};
use futures::StreamExt;

#[derive(Debug, Serialize, Deserialize)]
struct Product {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    title: String,
    description: String,
    price: u32,
    quantity: u8,
    img: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProductRequest {
    product_id: String,
}

#[get("/products")]
pub async fn get_products(db: web::Data<Database>) -> impl Responder {
    let products_collection: Collection<Document> = db.collection("products");

    let filter = doc! {}; // Empty filter to match all documents

    let mut cursor = match products_collection.find(filter).await {
        Ok(cursor) => cursor,
        Err(_) => return HttpResponse::InternalServerError().body("Error retrieving products"),
    };

    let mut products: Vec<Product> = Vec::new(); // Explicitly specifying the type here
    while let Some(result) = cursor.next().await {
        match result {
            Ok(document) => {
                // Convert BSON Document to Product struct
                match bson::from_bson(bson::Bson::Document(document)) {
                    Ok(product) => products.push(product),
                    Err(e) => eprintln!("Failed to convert document to Product: {:?}", e),
                }
            }
            Err(e) => eprintln!("Error retrieving document: {:?}", e),
        }
    }

    HttpResponse::Ok().json(products)
}

#[put("/products/{id}")]
pub async fn update_product(
    db: web::Data<Database>,
    path: web::Path<String>,
    product: web::Json<Product>,
) -> impl Responder {
    let id_str = path.into_inner();
    let id = match ObjectId::parse_str(&id_str) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid product ID format"),
    };

    let filter = doc! { "_id": id };
    let update_doc = bson::to_document(&product.into_inner()).unwrap();

    let mut update_doc = update_doc;
    update_doc.remove("_id");

    let update = doc! { "$set": update_doc };
    let products_collection: Collection<Document> = db.collection("products");

    match products_collection.update_one(filter, update).await {
        Ok(result) if result.matched_count > 0 => HttpResponse::Ok().body("Product updated successfully"),
        Ok(_) => HttpResponse::NotFound().body("Product not found"),
        Err(_) => HttpResponse::InternalServerError().body("Error updating product"),
    }
}


#[delete("/products/{id}")]
pub async fn delete_product(
    db: web::Data<Database>,
    path: web::Path<String>,
) -> impl Responder {
    // Extract the product ID from the path
    println!("Hi");
    let id_str = path.into_inner();
    let id = match ObjectId::parse_str(&id_str) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid product ID format"),
    };

    // Define the filter for the product to be deleted
    let filter = doc! { "_id": id };

    // Get the products collection
    let products_collection: Collection<Document> = db.collection("products");

    // Perform the deletion operation
    match products_collection.delete_one(filter).await {
        Ok(result) if result.deleted_count > 0 => HttpResponse::Ok().body("Product deleted successfully"),
        Ok(_) => HttpResponse::NotFound().body("Product not found"),
        Err(_) => HttpResponse::InternalServerError().body("Error deleting product"),
    }
}

#[post("/get-product")]
pub async fn get_product(
    db: web::Data<Database>,
    product_request: web::Json<ProductRequest>,
) -> impl Responder {
    let product_id = &product_request.product_id;
    let id = match ObjectId::parse_str(&product_id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid product ID format"),
    };

    let filter = doc! { "_id": id };
    let products_collection: Collection<Document> = db.collection("products");

    match products_collection.find_one(filter).await {
        Ok(Some(document)) => {
            // Convert BSON Document to Product struct
            match bson::from_bson::<Product>(bson::Bson::Document(document)) {
                Ok(product) => HttpResponse::Ok().json(product),
                Err(_) => HttpResponse::InternalServerError().body("Failed to convert document to Product"),
            }
        }
        Ok(None) => HttpResponse::NotFound().body("Product not found"),
        Err(_) => HttpResponse::InternalServerError().body("Error retrieving product"),
    }
}

