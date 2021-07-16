mod ddb;
mod domain;
mod firebase;
mod graphql;

#[macro_use]
extern crate diesel;

use crate::firebase::auth;
use crate::firebase::FirebaseError;
use actix_web::http::StatusCode;
use actix_web::{error, web, App, HttpRequest, HttpResponse, HttpServer, ResponseError};
use derive_more::Error;
use dotenv;
use juniper_actix::{graphql_handler, playground_handler};
use serde::Serialize;
use std::env;
use std::fmt;

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
    let authorized_user_id = match req.headers().get("x-user-id") {
        Some(v) => Some(v.to_str().map_err(|e| error::ErrorBadRequest(e))?.into()),
        None => Some(
            auth(&req)
                .await
                .map_err(|e| error::ErrorBadRequest(e))?
                .into(),
        ),
    };

    if let Some(id) = authorized_user_id.clone() {
        println!("login user id: {}", id);
    }

    let context = graphql::Context { authorized_user_id };
    graphql_handler(&schema, &context, req, payload).await
}

async fn auth(req: &HttpRequest) -> actix_web::Result<String> {
    let token_header: Option<String> = match req.headers().get("authorization") {
        Some(v) => Some(v.to_str().map_err(|e| error::ErrorBadRequest(e))?.into()),
        None => None,
    };

    if let None = token_header {
        return Err(error::ErrorUnauthorized(AppError::UnAuthenticate));
    }

    let token = token_header.unwrap();
    if token.len() < 7 {
        return Err(error::ErrorUnauthorized(AppError::UnAuthenticate));
    }

    auth::verify_id_token(&token[7..])
        .await
        .map_err(AppError::from)
        .map(|id| Ok(id))?
}

#[derive(Error, Debug)]
pub enum AppError {
    UnAuthenticate,
    Internal,
}

impl AppError {
    pub fn name(&self) -> String {
        match self {
            Self::UnAuthenticate => "認証エラーです".to_string(),
            Self::Internal => "サーバーエラーです".to_string(),
        }
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match *self {
            Self::UnAuthenticate => StatusCode::UNAUTHORIZED,
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_response = ErrorResponse {
            code: status_code.as_u16(),
            message: self.to_string(),
            error: self.name(),
        };
        HttpResponse::build(status_code).json(error_response)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    code: u16,
    error: String,
    message: String,
}

impl From<FirebaseError> for AppError {
    fn from(_e: FirebaseError) -> Self {
        Self::Internal
    }
}
