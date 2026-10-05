
// Database connection and server setup for the victims server application.

// Database pool configuration and connection setup

use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use fake::{faker::internet::en::SafeEmail, faker::name::en::Name, Fake};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite, FromRow};
use uuid::Uuid;

// Struct representing a user in the database which can be turned into JSON
#[derive(Debug, Serialize, Deserialize, FromRow)]
struct User {
    id: i64,
    name: String,
    email: String,
    password_hash: String,
    payment_token: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    println!("Starting the victims server.");

    // Database connection and server setup
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:app.db?mode=rwc")
        .await?;

    // Create the tables
    sqlx::migrate!("./migrations").run(&pool).await?;

    //C heck if the database is empty and seed it with fake data if necessary
    seed_database_if_empty(&pool).await?;

    // Setting up Axum Router and adding the database pool (State) to the routes
    let app = Router::new()
        .route("/", get(frontend_handler))
        .route("/api/users", get(get_users_api))
        .with_state(pool); // Sharing pool through the web interface

    // Start to listen to the server on localhost:3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("App interface is being listened: http://127.0.0.1:3000");
    axum::serve(listener, app).await?;

    Ok(())
}


async fn seed_database_if_empty(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;

    if count.0 == 0 {
        println!("Database is empty. Generating fake data...");
        for _ in 0..50 {
            let name: String = Name().fake();
            let email: String = SafeEmail().fake();
            let fake_hash = Uuid::new_v4().to_string().replace("-", ""); 
            let fake_token = format!("tok_{}", Uuid::new_v4().to_string().replace("-", ""));

            sqlx::query("INSERT INTO users (name, email, password_hash, payment_token) VALUES (?, ?, ?, ?)")
                .bind(name)
                .bind(email)
                .bind(fake_hash)
                .bind(fake_token)
                .execute(pool)
                .await?;
        }
        println!("50 fake users added successfully.");
    }
    Ok(())

}

// --- WEB ROUTE HANDLERS---

// API: Turns everything in the users table into JSON and returns it
async fn get_users_api(State(pool): State<Pool<Sqlite>>) -> impl IntoResponse {
    let users = sqlx::query_as::<_, User>("SELECT * FROM users")
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

    Json(users)
}

// FRONTEND: An interface that fetches data from the API and displays it in an HTML table
async fn frontend_handler() -> Html<&'static str> {
    Html(r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Company Internal Customer Management Panel</title>
            <style>
                body { font-family: Arial, sans-serif; background-color: #f4f4f9; padding: 20px; }
                .container { max-width: 1000px; margin: 0 auto; background: white; padding: 20px; border-radius: 8px; box-shadow: 0 4px 6px rgba(0,0,0,0.1); }
                h1 { color: #333; text-align: center; }
                table { width: 100%; border-collapse: collapse; margin-top: 20px; }
                th, td { padding: 12px; text-align: left; border-bottom: 1px solid #ddd; }
                th { background-color: #0056b3; color: white; }
                tr:hover { background-color: #f1f1f1; }
                .token-blur { font-family: monospace; color: #d9534f; }
            </style>
        </head>
        <body>
            <div class="container">
                <h1>🔒 Deniz GmbH - Customer Database</h1>
                <p style="text-align:center; color: #666;">Only authorized personnel can access. Data is end-to-end encrypted (Simulation).</p>
                <table id="usersTable">
                    <thead>
                        <tr>
                            <th>ID</th>
                            <th>Customer Name</th>
                            <th>E-Mail</th>
                            <th>Password Hash (Partial)</th>
                            <th>Payment Token (Partial)</th>
                        </tr>
                    </thead>
                    <tbody>
                        <!-- Data will be loaded here with JavaScript -->
                    </tbody>
                </table>
            </div>

            <script>
                async function loadUsers() {
                    const response = await fetch('/api/users');
                    const users = await response.json();
                    const tbody = document.querySelector('#usersTable tbody');
                    
                    users.forEach(user => {
                        const tr = document.createElement('tr');
                        // Show only first 8 characters of password hash and first 12 characters of payment token for security
                        tr.innerHTML = `
                            <td>${user.id}</td>
                            <td>${user.name}</td>
                            <td>${user.email}</td>
                            <td class="token-blur">${user.password_hash.substring(0, 8)}********</td>
                            <td class="token-blur">${user.payment_token.substring(0, 12)}********</td>
                        `;
                        tbody.appendChild(tr);
                    });
                }
                loadUsers();
            </script>
        </body>
        </html>
    "#)
}