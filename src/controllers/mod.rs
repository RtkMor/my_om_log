pub mod auth;
pub mod user_details;
mod cart;
mod product;

pub use auth::{signup, login, add_product};
pub use user_details::{fetch_user_details, update_user};

pub use product::{get_products,update_product,delete_product,get_product};

pub use cart::{add_to_cart,fetch_cart_details, update_quantity, delete_cart_product};