
// Database connection and server setup for the victims server application.

// Database pool configuration and connection setup
#[tokio::main]
async fn main() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite://app.db")
        .await
        .unwrap();

    let _ = pool;
}
