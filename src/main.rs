use actix_web::{App, HttpServer};
use dotenv::dotenv;
use log::info;
use crate::database::connect_db;
use crate::routes::config;
use crate::redis_client::RedisClient;

mod database;
mod handlers;
mod models;
mod routes;
mod redis_client; // 🔥 Certifique-se de incluir isso

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 🔥 Carregar variáveis de ambiente
    dotenv().ok();
    env_logger::init();
    info!("📌 Variáveis de ambiente carregadas com sucesso.");

    // 🔗 Conectar ao MongoDB
    let db = connect_db().await;
    info!("✅ Conectado ao MongoDB (auth_db).");

    // 🔗 Conectar ao Redis
    let redis = RedisClient::new().await;
    info!("✅ Conectado ao Redis.");

    // 🚀 Iniciar o servidor HTTP
    let server_address = "0.0.0.0:4000";
    info!("🚀 Servidor Auth Service rodando em http://127.0.0.1:4000");

    HttpServer::new(move || {
        info!("📌 Carregando rotas..."); // 🔥 Log para verificar carregamento de rotas

        App::new()
            .app_data(actix_web::web::Data::new(db.clone()))
            .app_data(actix_web::web::Data::new(redis.clone()))
            .configure(config) // 🔧 Carregar rotas
    })
    .bind(server_address)?
    .run()
    .await
}
