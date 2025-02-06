use actix_web::{web, HttpResponse, Responder, HttpRequest};  // 🔥 Corrigindo a importação
use serde::Deserialize;
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use mongodb::Database;
use crate::models::User;
use bcrypt::{hash, verify};
use jsonwebtoken::{encode, EncodingKey, Header};
use std::env;
use chrono::Utc;
use solana_sdk::signature::{Keypair, Signer};
use crate::redis_client::RedisClient;  // 🔥 Importando Redis

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(serde::Serialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub async fn register(
    db: web::Data<Database>, 
    redis: web::Data<RedisClient>,  // 🔥 Redis injetado
    req: web::Json<RegisterRequest>
) -> impl Responder {
    let users = db.collection::<User>("users");

    // Verificar se o usuário já existe
    if let Ok(Some(_)) = users.find_one(doc! {"email": &req.email}, None).await {
        return HttpResponse::Conflict().json("Usuário já existe");
    }

    let hashed_password = hash(&req.password, 10).unwrap();

    // Criando uma nova wallet Solana
    let keypair = Keypair::new();
    let public_key = keypair.pubkey().to_string();

    let new_user = User {
        id: Some(ObjectId::new()),
        username: req.username.clone(),
        email: req.email.clone(),
        password_hash: hashed_password,
        solana_wallet: Some(public_key.clone()),
        is_approved: false,
        created_at: mongodb::bson::DateTime::from_system_time(std::time::SystemTime::now()),
    };

    if let Err(e) = users.insert_one(new_user, None).await {
        log::error!("❌ Erro ao inserir usuário no MongoDB: {:?}", e);
        return HttpResponse::InternalServerError().json("Erro ao criar usuário");
    }

    HttpResponse::Created().json(serde_json::json!({
        "message": "User registered successfully",
        "solana_wallet": public_key
    }))
}

pub async fn login(
    db: web::Data<Database>, 
    redis: web::Data<RedisClient>,
    req: web::Json<LoginRequest>
) -> impl Responder {
    let users = db.collection::<User>("users");

    if let Some(user) = users.find_one(doc! {"email": &req.email}, None).await.unwrap() {
        if verify(&req.password, &user.password_hash).unwrap() {
            let expiration = Utc::now()
                .checked_add_signed(chrono::Duration::hours(24))
                .expect("valid timestamp")
                .timestamp() as usize;

            let claims = Claims {
                sub: user.id.unwrap().to_hex(),
                exp: expiration,
            };

            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(env::var("JWT_SECRET").expect("JWT_SECRET not set").as_ref()),
            )
            .unwrap();

            // 🔥 Definir expiração de 10 segundos para testes
            redis.set_token(&token, &user.id.unwrap().to_hex(), 100).await;

            return HttpResponse::Ok().json(serde_json::json!({
                "token": token,
                "solana_wallet": user.solana_wallet.unwrap_or_else(|| "No wallet assigned".to_string()),
            }));
        }
    }

    HttpResponse::Unauthorized().json("Invalid credentials")
}


pub async fn get_token(
    redis: web::Data<RedisClient>, 
    req: HttpRequest  // 🔥 Adicionando corretamente o HttpRequest
) -> impl Responder {
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            let token = auth_str.replace("Bearer ", "");  // 🔥 Remove "Bearer "

            // 🔥 Verifica se o token existe no Redis
            if let Some(user_id) = redis.get_token(&token).await {
                return HttpResponse::Ok().json(serde_json::json!({
                    "user_id": user_id
                }));
            } else {
                return HttpResponse::Unauthorized().json("Invalid or expired token");
            }
        }
    }

    HttpResponse::BadRequest().json("Missing Authorization Header")
}

pub async fn logout(
    redis: web::Data<RedisClient>,
    req: HttpRequest
) -> impl Responder {
    if let Some(auth_header) = req.headers().get("Authorization") {
        let token = auth_header.to_str().unwrap().replace("Bearer ", "");
        
        // 🔥 Removendo token do Redis
        redis.delete_token(&token).await;

        return HttpResponse::Ok().json(serde_json::json!({
            "message": "Logout realizado com sucesso!"
        }));
    }

    HttpResponse::BadRequest().json("Token não encontrado no cabeçalho")
}




