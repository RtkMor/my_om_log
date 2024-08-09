use actix_web::{post, get, web, HttpResponse, Responder, HttpRequest};
use mongodb::bson::doc;
use mongodb::Database;
use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, DecodingKey, Validation};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub email: String,
    pub password: String,
    pub is_admin: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateInfo {
    email: String,
    name: String
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

const SECRET_KEY: &str = "abcdefghijklmnopqrstuvwxyz";

#[get("/user")]
pub async fn fetch_user_details(req: HttpRequest, db: web::Data<Database>) -> impl Responder {
    let auth_header = req.headers().get("Authorization");
    if let Some(auth_header) = auth_header {
        if let Ok(auth_header_str) = auth_header.to_str() {
            let token = auth_header_str.trim_start_matches("Bearer ");

            let decoding_key = DecodingKey::from_secret(SECRET_KEY.as_ref());
            let validation = Validation::default();
            let decoded_token = match decode::<Claims>(token, &decoding_key, &validation) {
                Ok(decoded) => decoded,
                Err(_) => {
                    println!("Failed to decode token");
                    return HttpResponse::Unauthorized().body("Invalid token");
                }
            };

            let user_email = decoded_token.claims.sub;
            let users_collection = db.collection::<User>("users");

            let filter = doc! { "email": &user_email };
            let user: Option<User> = users_collection
                .find_one(filter)
                .await
                .expect("Error finding user");

            match user {
                Some(user) => HttpResponse::Ok().json(user),
                None => {
                    println!("User not found: {}", user_email);
                    HttpResponse::NotFound().body("User not found")
                },
            }
        } else {
            println!("Invalid Authorization header format");
            HttpResponse::Unauthorized().body("Invalid Authorization header format")
        }
    } else {
        println!("Missing Authorization header");
        HttpResponse::Unauthorized().body("Missing Authorization header")
    }
}

#[post("/update-user")]
pub async fn update_user(update_info: web::Json<UpdateInfo>, db: web::Data<Database>) -> impl Responder {
    let users_collection = db.collection::<User>("users");

    let filter = doc! { "email": &update_info.email };
    let update = doc! { "$set": { "name": &update_info.name } };

    let update_result = users_collection.update_one(filter, update).await;

    match update_result {
        Ok(result) => {
            if result.matched_count == 0 {
                HttpResponse::NotFound().body("User not found")
            } else if result.modified_count == 0 {
                HttpResponse::NotModified().body("No changes made")
            } else {
                HttpResponse::Ok().body("User updated successfully")
            }
        }
        Err(_) => HttpResponse::InternalServerError().body("Error updating user"),
    }
}