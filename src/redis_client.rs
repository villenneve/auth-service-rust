use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client, IntoConnectionInfo};
use std::env;
use log::info;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct RedisClient {
    pub conn: Arc<Mutex<ConnectionManager>>,
}

impl RedisClient {
    pub async fn new() -> Self {
        let redis_url = env::var("REDIS_URI").expect("⚠️ REDIS_URI não encontrada no .env");

        let client = Client::open(redis_url.clone())
            .expect("❌ Falha ao conectar ao Redis Cloud");

        // 🔥 Configurar conexão segura
        let conn = ConnectionManager::new(client)
            .await
            .expect("❌ Erro ao criar conexão Redis Cloud");

        info!("✅ Conectado ao Redis Cloud!");

        RedisClient { conn: Arc::new(Mutex::new(conn)) }
    }

    pub async fn set_token(&self, key: &str, value: &str, ttl: usize) {
        let mut conn = self.conn.lock().await;
        let _: () = conn
            .set_ex(key, value, ttl)
            .await
            .expect("❌ Erro ao salvar token no Redis Cloud");
    }

    pub async fn get_token(&self, key: &str) -> Option<String> {
        let mut conn = self.conn.lock().await;
        conn.get(key).await.ok()
    }

    pub async fn delete_token(&self, key: &str) {
        let mut conn = self.conn.lock().await;
        let _: () = conn
            .del(key)
            .await
            .expect("❌ Erro ao deletar token no Redis Cloud");
    }
}
