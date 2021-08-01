use app_core::misoca;
use app_core::task;
use app_core::CoreError;
use chrono::{DateTime, Utc};
use dotenv;
use std::env;

#[tokio::main]
async fn main() {
    let env_path = env::var("RUST_ENV").expect("should set env");
    dotenv::from_path(&env_path).expect("cannot read env");

    let args: Vec<String> = env::args().collect();
    let command = &args[1];
    let now: DateTime<Utc> = Utc::now();

    let misoca_cli = misoca::Client::new(
        env::var("MISOCA_CLIENT_ID").unwrap(),
        env::var("MISOCA_SECRET").unwrap(),
        env::var("MISOCA_REDIRECT_URL").unwrap(),
    );

    let result = if command == "sync-invoice" {
        task::sync_invoice::exec(misoca_cli, now).await
    } else if command == "create-invoice" {
        task::create_invoice::exec(misoca_cli, now).await
    } else {
        Err(CoreError::Internal("unknown command".to_string()))
    };

    match result {
        Ok(_) => println!("success {}", command),
        Err(err) => println!("error {}: {:#?}", command, err),
    }
}
