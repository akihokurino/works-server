pub mod ddb;
pub mod domain;
pub mod firebase;
pub mod graphql;
pub mod misoca;
pub mod task;
pub mod util;

#[macro_use]
extern crate diesel;

use jsonwebtoken;
use reqwest;
use thiserror::Error as ThisErr;

const INVOICE_BUCKET: &str = "works-userdata";
const INVOICE_PDF_DOWNLOAD_DURATION: u32 = 86400;

#[derive(ThisErr, Debug, PartialOrd, PartialEq)]
pub enum CoreError {
    #[error("notfound")]
    NotFound,
    #[error("forbidden")]
    Forbidden,
    #[error("internal error: {0}")]
    Internal(String),
}

pub type CoreResult<T> = Result<T, CoreError>;

impl From<String> for CoreError {
    fn from(v: String) -> Self {
        Self::Internal(v)
    }
}

impl From<diesel::result::Error> for CoreError {
    fn from(e: diesel::result::Error) -> Self {
        match e {
            diesel::result::Error::NotFound => Self::NotFound,
            _ => Self::Internal(e.to_string()),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for CoreError {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        Self::Internal(e.to_string())
    }
}

impl From<reqwest::Error> for CoreError {
    fn from(e: reqwest::Error) -> Self {
        Self::Internal(e.to_string())
    }
}
