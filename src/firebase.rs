pub mod auth;

use jsonwebtoken;
use reqwest;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use thiserror::Error;

#[derive(Clone, Debug, Deserialize)]
pub struct FirebaseConfig {
    pub project_id: String,
    pub private_key_id: String,
    pub private_key: String,
    pub client_email: String,
    pub client_id: String,
}

impl FirebaseConfig {
    pub fn new() -> FirebaseConfig {
        let file = File::open("firebase.prod.json").expect("cannot read firebase credential");
        let reader = BufReader::new(file);
        let config: FirebaseConfig = serde_json::from_reader(reader).unwrap();
        config
    }
}

#[derive(Error, Debug, PartialOrd, PartialEq)]
pub enum FirebaseError {
    #[error("firebase error: {0}")]
    Internal(String),
}

impl From<jsonwebtoken::errors::Error> for FirebaseError {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        Self::Internal(e.to_string())
    }
}

impl From<reqwest::Error> for FirebaseError {
    fn from(e: reqwest::Error) -> Self {
        Self::Internal(e.to_string())
    }
}

pub type FirebaseResult<R> = Result<R, FirebaseError>;
