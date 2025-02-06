use actix_web::{web, Scope};
use crate::handlers::{register, login, get_token, logout};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")  // 🔥 O escopo está vazio, então as rotas são diretas
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            .route("/get_token", web::get().to(get_token))  // 🔥 Certifique-se de que está aqui!
            .route("/logout", web::delete().to(logout))
    );
}
