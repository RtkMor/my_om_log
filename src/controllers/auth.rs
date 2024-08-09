use actix_web::{post, web, HttpResponse, Responder};
use bcrypt::{hash, DEFAULT_COST};
use mongodb::bson::doc;
use mongodb::Database;
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, Header, EncodingKey};
use chrono::{Utc, Duration};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    name: String,
    email: String,
    password: String,
    is_admin: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginInfo {
    email: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    token: String,
}

const SECRET_KEY: &str = "abcdefghijklmnopqrstuvwxyz";

fn create_jwt(email: &str) -> String {
    let expiration = Utc::now()
        .checked_add_signed(Duration::days(7))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: email.to_owned(),
        exp: expiration as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(SECRET_KEY.as_ref()),
    )
        .expect("Error creating token")
}

#[post("/signup")]
pub async fn signup(user: web::Json<User>, db: web::Data<Database>) -> impl Responder {
    let users_collection = db.collection::<User>("users");

    // Check if email already exists
    let filter = doc! { "email": &user.email };
    let existing_user: Option<User> = users_collection
        .find_one(filter)
        .await
        .expect("Error checking user existence");

    if existing_user.is_some() {
        return HttpResponse::BadRequest().body("Email already in use");
    }

    // Check if password is at least 8 characters long
    if user.password.len() < 8 {
        return HttpResponse::BadRequest().body("Password must be at least 8 characters long");
    }

    let hashed_password = hash(&user.password, DEFAULT_COST).expect("Error hashing password");

    let new_user = User {
        name: user.name.clone(),
        email: user.email.clone(),
        password: hashed_password,
        is_admin: false,
    };

    let result = users_collection.insert_one(new_user).await;

    match result {
        Ok(_) => HttpResponse::Ok().body("User created successfully"),
        Err(_) => HttpResponse::InternalServerError().body("Error creating user"),
    }
}

#[post("/login")]
pub async fn login(info: web::Json<LoginInfo>, db: web::Data<Database>) -> impl Responder {
    let users_collection = db.collection::<User>("users");

    let filter = doc! { "email": &info.email };
    let user: Option<User> = users_collection
        .find_one(filter)
        .await
        .expect("Error finding user");

    match user {
        Some(user) => {
            if bcrypt::verify(&info.password, &user.password).expect("Error verifying password") {
                let token = create_jwt(&user.email);
                HttpResponse::Ok().json(LoginResponse { token })
            } else {
                HttpResponse::Unauthorized().body("Invalid credentials")
            }
        }
        None => HttpResponse::Unauthorized().body("Invalid credentials"),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Product {
    title: String,
    description: String,
    price: u32,
    quantity: u8,
    img: String,
}

#[post("/add_product")]
pub async fn add_product(product: web::Json<Product>, db: web::Data<Database>) -> impl Responder {

    let products_collection = db.collection("products");


    let new_product = Product {
        title: product.title.clone(),
        description: product.description.clone(),
        price: product.price,
        quantity: product.quantity,
        img: product.img.clone(),
    };

    let result = products_collection.insert_one(new_product).await;

    match result {
        Ok(_) => HttpResponse::Ok().body("Product added successfully"),
        Err(_) => HttpResponse::InternalServerError().body("Error adding product"),
    }
}