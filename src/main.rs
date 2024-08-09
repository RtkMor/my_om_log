pub mod controllers;
mod models;
mod db;

use actix_cors::Cors;
use actix_web::{get, web, App, HttpServer, Responder};
use controllers::{signup, login, fetch_user_details, update_user};
use controllers::{add_product,get_products,update_product,add_to_cart,delete_product,fetch_cart_details,get_product, update_quantity, delete_cart_product};
use db::get_database;

#[get("/")]
async fn index() -> impl Responder {
    "Hello, World!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = get_database().await;
    println!("Server Running!");
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        App::new()
            .app_data(web::Data::new(db.clone()))
            .wrap(cors)
            .service(index)
            .service(signup)
            .service(login)
            .service(fetch_user_details)
            .service(update_user)
            .service(add_product)
            .service(get_products)
            .service(update_product)
            .service(delete_product)
            .service(add_to_cart)
            .service(fetch_cart_details)
            .service(get_product)
            .service(update_quantity)
            .service(delete_cart_product)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}