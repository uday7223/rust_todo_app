use sqlx::{Pool, Postgres};

pub async fn connect_db() -> Pool<Postgres> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to DB");

    // Run migrations on startup
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    println!("✅ Migrations applied successfully");

    pool
}
