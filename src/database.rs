use mongodb::{Client, Database};
use std::env;
use log::info;
use dotenv::dotenv;

pub async fn connect_db() -> Database {
    dotenv().ok();  // 🔥 Carregar as variáveis do .env

    let database_url = env::var("MONGO_URI").expect("⚠️ MONGO_URI não encontrada no .env");

    let client = Client::with_uri_str(&database_url)
        .await
        .expect("❌ Falha ao conectar no MongoDB");

    let db: Database = client.database("auth_db"); // 🔹 Nome do banco de autenticação

    info!("✅ Conectado ao MongoDB: auth_db");

    db
}