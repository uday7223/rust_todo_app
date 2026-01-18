mod auth;
mod db;
mod models;
mod routes;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let _db = db::connect_db().await;

    println!("Rust Todo API initialized");
}
