mod ddb;
mod domain;
mod firebase;
mod graphql;

#[macro_use]
extern crate diesel;

use crate::firebase::auth;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use dotenv;
use juniper_actix::{graphql_handler, playground_handler};
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let env_path = env::var("RUST_ENV").expect("should set env");
    dotenv::from_path(&env_path).expect("cannot read env");

    let port = env::var("PORT").unwrap_or("3000".to_string());

    println!("running server on port {}", port);

    HttpServer::new(|| {
        let schema = graphql::new_schema();

        App::new()
            .data(schema)
            .service(
                web::resource("/graphql")
                    .route(web::post().to(graphql_route))
                    .route(web::get().to(graphql_route)),
            )
            .service(web::resource("/playground").route(web::get().to(playground_route)))
            .service(web::resource("/health_check").route(web::get().to(health_check_route)))
    })
    .bind(format!("0.0.0.0:{}", port))
    .unwrap()
    .run()
    .await
}

async fn health_check_route() -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok().body("ok"))
}

async fn playground_route() -> actix_web::Result<HttpResponse> {
    playground_handler("/graphql", None).await
}

async fn graphql_route(
    req: HttpRequest,
    payload: web::Payload,
    schema: web::Data<graphql::Schema>,
) -> actix_web::Result<HttpResponse> {
    // 開発用
    let authorized_user_id: Option<String> = match req.headers().get("x-user-id") {
        Some(v) => v.to_str().map(|id| Some(id.to_string())).unwrap_or(None),
        None => auth(&req).await.into(),
    };

    if let Some(id) = authorized_user_id.clone() {
        println!("login user id: {}", id);
    }

    let context = graphql::Context { authorized_user_id };
    graphql_handler(&schema, &context, req, payload).await
}

async fn auth(req: &HttpRequest) -> Option<String> {
    let token_header: Option<String> = match req.headers().get("authorization") {
        Some(v) => v.to_str().map(|id| Some(id.to_string())).unwrap_or(None),
        None => None,
    };

    if let None = token_header {
        return None;
    }

    let token = token_header.unwrap_or("".to_string());
    if token.len() < 7 {
        return None;
    }

    let result = auth::verify_id_token(&token[7..]).await;

    match result {
        Ok(id) => Some(id),
        Err(_) => None,
    }
}
